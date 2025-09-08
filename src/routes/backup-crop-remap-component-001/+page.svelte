<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { open } from '@tauri-apps/plugin-dialog';
    import { appDataDir } from '@tauri-apps/api/path';
    import Konva from 'konva';
    import { onMount } from 'svelte';

    let sourceDir: string | null = null;
    let outputDir: string | null = null;
    
    // Dynamic label selection
    let datasetSummary: any = null;
    let availableLabels: string[] = [];
    let selectedParentLabel: string = "person"; // Default
    let selectedChildLabels: string[] = []; // Dynamic selection
    let datasetLoaded: boolean = false;
    let analyzing: boolean = false;
    let paddingFactor: number = 1.2; // Default 20% padding

    // Auto-preview state
    let previewImages: any[] = []; // 5 randomly selected images for preview
    let previewLoading: boolean = false;

    let loading: boolean = false;
    let successMessage: string | null = null;
    let errorMessage: string | null = null;

    // Preview modal state
    let showPreviewModal: boolean = false;
    let previewModalImage: any = null;
    
    // KonvaJS variables - Enhanced
    let konvaContainer: HTMLDivElement;
    let stage: Konva.Stage | null = null;
    let mainLayer: Konva.Layer | null = null;
    let annotationLayer: Konva.Layer | null = null;
    let uiLayer: Konva.Layer | null = null;
    let imageNode: Konva.Image | null = null;
    let transformer: Konva.Transformer | null = null;

    // Enhanced state for KonvaJS
    let scale = 1;
    let stageX = 0;
    let stageY = 0;
    let selectedAnnotation: any = null;
    let annotationGroups: Konva.Group[] = [];
    let isDragging = false;
    let lastPointerPosition = { x: 0, y: 0 };

    // Handle keyboard events for modal and KonvaJS controls
    function handleKeydown(event: KeyboardEvent) {
        if (event.key === "Escape") {
            if (showPreviewModal) {
                closePreviewModal();
            } else if (selectedAnnotation) {
                deselectAnnotation();
            }
        }

        // KonvaJS keyboard shortcuts
        if (showPreviewModal && stage) {
            switch (event.key.toLowerCase()) {
                case 'delete':
                case 'backspace':
                    event.preventDefault();
                    deleteSelectedAnnotation();
                    break;
                case 'a':
                    if (event.ctrlKey || event.metaKey) {
                        event.preventDefault();
                        selectAllAnnotations();
                    }
                    break;
                case 'escape':
                    event.preventDefault();
                    deselectAnnotation();
                    break;
                case '=':
                case '+':
                    event.preventDefault();
                    zoomIn();
                    break;
                case '-':
                case '_':
                    event.preventDefault();
                    zoomOut();
                    break;
                case '0':
                    event.preventDefault();
                    resetZoom();
                    break;
                case 'r':
                    event.preventDefault();
                    fitToScreen();
                    break;
            }
        }
    }

    async function selectDirectory(type: 'source' | 'output') {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: `Select ${type === 'source' ? 'Source' : 'Output'} Directory`,
            });

            if (selected && typeof selected === 'string') {
                if (type === 'source') {
                    sourceDir = selected as string;
                    // Reset dataset analysis when source changes
                    datasetSummary = null;
                    availableLabels = [];
                    datasetLoaded = false;
                    selectedChildLabels = [];
                    // Reset preview state
                    previewImages = [];
                    previewLoading = false;
                    // Automatically analyze dataset after selecting source directory
                    await analyzeDataset();
                } else {
                    outputDir = selected;
                }
                // Clear messages when a directory is selected
                successMessage = null;
                errorMessage = null;
            }
        } catch (err) {
            console.error("Error selecting directory:", err);
            errorMessage = `Failed to select directory: ${err instanceof Error ? err.message : String(err)}`;
        }
    }

    function suggestParentLabel(labels: string[]): string {
        // Priority order for common parent labels
        const commonParents = ['person', 'people', 'human', 'worker', 'individual'];
        
        for (const parent of commonParents) {
            if (labels.includes(parent)) {
                return parent;
            }
        }
        
        // Fallback to most common label (by count)
        if (datasetSummary?.label_counts) {
            const sortedLabels = Object.entries(datasetSummary.label_counts)
                .sort(([,a], [,b]) => (b as number) - (a as number))
                .map(([label]) => label);
            return sortedLabels[0] || 'person';
        }
        
        return labels[0] || 'person';
    }

    function suggestChildLabels(labels: string[]): string[] {
        const safetyEquipment = [
            'safety_helmet', 'helmet', 'hard_hat',
            'reflective_vest', 'vest', 'safety_vest', 
            'body_harness', 'harness', 'safety_harness',
            'gloves', 'safety_gloves',
            'boots', 'safety_boots'
        ];
        
        return labels.filter(label => 
            safetyEquipment.some(safety => 
                label.toLowerCase().includes(safety.toLowerCase())
            )
        );
    }

    async function analyzeDataset() {
        if (!sourceDir) return;
        
        try {
            analyzing = true;
            errorMessage = null;
            
            console.log("Analyzing dataset:", sourceDir);
            const result = await invoke("get_labelme_summary", { path: sourceDir as string });
            datasetSummary = JSON.parse(result);
            
            console.log("Dataset summary:", datasetSummary);
            
            // Extract available labels
            availableLabels = Object.keys(datasetSummary.label_counts || {});
            
            if (availableLabels.length === 0) {
                errorMessage = "No labels found in the dataset. Please check if the directory contains LabelMe JSON files.";
                return;
            }
            
            // Set smart defaults
            selectedParentLabel = suggestParentLabel(availableLabels);
            selectedChildLabels = suggestChildLabels(availableLabels);

            datasetLoaded = true;

            console.log("Dataset analysis complete, triggering auto-preview...");
            // Auto-preview 5 random annotated images
            await autoPreviewAnnotatedImages();
            
        } catch (err) {
            console.error("Error analyzing dataset:", err);
            errorMessage = `Failed to analyze dataset: ${err instanceof Error ? err.message : String(err)}`;
            datasetSummary = null;
            availableLabels = [];
            datasetLoaded = false;
        } finally {
            analyzing = false;
        }
    }

    // Function to automatically preview 5 random annotated images
    async function autoPreviewAnnotatedImages() {
        try {
            previewLoading = true;
            previewImages = [];

            if (!datasetSummary || !sourceDir) {
                console.log("No dataset available for preview");
                return;
            }

            console.log("Starting auto-preview of annotated images");

            // Create a temporary directory for previews in app data directory
            const appData = await appDataDir();
            const tempDir = `${appData}previews_${Date.now()}`;

            // Generate annotated preview images using the backend
            const result = await invoke("generate_annotated_previews", {
                sourceDir: sourceDir as string,
                numPreviews: 5,
                tempDir: tempDir
            }) as string;

            const data = JSON.parse(result);

            if (data.annotated_images && data.annotated_images.length > 0) {
                console.log(`Loaded ${data.annotated_images.length} images with annotation data`);

                // Convert file paths to proper Tauri URLs and prepare preview data
                const selectedImages = data.annotated_images.map((imageData: any, index: number) => ({
                    id: `preview_${index}`,
                    path: imageData.path,
                    previewUrl: convertFileSrc(imageData.path), // Load original clean image
                    name: `Preview ${index + 1}`,
                    annotations: imageData.annotations || [] // Draw annotations with KonvaJS
                }));

                previewImages = selectedImages;
                console.log(`Auto-preview ready with ${selectedImages.length} annotated images`);
            } else {
                console.log("No annotated preview images were generated");
            }

        } catch (err) {
            console.error("Error in auto-preview:", err);
            previewImages = [];
        } finally {
            previewLoading = false;
        }
    }

    // Function to open preview modal
    function openPreviewModal(image: any) {
        console.log('Opening preview modal for image:', image);
        previewModalImage = image;
        showPreviewModal = true;
        // Initialize KonvaJS after modal opens
        setTimeout(() => {
            console.log('Initializing Konva stage after timeout');
            initializeKonvaStage(image);
        }, 300); // Increased timeout for better DOM readiness
    }

    // Function to close preview modal
    function closePreviewModal() {
        showPreviewModal = false;
        previewModalImage = null;
        // Clean up Konva stage
        if (stage) {
            stage.destroy();
            stage = null;
            mainLayer = null;
            annotationLayer = null;
            uiLayer = null;
            imageNode = null;
            transformer = null;
        }
        // Reset state
        scale = 1;
        stageX = 0;
        stageY = 0;
        selectedAnnotation = null;
        annotationGroups = [];
        isDragging = false;
    }
    
    // Initialize KonvaJS stage with advanced features
    function initializeKonvaStage(image: any) {
        console.log('initializeKonvaStage called with:', image);
        console.log('konvaContainer:', konvaContainer);

        if (!konvaContainer) {
            console.error('konvaContainer is not available');
            return;
        }

        // Get container dimensions
        const containerRect = konvaContainer.getBoundingClientRect();
        console.log('Container dimensions:', containerRect);

        // Clean up existing stage
        if (stage) {
            stage.destroy();
        }

        // Create new stage with container dimensions
        const stageWidth = Math.max(containerRect.width || 800, 800);
        const stageHeight = Math.max(containerRect.height || 600, 600);

        console.log('Creating stage with dimensions:', stageWidth, 'x', stageHeight);

        // Create stage with enhanced configuration
        stage = new Konva.Stage({
            container: konvaContainer,
            width: stageWidth,
            height: stageHeight,
            x: stageX,
            y: stageY,
            scaleX: scale,
            scaleY: scale,
            draggable: true,
        });

        // Create multiple layers for better organization
        mainLayer = new Konva.Layer();
        annotationLayer = new Konva.Layer();
        uiLayer = new Konva.Layer();

        stage.add(mainLayer);
        stage.add(annotationLayer);
        stage.add(uiLayer);

        // Create transformer for editing annotations
        transformer = new Konva.Transformer({
            rotateEnabled: true,
            borderEnabled: true,
            borderStroke: '#4A90E2',
            borderStrokeWidth: 2,
            anchorFill: '#4A90E2',
            anchorStroke: '#FFFFFF',
            anchorStrokeWidth: 2,
            anchorSize: 8,
        });
        uiLayer.add(transformer);

        // Setup mouse wheel zoom
        setupZoomControls();

        // Setup stage event listeners
        setupStageEvents();

        console.log('Enhanced Konva stage created successfully');

        // Load and display image
        loadImageWithAnnotations(image);
    }
    
    // Setup zoom controls (mouse wheel)
    function setupZoomControls() {
        if (!stage) return;

        // Mouse wheel zoom
        stage.on('wheel', (e) => {
            e.evt.preventDefault();

            const pointer = stage.getPointerPosition();
            if (!pointer) return;

            const oldScale = stage.scaleX();
            const newScale = e.evt.deltaY > 0 ? oldScale * 0.9 : oldScale * 1.1;

            // Limit zoom levels
            const clampedScale = Math.max(0.1, Math.min(5, newScale));

            const mousePointTo = {
                x: (pointer.x - stage.x()) / oldScale,
                y: (pointer.y - stage.y()) / oldScale,
            };

            stage.scale({ x: clampedScale, y: clampedScale });

            const newPos = {
                x: pointer.x - mousePointTo.x * clampedScale,
                y: pointer.y - mousePointTo.y * clampedScale,
            };

            stage.position(newPos);
            stage.batchDraw();

            // Update global state
            scale = clampedScale;
            stageX = newPos.x;
            stageY = newPos.y;
        });
    }

    // Setup stage event listeners
    function setupStageEvents() {
        if (!stage) return;

        // Click to select/deselect annotations
        stage.on('click tap', (e) => {
            if (e.target === stage) {
                deselectAnnotation();
            } else if (e.target.getParent() && annotationGroups.includes(e.target.getParent())) {
                selectAnnotation(e.target.getParent());
            }
        });

        // Prevent default drag behavior when clicking on annotations
        stage.on('dragstart', (e) => {
            if (annotationGroups.includes(e.target.getParent())) {
                e.cancelBubble = true;
            }
        });
    }

    // Zoom functions
    function zoomIn() {
        if (!stage) return;
        const newScale = Math.min(5, stage.scaleX() * 1.2);
        zoomToScale(newScale);
    }

    function zoomOut() {
        if (!stage) return;
        const newScale = Math.max(0.1, stage.scaleX() * 0.8);
        zoomToScale(newScale);
    }

    function resetZoom() {
        zoomToScale(1);
        stage.position({ x: 0, y: 0 });
        stageX = 0;
        stageY = 0;
    }

    function fitToScreen() {
        if (!stage || !imageNode) return;

        const containerRect = konvaContainer.getBoundingClientRect();
        const imageRect = imageNode.getClientRect();

        const scaleX = containerRect.width / imageRect.width;
        const scaleY = containerRect.height / imageRect.height;
        const newScale = Math.min(scaleX, scaleY, 1);

        zoomToScale(newScale);
        stage.position({ x: 0, y: 0 });
        stageX = 0;
        stageY = 0;
    }

    function zoomToScale(newScale: number) {
        if (!stage) return;

        const center = {
            x: stage.width() / 2,
            y: stage.height() / 2,
        };

        const relatedTo = {
            x: (center.x - stage.x()) / stage.scaleX(),
            y: (center.y - stage.y()) / stage.scaleY(),
        };

        stage.scale({ x: newScale, y: newScale });

        const newPos = {
            x: center.x - relatedTo.x * newScale,
            y: center.y - relatedTo.y * newScale,
        };

        stage.position(newPos);
        stage.batchDraw();

        scale = newScale;
        stageX = newPos.x;
        stageY = newPos.y;
    }

    // Load image and draw annotations
    async function loadImageWithAnnotations(image: any) {
        console.log('loadImageWithAnnotations called with:', image);
        console.log('Stage:', stage, 'MainLayer:', mainLayer);

        if (!stage || !mainLayer) {
            console.error('Stage or mainLayer not available');
            return;
        }
        
        const imageObj = new Image();
        imageObj.crossOrigin = 'anonymous';
        
        imageObj.onload = async () => {
            console.log('Image loaded successfully:', imageObj.width, 'x', imageObj.height);

            // Calculate scaling to fit the stage
            const scaleX = stage!.width() / imageObj.width;
            const scaleY = stage!.height() / imageObj.height;
            const fitScale = Math.min(scaleX, scaleY, 1); // Don't scale up

            const scaledWidth = imageObj.width * fitScale;
            const scaledHeight = imageObj.height * fitScale;

            console.log('Calculated scale:', fitScale, 'Scaled dimensions:', scaledWidth, 'x', scaledHeight);

            // Center the image
            const x = (stage!.width() - scaledWidth) / 2;
            const y = (stage!.height() - scaledHeight) / 2;

            console.log('Image position:', x, y);

            // Create Konva image
            imageNode = new Konva.Image({
                x: x,
                y: y,
                image: imageObj,
                width: scaledWidth,
                height: scaledHeight,
            });

            console.log('Created Konva image node');
            mainLayer!.add(imageNode);
            mainLayer!.draw();

            // Load and draw annotations if available
            await loadAnnotations(image, fitScale, x, y);

            console.log('Drawing layers');
            stage!.batchDraw();
        };
        
        imageObj.onerror = (error) => {
            console.error('Failed to load image:', error);
            console.error('Image src was:', imageObj.src);
        };
        
        // Load the original image from the path provided by backend
        const originalImageUrl = convertFileSrc(image.path);

        console.log('Loading image from path:', image.path);
        console.log('Converted URL:', originalImageUrl);

        imageObj.src = originalImageUrl;
    }
    
    // Annotation selection and editing functions
    function selectAnnotation(annotationGroup: any) {
        if (!transformer || !stage) return;

        selectedAnnotation = annotationGroup;
        transformer.nodes([annotationGroup]);
        uiLayer!.draw();

        console.log('Selected annotation:', annotationGroup);
    }

    function deselectAnnotation() {
        if (!transformer) return;

        selectedAnnotation = null;
        transformer.nodes([]);
        uiLayer!.draw();

        console.log('Deselected annotation');
    }

    function selectAllAnnotations() {
        if (!transformer || !stage) return;

        if (annotationGroups.length > 0) {
            selectedAnnotation = annotationGroups;
            transformer.nodes(annotationGroups);
            uiLayer!.draw();
        }
    }

    function deleteSelectedAnnotation() {
        if (!selectedAnnotation || !annotationLayer) return;

        if (Array.isArray(selectedAnnotation)) {
            selectedAnnotation.forEach(group => {
                group.destroy();
            });
            annotationGroups = annotationGroups.filter(group =>
                !selectedAnnotation.includes(group)
            );
        } else {
            selectedAnnotation.destroy();
            annotationGroups = annotationGroups.filter(group => group !== selectedAnnotation);
        }

        deselectAnnotation();
        annotationLayer.draw();

        console.log('Deleted selected annotation(s)');
    }

    // Load annotations from the image data (already available from generate_annotated_previews)
    async function loadAnnotations(image: any, scale: number, offsetX: number, offsetY: number) {
        console.log('loadAnnotations called for:', image.name);
        console.log('Scale:', scale, 'Offset:', offsetX, offsetY);

        if (!annotationLayer) {
            console.error('Annotation layer not available');
            return;
        }

        try {
            // Use annotation data that's already available in the image object
            if (image.annotations && image.annotations.length > 0) {
                console.log('Drawing', image.annotations.length, 'annotations from cached data');
                drawAnnotationsOnCanvas(image.annotations, scale, offsetX, offsetY);
            } else {
                console.log('No annotations found for this image');
            }
        } catch (error) {
            console.error('Error loading annotations:', error);
        }
    }
    
    // Helper function to draw annotations on the Konva canvas
    function drawAnnotationsOnCanvas(annotations: any[], scale: number, offsetX: number, offsetY: number) {
        if (!annotationLayer || !uiLayer) return;

        // Clear existing annotations
        annotationLayer.destroyChildren();
        annotationGroups = [];

        annotations.forEach((annotation: any, index: number) => {
            console.log(`Drawing annotation ${index}:`, annotation);
            const annotationGroup = drawAnnotationShape(annotation, scale, offsetX, offsetY, index);
            if (annotationGroup) {
                annotationGroups.push(annotationGroup);
            }
        });

        annotationLayer.draw();
        uiLayer.draw();
    }
    
    // Draw individual annotation shape with advanced features
    function drawAnnotationShape(annotation: any, scale: number, offsetX: number, offsetY: number, index: number) {
        console.log('Drawing annotation:', annotation.shape_type, 'with', annotation.points?.length, 'points');

        if (!annotationLayer || !uiLayer) {
            console.error('Layers not available for drawing');
            return null;
        }

        // Enhanced color palette with better contrast
        const colors = [
            { fill: '#FF6B6B', stroke: '#E53E3E', text: '#FFFFFF' }, // Red
            { fill: '#4ECDC4', stroke: '#319795', text: '#FFFFFF' }, // Teal
            { fill: '#45B7D1', stroke: '#3182CE', text: '#FFFFFF' }, // Blue
            { fill: '#96CEB4', stroke: '#38A169', text: '#FFFFFF' }, // Green
            { fill: '#FFEAA7', stroke: '#D69E2E', text: '#000000' }, // Yellow
            { fill: '#DDA0DD', stroke: '#9F7AEA', text: '#FFFFFF' }, // Purple
            { fill: '#98D8C8', stroke: '#4FD1C9', text: '#000000' }, // Cyan
        ];
        const colorScheme = colors[index % colors.length];

        try {
            const annotationGroup = new Konva.Group({
                draggable: true,
                name: `annotation-${index}`,
            });

            let shape;
            let bounds;

            if (annotation.shape_type === 'rectangle' && annotation.points && annotation.points.length >= 2) {
                const x1 = annotation.points[0][0] * scale + offsetX;
                const y1 = annotation.points[0][1] * scale + offsetY;
                const x2 = annotation.points[1][0] * scale + offsetX;
                const y2 = annotation.points[1][1] * scale + offsetY;

                bounds = {
                    x: Math.min(x1, x2),
                    y: Math.min(y1, y2),
                    width: Math.abs(x2 - x1),
                    height: Math.abs(y2 - y1)
                };

                shape = new Konva.Rect({
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                    fill: colorScheme.fill,
                    stroke: colorScheme.stroke,
                    strokeWidth: 2,
                    opacity: 0.25,
                    shadowColor: colorScheme.stroke,
                    shadowBlur: 4,
                    shadowOpacity: 0.3,
                });

            } else if (annotation.shape_type === 'bounding_box' && annotation.points && annotation.points.length >= 2) {
                const x1 = annotation.points[0][0] * scale + offsetX;
                const y1 = annotation.points[0][1] * scale + offsetY;
                const x2 = annotation.points[1][0] * scale + offsetX;
                const y2 = annotation.points[1][1] * scale + offsetY;

                bounds = {
                    x: Math.min(x1, x2),
                    y: Math.min(y1, y2),
                    width: Math.abs(x2 - x1),
                    height: Math.abs(y2 - y1)
                };

                shape = new Konva.Rect({
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                    fill: colorScheme.fill,
                    stroke: colorScheme.stroke,
                    strokeWidth: 2,
                    opacity: 0.25,
                    shadowColor: colorScheme.stroke,
                    shadowBlur: 4,
                    shadowOpacity: 0.3,
                });

            } else if (annotation.shape_type === 'polygon' && annotation.points && annotation.points.length > 2) {
                const points: number[] = [];
                annotation.points.forEach((point: number[]) => {
                    points.push(point[0] * scale + offsetX);
                    points.push(point[1] * scale + offsetY);
                });

                // Calculate bounds for polygon
                const xs = annotation.points.map(p => p[0] * scale + offsetX);
                const ys = annotation.points.map(p => p[1] * scale + offsetY);
                bounds = {
                    x: Math.min(...xs),
                    y: Math.min(...ys),
                    width: Math.max(...xs) - Math.min(...xs),
                    height: Math.max(...ys) - Math.min(...ys)
                };

                shape = new Konva.Line({
                    points: points,
                    fill: colorScheme.fill,
                    stroke: colorScheme.stroke,
                    strokeWidth: 2,
                    opacity: 0.25,
                    closed: true,
                    shadowColor: colorScheme.stroke,
                    shadowBlur: 4,
                    shadowOpacity: 0.3,
                });
            }

            if (shape && bounds) {
                annotationGroup.add(shape);

                // Enhanced label with background
                if (annotation.label) {
                    const labelBg = new Konva.Rect({
                        x: bounds.x,
                        y: bounds.y - 25,
                        width: annotation.label.length * 8 + 10,
                        height: 20,
                        fill: colorScheme.stroke,
                        opacity: 0.8,
                        cornerRadius: 4,
                    });

                    const label = new Konva.Text({
                        x: bounds.x + 5,
                        y: bounds.y - 23,
                        text: annotation.label,
                        fontSize: 12,
                        fontFamily: 'Arial',
                        fontStyle: 'bold',
                        fill: colorScheme.text,
                    });

                    annotationGroup.add(labelBg);
                    annotationGroup.add(label);
                }

                // Add tooltip on hover
                const tooltip = new Konva.Text({
                    text: `${annotation.label || 'Unknown'}\nType: ${annotation.shape_type}\nConfidence: ${annotation.confidence || 'N/A'}`,
                    fontSize: 12,
                    fontFamily: 'Arial',
                    fill: '#000000',
                    padding: 8,
                    visible: false,
                    opacity: 0.9,
                });

                const tooltipBg = new Konva.Rect({
                    fill: '#FFFFFF',
                    stroke: '#CCCCCC',
                    strokeWidth: 1,
                    cornerRadius: 4,
                    shadowColor: '#000000',
                    shadowBlur: 8,
                    shadowOpacity: 0.2,
                });

                uiLayer.add(tooltipBg);
                uiLayer.add(tooltip);

                // Hover effects
                annotationGroup.on('mouseenter', () => {
                    document.body.style.cursor = 'pointer';
                    shape.opacity(0.4);
                    annotationLayer.draw();

                    // Show tooltip
                    const mousePos = stage!.getPointerPosition();
                    if (mousePos) {
                        tooltip.position({
                            x: mousePos.x + 10,
                            y: mousePos.y - 10,
                        });
                        tooltipBg.position({
                            x: mousePos.x + 5,
                            y: mousePos.y - 15,
                        });
                        tooltipBg.width(tooltip.width() + 16);
                        tooltipBg.height(tooltip.height() + 16);
                        tooltip.visible(true);
                        tooltipBg.visible(true);
                        uiLayer.draw();
                    }
                });

                annotationGroup.on('mouseleave', () => {
                    document.body.style.cursor = 'default';
                    shape.opacity(0.25);
                    annotationLayer.draw();

                    // Hide tooltip
                    tooltip.visible(false);
                    tooltipBg.visible(false);
                    uiLayer.draw();
                });

                // Click to select
                annotationGroup.on('click tap', (e) => {
                    e.cancelBubble = true;
                    selectAnnotation(annotationGroup);
                });

                annotationLayer.add(annotationGroup);
                return annotationGroup;
            }

        } catch (error) {
            console.error('Error drawing annotation:', error, annotation);
        }

        return null;
    }

    // Function to handle window resize for modal
    function handleResize() {
        // Handle modal responsiveness if needed
    }

    function validateSelection(): string | null {
        if (!selectedParentLabel) return "Please select a parent label";
        if (selectedChildLabels.length === 0) return "Please select at least one child label";
        
        const parentCount = datasetSummary?.label_counts[selectedParentLabel] || 0;
        if (parentCount === 0) return `No instances of '${selectedParentLabel}' found in dataset`;
        
        return null;
    }

    function getFilteredChildLabels(): string[] {
        return availableLabels.filter(label => label !== selectedParentLabel);
    }

    async function runProcessing() {
        if (!sourceDir || !outputDir) {
            errorMessage = "Please select source directory and output directory.";
            return;
        }

        if (!datasetLoaded) {
            errorMessage = "Dataset is still being analyzed. Please wait for analysis to complete.";
            return;
        }

        const validationError = validateSelection();
        if (validationError) {
            errorMessage = validationError;
            return;
        }

        loading = true;
        successMessage = null;
        errorMessage = null;

        try {
            const message = await invoke("crop_and_remap_annotations", {
                sourceDir: sourceDir,
                outputDir: outputDir,
                parentLabel: selectedParentLabel,
                requiredChildLabelsStr: selectedChildLabels.join(","),
                paddingFactor: paddingFactor
            });
            successMessage = String(message);
        } catch (err) {
            console.error("Error running processing:", err);
            errorMessage = `Processing failed: ${err instanceof Error ? err.message : String(err)}`;
        } finally {
            loading = false;
        }
    }

</script>

<svelte:head>
    <title>Crop & Remap Annotations</title>
    <meta name="description" content="Advanced crop tool with dynamic multi-label safety equipment detection." />
</svelte:head>

<svelte:window on:keydown={handleKeydown} on:resize={handleResize} />

<div class="container mx-auto px-4 py-8">
    <div class="max-w-2xl mx-auto bg-white p-6 rounded-lg shadow-md">
        <h1 class="text-2xl font-bold text-gray-800 mb-6">Crop and Remap Tool</h1>
        <p class="text-sm text-gray-600 mb-6">Advanced annotation processing with dynamic multi-label detection based on your dataset</p>

        <div class="space-y-4 mb-6">
            <!-- Source Directory -->
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Source Directory</label>
                <div class="flex items-center gap-2">
                    <input 
                        type="text" 
                        readonly 
                        placeholder="Select source directory..." 
                        value={sourceDir || ''}
                        class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-600 text-sm truncate"
                    />
                    <button 
                        on:click={() => selectDirectory('source')}
                        class="px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md border border-gray-300 text-sm"
                    >
                        Browse...
                    </button>
                </div>
            </div>


            <!-- Dataset Summary -->
            {#if datasetSummary}
                <div class="bg-blue-50 p-3 rounded-md border border-blue-200">
                    <h4 class="font-medium text-blue-900 mb-1">Dataset Summary</h4>
                    <div class="text-sm text-blue-700 space-y-1">
                        <p>📁 {datasetSummary.total_images} images</p>
                        <p>🏷️ {datasetSummary.total_annotations} annotations</p>
                        <p>🎯 {datasetSummary.unique_labels} unique labels</p>
                    </div>
                </div>
            {/if}

            <!-- Auto Preview Section -->
            {#if datasetLoaded}
                <div class="bg-purple-50 p-4 rounded-md border border-purple-200">
                    <h4 class="font-medium text-purple-900 mb-3 flex items-center">
                        <span class="mr-2">👁️</span>
                        Dataset Preview
                        {#if previewLoading}
                            <div class="ml-2 animate-spin h-4 w-4 border-2 border-purple-600 border-t-transparent rounded-full"></div>
                        {/if}
                    </h4>

                    {#if previewLoading}
                        <div class="text-sm text-purple-700">Loading preview images...</div>
                    {:else if previewImages.length > 0}
                        <div class="text-sm text-purple-700 mb-3">
                            Here are {previewImages.length} random annotated images from your dataset with annotations already drawn on them:
                        </div>
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-3">
                            {#each previewImages as image, index (image.path)}
                                <div class="bg-white rounded-lg shadow-sm overflow-hidden border border-purple-200">
                                    <div class="relative pb-[75%]">
                                        <button
                                            type="button"
                                            class="absolute inset-0 w-full h-full focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2"
                                            on:click={() => openPreviewModal(image)}
                                            aria-label={`View full size image: ${image.name}`}
                                        >
                                            <img
                                                src={image.previewUrl}
                                                alt={image.name}
                                                class="w-full h-full object-cover hover:opacity-90 transition-opacity"
                                            />
                                        </button>
                                        <div class="absolute top-1 right-1 bg-purple-600 text-white text-xs px-2 py-0.5 rounded-full">
                                            Preview
                                        </div>
                                    </div>
                                    <div class="p-2">
                                        <p class="text-xs text-gray-600 truncate" title={image.name}>
                                            {image.name}
                                        </p>
                                    </div>
                                </div>
                            {/each}
                        </div>
                        <div class="text-xs text-purple-600 mt-2">
                            Click any image to view full size (annotations are already drawn on the images)
                        </div>
                    {:else}
                        <div class="text-sm text-purple-700">No annotated images found for preview</div>
                    {/if}
                </div>
            {/if}

            <!-- Output Directory -->
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Output Directory</label>
                <div class="flex items-center gap-2">
                    <input 
                        type="text" 
                        readonly 
                        placeholder="Select output directory..." 
                        value={outputDir || ''}
                        class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-600 text-sm truncate"
                    />
                    <button 
                        on:click={() => selectDirectory('output')}
                        class="px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md border border-gray-300 text-sm"
                    >
                        Browse...
                    </button>
                </div>
            </div>

            <!-- Dynamic Parent Label Selection -->
            {#if datasetLoaded && availableLabels.length > 0}
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">Parent Label</label>
                    <select 
                        bind:value={selectedParentLabel}
                        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500 text-sm"
                    >
                        {#each availableLabels as label}
                            <option value={label}>
                                {label} ({datasetSummary?.label_counts[label] || 0} annotations)
                            </option>
                        {/each}
                    </select>
                    <p class="text-xs text-gray-500 mt-1">The label of the object to crop around.</p>
                </div>

                <!-- Dynamic Child Labels Selection -->
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">Required Child Labels</label>
                    <div class="max-h-48 overflow-y-auto space-y-2 p-3 bg-gray-50 rounded-md border">
                        {#each getFilteredChildLabels() as label}
                            <label class="flex items-center">
                                <input 
                                    type="checkbox" 
                                    bind:group={selectedChildLabels}
                                    value={label}
                                    class="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                                />
                                <span class="ml-2 text-sm text-gray-700">
                                    {label} ({datasetSummary?.label_counts[label] || 0} annotations)
                                </span>
                            </label>
                        {/each}
                    </div>
                    <p class="text-xs text-gray-500 mt-1">
                        Only people wearing at least one of the selected items will be processed.
                        {#if selectedChildLabels.length > 0}
                            <br><strong>Selected:</strong> {selectedChildLabels.join(", ")}
                        {:else}
                            <br><strong>Selected:</strong> None
                        {/if}
                    </p>
                </div>
            {:else if !datasetLoaded}
                <div class="bg-yellow-50 p-3 rounded-md border border-yellow-200">
                    <p class="text-sm text-yellow-800">
                        📋 Please analyze your dataset first to see available labels and configure processing options.
                    </p>
                </div>
            {/if}

            <!-- Padding Factor -->
            {#if datasetLoaded}
                <div>
                    <label for="paddingFactor" class="block text-sm font-medium text-gray-700 mb-1">
                        Padding Factor
                        <span class="text-indigo-600 font-medium">({((paddingFactor - 1) * 100).toFixed(0)}% {paddingFactor > 1 ? 'larger' : paddingFactor < 1 ? 'smaller' : 'original'})</span>
                    </label>
                    <div class="flex items-center gap-3">
                        <input 
                            type="range" 
                            id="paddingFactor"
                            bind:value={paddingFactor}
                            min="0.5"
                            max="2.0"
                            step="0.1"
                            class="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
                        />
                        <input 
                            type="number" 
                            bind:value={paddingFactor}
                            min="0.5"
                            max="2.0"
                            step="0.1"
                            class="w-20 px-2 py-1 border border-gray-300 rounded text-sm text-center"
                        />
                    </div>
                    <div class="flex justify-between text-xs text-gray-500 mt-1">
                        <span>0.5x (50% smaller)</span>
                        <span>1.0x (original size)</span>
                        <span>2.0x (100% larger)</span>
                    </div>
                    <p class="text-xs text-gray-500 mt-1">
                        Controls how much larger the crop area should be compared to the parent bounding box. 
                        Larger values provide more context but may include unwanted background.
                    </p>
                </div>
            {/if}
        </div>

        <!-- Run Button -->
        <div class="mb-6">
            <button 
                on:click={runProcessing}
                disabled={loading || !sourceDir || !outputDir || !datasetLoaded || !selectedParentLabel || selectedChildLabels.length === 0}
                class="w-full inline-flex justify-center items-center px-6 py-3 border border-transparent text-base font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
            >
                {#if loading}
                    <div class="mr-2 animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"></div>
                    Processing...
                {:else}
                    Run Crop & Remap
                {/if}
            </button>
            
            <!-- Validation Warning -->
            {#if datasetLoaded}
                {#if validateSelection()}
                    <p class="text-sm text-red-600 mt-2">⚠️ {validateSelection()}</p>
                {/if}
            {/if}
        </div>

        <!-- Status Messages -->
        <div class="space-y-3">
            {#if successMessage}
                <div class="bg-green-50 text-green-700 p-3 rounded-md text-sm">
                    <p class="font-medium">Success!</p>
                    <p>{successMessage}</p>
                </div>
            {/if}
            {#if errorMessage}
                <div class="bg-red-50 text-red-700 p-3 rounded-md text-sm">
                    <p class="font-medium">Error!</p>
                    <p>{errorMessage}</p>
                </div>
            {/if}
        </div>

    </div>
</div>

<!-- Preview Modal -->
{#if showPreviewModal && previewModalImage}
    <div
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
        role="dialog"
        aria-modal="true"
        aria-labelledby="preview-modal-title"
        on:click={closePreviewModal}
        on:keydown={(e) => {
            if (e.key === 'Escape') closePreviewModal();
        }}
        tabindex="-1"
    >
        <div
            class="bg-white rounded-lg shadow-xl max-w-6xl max-h-[95vh] overflow-hidden"
            role="document"
            on:click|stopPropagation
        >
            <div class="p-4 border-b border-gray-200 flex justify-between items-center">
                <h3 id="preview-modal-title" class="text-lg font-medium text-gray-900">{previewModalImage.name}</h3>
                <button
                    on:click={closePreviewModal}
                    class="text-gray-400 hover:text-gray-600 text-2xl leading-none"
                >
                    ×
                </button>
            </div>
            <div class="p-4">
                <!-- Control Panel -->
                <div class="flex flex-wrap items-center gap-2 mb-4 p-3 bg-gray-50 rounded-lg">
                    <div class="text-sm text-gray-600 mr-4">Controls:</div>

                    <!-- Zoom Controls -->
                    <button
                        on:click={zoomOut}
                        class="px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white text-sm rounded transition-colors"
                        title="Zoom Out (-)"
                    >
                        🔍-
                    </button>
                    <button
                        on:click={resetZoom}
                        class="px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white text-sm rounded transition-colors"
                        title="Reset Zoom (0)"
                    >
                        100%
                    </button>
                    <button
                        on:click={zoomIn}
                        class="px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white text-sm rounded transition-colors"
                        title="Zoom In (=)"
                    >
                        🔍+
                    </button>

                    <!-- Fit to Screen -->
                    <button
                        on:click={fitToScreen}
                        class="px-3 py-1 bg-green-500 hover:bg-green-600 text-white text-sm rounded transition-colors ml-4"
                        title="Fit to Screen (R)"
                    >
                        📐 Fit
                    </button>

                    <!-- Annotation Controls -->
                    <div class="ml-4 flex items-center gap-2">
                        <span class="text-sm text-gray-600">Annotations:</span>
                        <button
                            on:click={selectAllAnnotations}
                            class="px-3 py-1 bg-purple-500 hover:bg-purple-600 text-white text-sm rounded transition-colors"
                            title="Select All (Ctrl+A)"
                        >
                            Select All
                        </button>
                        <button
                            on:click={deselectAnnotation}
                            class="px-3 py-1 bg-gray-500 hover:bg-gray-600 text-white text-sm rounded transition-colors"
                            title="Deselect (Esc)"
                        >
                            Deselect
                        </button>
                        <button
                            on:click={deleteSelectedAnnotation}
                            class="px-3 py-1 bg-red-500 hover:bg-red-600 text-white text-sm rounded transition-colors"
                            title="Delete Selected (Del)"
                        >
                            🗑️ Delete
                        </button>
                    </div>

                    <!-- Status -->
                    <div class="ml-auto text-sm text-gray-600">
                        Zoom: {Math.round(scale * 100)}% |
                        Annotations: {annotationGroups.length}
                        {#if selectedAnnotation}
                            | Selected: {Array.isArray(selectedAnnotation) ? selectedAnnotation.length : 1}
                        {/if}
                    </div>
                </div>

                <!-- Keyboard Shortcuts Info -->
                <div class="text-xs text-gray-500 mb-4 bg-blue-50 p-2 rounded">
                    <strong>Keyboard Shortcuts:</strong> Zoom (+/-), Reset (0), Fit (R), Select All (Ctrl+A), Delete (Del), Deselect (Esc), Close (Esc)
                </div>

                <div class="relative max-w-full max-h-[70vh] overflow-hidden rounded-lg bg-gray-100">
                    <!-- KonvaJS Container -->
                    <div
                        bind:this={konvaContainer}
                        class="w-full h-full min-h-[600px] border-2 border-gray-300 bg-gray-50 flex items-center justify-center"
                        style="width: 1000px; height: 700px;"
                    >
                        <div class="text-gray-500">Loading advanced canvas...</div>
                    </div>
                    <!-- Status Indicators -->
                    <div class="absolute top-2 left-2 bg-green-600 text-white text-xs px-2 py-1 rounded shadow z-10">
                        Enhanced KonvaJS
                    </div>
                    <div class="absolute bottom-2 right-2 bg-blue-600 text-white text-xs px-2 py-1 rounded shadow z-10">
                        {annotationGroups.length} annotations loaded
                    </div>
                </div>
                <div class="mt-4 text-sm text-gray-600 text-center">
                    Advanced interactive annotations with zoom, pan, selection, and editing capabilities
                </div>
            </div>
        </div>
    </div>
{/if}
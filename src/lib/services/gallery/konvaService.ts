// KonvaJS Service for Advanced Annotation Viewer
// Handles all KonvaJS-related functionality in a centralized way

import Konva from 'konva';
import { safeConvertFileSrc } from '$lib/utils/tauriUtils';

export interface KonvaAnnotation {
    label?: string;
    shape_type: 'rectangle' | 'bounding_box' | 'polygon';
    points: number[][];
    confidence?: number;
}

export interface KonvaImageData {
    id: string;
    path: string;
    previewUrl: string;
    name: string;
    annotations: KonvaAnnotation[];
}

export interface KonvaState {
    stage: Konva.Stage | null;
    mainLayer: Konva.Layer | null;
    annotationLayer: Konva.Layer | null;
    uiLayer: Konva.Layer | null;
    imageNode: Konva.Image | null;
    transformer: Konva.Transformer | null;
    scale: number;
    stageX: number;
    stageY: number;
    selectedAnnotation: Konva.Group | Konva.Group[] | null;
    annotationGroups: Konva.Group[];
    isDragging: boolean;
    lastPointerPosition: { x: number; y: number };
}

export class KonvaManager {
    private state: KonvaState;
    private container: HTMLDivElement | null = null;

    constructor() {
        this.state = this.getInitialState();
    }

    private getInitialState(): KonvaState {
        return {
            stage: null,
            mainLayer: null,
            annotationLayer: null,
            uiLayer: null,
            imageNode: null,
            transformer: null,
            scale: 1,
            stageX: 0,
            stageY: 0,
            selectedAnnotation: null,
            annotationGroups: [],
            isDragging: false,
            lastPointerPosition: { x: 0, y: 0 }
        };
    }

    // Initialize Konva stage with enhanced configuration
    initializeStage(container: HTMLDivElement, width: number = 1000, height: number = 700): void {
        this.container = container;
        this.cleanup(); // Clean up any existing stage

        console.log('Initializing Konva stage with dimensions:', width, 'x', height);

        this.state.stage = new Konva.Stage({
            container: container,
            width: width,
            height: height,
            x: this.state.stageX,
            y: this.state.stageY,
            scaleX: this.state.scale,
            scaleY: this.state.scale,
            draggable: true,
        });

        // Create layers
        this.state.mainLayer = new Konva.Layer();
        this.state.annotationLayer = new Konva.Layer();
        this.state.uiLayer = new Konva.Layer();

        this.state.stage.add(this.state.mainLayer);
        this.state.stage.add(this.state.annotationLayer);
        this.state.stage.add(this.state.uiLayer);

        // Create transformer for editing
        this.state.transformer = new Konva.Transformer({
            rotateEnabled: true,
            borderEnabled: true,
            borderStroke: '#4A90E2',
            borderStrokeWidth: 2,
            anchorFill: '#4A90E2',
            anchorStroke: '#FFFFFF',
            anchorStrokeWidth: 2,
            anchorSize: 8,
        });

        this.state.uiLayer.add(this.state.transformer);

        // Setup event handlers
        this.setupZoomControls();
        this.setupStageEvents();

        console.log('Konva stage initialized successfully');
    }

    // Setup mouse wheel zoom controls
    private setupZoomControls(): void {
        if (!this.state.stage) return;

        this.state.stage.on('wheel', (e) => {
            e.evt.preventDefault();

            const pointer = this.state.stage!.getPointerPosition();
            if (!pointer) return;

            const oldScale = this.state.stage!.scaleX();
            const newScale = e.evt.deltaY > 0 ? oldScale * 0.9 : oldScale * 1.1;
            const clampedScale = Math.max(0.1, Math.min(5, newScale));

            const mousePointTo = {
                x: (pointer.x - this.state.stage!.x()) / oldScale,
                y: (pointer.y - this.state.stage!.y()) / oldScale,
            };

            this.state.stage!.scale({ x: clampedScale, y: clampedScale });

            const newPos = {
                x: pointer.x - mousePointTo.x * clampedScale,
                y: pointer.y - mousePointTo.y * clampedScale,
            };

            this.state.stage!.position(newPos);
            this.state.stage!.batchDraw();

            this.state.scale = clampedScale;
            this.state.stageX = newPos.x;
            this.state.stageY = newPos.y;
        });
    }

    // Setup stage event handlers
    private setupStageEvents(): void {
        if (!this.state.stage) return;

        // Click to select/deselect annotations
        this.state.stage.on('click tap', (e) => {
            if (e.target === this.state.stage) {
                this.deselectAnnotation();
            } else if (e.target.getParent() && e.target.getParent() instanceof Konva.Group) {
                const parentGroup = e.target.getParent() as Konva.Group;
                if (this.state.annotationGroups.includes(parentGroup)) {
                    this.selectAnnotation(parentGroup);
                }
            }
        });

        // Prevent default drag behavior when clicking on annotations
        this.state.stage.on('dragstart', (e) => {
            if (e.target.getParent() && e.target.getParent() instanceof Konva.Group) {
                const parentGroup = e.target.getParent() as Konva.Group;
                if (this.state.annotationGroups.includes(parentGroup)) {
                    e.cancelBubble = true;
                }
            }
        });
    }

    // Load and display image with annotations (optimized progressive loading)
    async loadImageWithAnnotations(imageData: KonvaImageData, callback?: (scale: number, offsetX: number, offsetY: number) => void): Promise<void> {
        if (!this.state.stage || !this.state.mainLayer) {
            console.error('Stage or mainLayer not available');
            return;
        }

        return new Promise((resolve, reject) => {
            console.log('Starting progressive image loading for:', imageData.name);

            // Step 1: Load low-resolution preview first (if available)
            const loadLowResPreview = () => {
                // For now, we'll load the main image directly
                // In a more advanced implementation, you could load a separate thumbnail first
                loadFullImage();
            };

            // Step 2: Load full-resolution image
            const loadFullImage = () => {
                const imageObj = new Image();
                imageObj.crossOrigin = 'anonymous';

                // Add loading progress tracking
                imageObj.onloadstart = () => {
                    console.log('Image loading started...');
                };

                imageObj.onprogress = (event) => {
                    if (event.lengthComputable) {
                        const percentComplete = (event.loaded / event.total) * 100;
                        console.log(`Image loading: ${Math.round(percentComplete)}%`);
                    }
                };

                imageObj.onload = () => {
                    console.log('✅ Image loaded successfully:', imageObj.width, 'x', imageObj.height);

                    // Calculate optimal scaling
                    const scaleX = this.state.stage!.width() / imageObj.width;
                    const scaleY = this.state.stage!.height() / imageObj.height;
                    const fitScale = Math.min(scaleX, scaleY, 1);

                    const scaledWidth = imageObj.width * fitScale;
                    const scaledHeight = imageObj.height * fitScale;

                    // Center the image
                    const x = (this.state.stage!.width() - scaledWidth) / 2;
                    const y = (this.state.stage!.height() - scaledHeight) / 2;

                    // Remove existing image if any
                    if (this.state.imageNode) {
                        this.state.imageNode.destroy();
                    }

                    // Create optimized Konva image
                    this.state.imageNode = new Konva.Image({
                        x: x,
                        y: y,
                        image: imageObj,
                        width: scaledWidth,
                        height: scaledHeight,
                        // Performance optimizations
                        listening: false, // Don't listen for events on image
                        perfectDrawEnabled: false, // Disable perfect draw for better performance
                    });

                    this.state.mainLayer!.add(this.state.imageNode);
                    this.state.mainLayer!.batchDraw();

                    // Step 3: Load annotations progressively
                    if (callback) {
                        // Use setTimeout to allow UI to update first
                        setTimeout(() => {
                            this.loadAnnotationsProgressively(imageData.annotations, fitScale, x, y, callback);
                        }, 50);
                    }

                    resolve();
                };

                imageObj.onerror = (error) => {
                    console.error('❌ Failed to load image:', error);
                    reject(error);
                };

                console.log('🔄 Loading full-resolution image...');
                imageObj.src = imageData.previewUrl;
            };

            // Start the loading process
            loadLowResPreview();
        });
    }

    // Progressive annotation loading for better performance
    private loadAnnotationsProgressively(
        annotations: KonvaAnnotation[],
        scale: number,
        offsetX: number,
        offsetY: number,
        finalCallback: (scale: number, offsetX: number, offsetY: number) => void
    ): void {
        if (!annotations || annotations.length === 0) {
            finalCallback(scale, offsetX, offsetY);
            return;
        }

        console.log(`🔄 Loading ${annotations.length} annotations progressively...`);

        // Process annotations in chunks for better performance
        const chunkSize = 50; // Process 50 annotations at a time
        let processed = 0;

        const processChunk = () => {
            const chunk = annotations.slice(processed, processed + chunkSize);
            console.log(`Processing annotation chunk: ${processed + 1}-${processed + chunk.length}`);

            chunk.forEach((annotation, index) => {
                const globalIndex = processed + index;
                const annotationGroup = this.drawAnnotationShape(annotation, scale, offsetX, offsetY, globalIndex);
                if (annotationGroup) {
                    this.state.annotationGroups.push(annotationGroup);
                }
            });

            processed += chunk.length;

            // Continue with next chunk or finish
            if (processed < annotations.length) {
                // Use setTimeout to allow UI updates between chunks
                setTimeout(processChunk, 10);
            } else {
                // All annotations processed
                console.log(`✅ All ${annotations.length} annotations loaded`);
                this.state.annotationLayer!.batchDraw();
                this.state.uiLayer!.batchDraw();

                // Call the final callback
                finalCallback(scale, offsetX, offsetY);
            }
        };

        // Start processing the first chunk
        processChunk();
    }

    // Draw annotations on canvas (optimized for progressive loading)
    drawAnnotations(annotations: KonvaAnnotation[], scale: number, offsetX: number, offsetY: number): void {
        if (!this.state.annotationLayer || !this.state.uiLayer) return;

        console.log(`🎨 Drawing ${annotations.length} annotations on canvas`);

        // Clear existing annotations only if this is a full redraw
        // For progressive loading, annotations are added incrementally
        if (this.state.annotationGroups.length === 0) {
            this.state.annotationLayer.destroyChildren();
            this.state.annotationGroups = [];
        }

        // Process annotations efficiently
        annotations.forEach((annotation, index) => {
            // Skip if annotation already exists (prevent duplicates in progressive loading)
            const exists = this.state.annotationGroups.some(group =>
                group.getAttr('annotationIndex') === index
            );

            if (!exists) {
                const annotationGroup = this.drawAnnotationShape(annotation, scale, offsetX, offsetY, index);
                if (annotationGroup) {
                    // Add index for duplicate checking
                    annotationGroup.setAttr('annotationIndex', index);
                    this.state.annotationGroups.push(annotationGroup);
                }
            }
        });

        // Batch draw for better performance
        this.state.annotationLayer.batchDraw();
        this.state.uiLayer.batchDraw();
    }

    // Draw individual annotation shape
    private drawAnnotationShape(annotation: KonvaAnnotation, scale: number, offsetX: number, offsetY: number, index: number): Konva.Group | null {
        if (!this.state.annotationLayer || !this.state.uiLayer) return null;

        const colors = [
            { fill: '#FF6B6B', stroke: '#E53E3E', text: '#FFFFFF' },
            { fill: '#4ECDC4', stroke: '#319795', text: '#FFFFFF' },
            { fill: '#45B7D1', stroke: '#3182CE', text: '#FFFFFF' },
            { fill: '#96CEB4', stroke: '#38A169', text: '#FFFFFF' },
            { fill: '#FFEAA7', stroke: '#D69E2E', text: '#000000' },
            { fill: '#DDA0DD', stroke: '#9F7AEA', text: '#FFFFFF' },
            { fill: '#98D8C8', stroke: '#4FD1C9', text: '#000000' },
        ];
        const colorScheme = colors[index % colors.length];

        try {
            const annotationGroup = new Konva.Group({
                draggable: true,
                name: `annotation-${index}`,
                // Performance optimizations
                listening: true,
                visible: true,
            });

            let shape: Konva.Shape | null = null;
            let bounds: { x: number; y: number; width: number; height: number } | null = null;

            if ((annotation.shape_type === 'rectangle' || annotation.shape_type === 'bounding_box') && annotation.points && annotation.points.length >= 2) {
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

                // Optimized rectangle creation
                shape = new Konva.Rect({
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                    fill: colorScheme.fill,
                    stroke: colorScheme.stroke,
                    strokeWidth: 2,
                    opacity: 0.25,
                    // Removed shadow for better performance
                    listening: false, // Shape doesn't need to listen for events
                });
            } else if (annotation.shape_type === 'polygon' && annotation.points && annotation.points.length > 2) {
                const points: number[] = [];
                annotation.points.forEach((point) => {
                    points.push(point[0] * scale + offsetX);
                    points.push(point[1] * scale + offsetY);
                });

                const xs = annotation.points.map(p => p[0] * scale + offsetX);
                const ys = annotation.points.map(p => p[1] * scale + offsetY);
                bounds = {
                    x: Math.min(...xs),
                    y: Math.min(...ys),
                    width: Math.max(...xs) - Math.min(...xs),
                    height: Math.max(...ys) - Math.min(...ys)
                };

                // Optimized polygon creation
                shape = new Konva.Line({
                    points: points,
                    fill: colorScheme.fill,
                    stroke: colorScheme.stroke,
                    strokeWidth: 2,
                    opacity: 0.25,
                    closed: true,
                    // Removed shadow for better performance
                    listening: false, // Shape doesn't need to listen for events
                });
            }

            if (shape && bounds) {
                annotationGroup.add(shape);

                // Add simplified label if available (only for larger annotations)
                if (annotation.label && bounds.width > 50 && bounds.height > 20) {
                    const labelText = annotation.label.length > 10
                        ? annotation.label.substring(0, 10) + '...'
                        : annotation.label;

                    const label = new Konva.Text({
                        x: bounds.x + 5,
                        y: bounds.y - 20,
                        text: labelText,
                        fontSize: 11,
                        fontFamily: 'Arial',
                        fontStyle: 'bold',
                        fill: colorScheme.stroke,
                        listening: false,
                    });

                    // Only add label if it fits within bounds
                    if (label.width() < bounds.width - 10) {
                        annotationGroup.add(label);
                    }
                }

                // Simplified hover effects (only change opacity)
                annotationGroup.on('mouseenter', () => {
                    document.body.style.cursor = 'pointer';
                    if (shape) {
                        shape.opacity(0.4);
                        this.state.annotationLayer!.batchDraw();
                    }
                });

                annotationGroup.on('mouseleave', () => {
                    document.body.style.cursor = 'default';
                    if (shape) {
                        shape.opacity(0.25);
                        this.state.annotationLayer!.batchDraw();
                    }
                });

                // Add click handler
                annotationGroup.on('click tap', (e) => {
                    e.cancelBubble = true;
                    this.selectAnnotation(annotationGroup);
                });

                this.state.annotationLayer.add(annotationGroup);
                return annotationGroup;
            }
        } catch (error) {
            console.error('Error drawing annotation:', error, annotation);
        }

        return null;
    }

    // Add hover effects to annotation
    private addHoverEffects(annotationGroup: Konva.Group, shape: Konva.Shape, annotation: KonvaAnnotation): void {
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

        this.state.uiLayer!.add(tooltipBg);
        this.state.uiLayer!.add(tooltip);

        annotationGroup.on('mouseenter', () => {
            document.body.style.cursor = 'pointer';
            shape.opacity(0.4);
            this.state.annotationLayer!.draw();

            // Show tooltip
            const mousePos = this.state.stage!.getPointerPosition();
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
                this.state.uiLayer!.draw();
            }
        });

        annotationGroup.on('mouseleave', () => {
            document.body.style.cursor = 'default';
            shape.opacity(0.25);
            this.state.annotationLayer!.draw();

            // Hide tooltip
            tooltip.visible(false);
            tooltipBg.visible(false);
            this.state.uiLayer!.draw();
        });
    }

    // Zoom functions
    zoomIn(): void {
        if (!this.state.stage) return;
        const newScale = Math.min(5, this.state.stage.scaleX() * 1.2);
        this.zoomToScale(newScale);
    }

    zoomOut(): void {
        if (!this.state.stage) return;
        const newScale = Math.max(0.1, this.state.stage.scaleX() * 0.8);
        this.zoomToScale(newScale);
    }

    resetZoom(): void {
        this.zoomToScale(1);
        this.state.stage!.position({ x: 0, y: 0 });
        this.state.stageX = 0;
        this.state.stageY = 0;
    }

    fitToScreen(): void {
        if (!this.state.stage || !this.state.imageNode) return;

        const containerRect = this.container?.getBoundingClientRect();
        if (!containerRect || containerRect.width === 0) return;

        // Use stage size if container rect is not available (though it should be here)
        const targetWidth = containerRect.width;
        const targetHeight = containerRect.height;

        const imageRect = this.state.imageNode.getClientRect({ skipTransform: true });
        const scaleX = targetWidth / imageRect.width;
        const scaleY = targetHeight / imageRect.height;

        // Find best fit scale
        const newScale = Math.min(scaleX, scaleY) * 0.95; // 5% padding

        this.zoomToScale(newScale);

        // Center stage
        const scaledWidth = imageRect.width * newScale;
        const scaledHeight = imageRect.height * newScale;

        const newX = (targetWidth - scaledWidth) / 2;
        const newY = (targetHeight - scaledHeight) / 2;

        this.state.stage.position({ x: newX, y: newY });
        this.state.stageX = newX;
        this.state.stageY = newY;
        this.state.stage.batchDraw();
    }

    // Resize stage to match current container dimensions
    resize(): void {
        if (!this.state.stage || !this.container) return;

        const rect = this.container.getBoundingClientRect();
        if (rect.width === 0 || rect.height === 0) return;

        console.log(`Resizing stage to: ${rect.width}x${rect.height}`);

        this.state.stage.width(rect.width);
        this.state.stage.height(rect.height);
        this.state.stage.batchDraw();
    }

    private zoomToScale(newScale: number): void {
        if (!this.state.stage) return;

        const center = {
            x: this.state.stage.width() / 2,
            y: this.state.stage.height() / 2,
        };

        const relatedTo = {
            x: (center.x - this.state.stage.x()) / this.state.stage.scaleX(),
            y: (center.y - this.state.stage.y()) / this.state.stage.scaleY(),
        };

        this.state.stage.scale({ x: newScale, y: newScale });

        const newPos = {
            x: center.x - relatedTo.x * newScale,
            y: center.y - relatedTo.y * newScale,
        };

        this.state.stage.position(newPos);
        this.state.stage.batchDraw();

        this.state.scale = newScale;
        this.state.stageX = newPos.x;
        this.state.stageY = newPos.y;
    }

    // Annotation selection functions
    selectAnnotation(annotationGroup: Konva.Group): void {
        if (!this.state.transformer || !this.state.stage) return;

        this.state.selectedAnnotation = annotationGroup;
        this.state.transformer.nodes([annotationGroup]);
        this.state.uiLayer!.draw();

        console.log('Selected annotation:', annotationGroup);
    }

    deselectAnnotation(): void {
        if (!this.state.transformer) return;

        this.state.selectedAnnotation = null;
        this.state.transformer.nodes([]);
        this.state.uiLayer!.draw();

        console.log('Deselected annotation');
    }

    selectAllAnnotations(): void {
        if (!this.state.transformer || !this.state.stage) return;

        if (this.state.annotationGroups.length > 0) {
            this.state.selectedAnnotation = this.state.annotationGroups;
            this.state.transformer.nodes(this.state.annotationGroups);
            this.state.uiLayer!.draw();
        }
    }

    deleteSelectedAnnotation(): void {
        if (!this.state.selectedAnnotation || !this.state.annotationLayer) return;

        if (Array.isArray(this.state.selectedAnnotation)) {
            this.state.selectedAnnotation.forEach(group => {
                group.destroy();
            });
            this.state.annotationGroups = this.state.annotationGroups.filter(group =>
                !(this.state.selectedAnnotation as Konva.Group[]).includes(group)
            );
        } else if (this.state.selectedAnnotation) {
            this.state.selectedAnnotation.destroy();
            this.state.annotationGroups = this.state.annotationGroups.filter(group =>
                group !== this.state.selectedAnnotation
            );
        }

        this.deselectAnnotation();
        this.state.annotationLayer.draw();

        console.log('Deleted selected annotation(s)');
    }

    // Get current state for external access
    getState(): KonvaState {
        return { ...this.state };
    }

    // Update state from external source
    setState(newState: Partial<KonvaState>): void {
        this.state = { ...this.state, ...newState };
    }

    // Cleanup and destroy stage
    cleanup(): void {
        if (this.state.stage) {
            this.state.stage.destroy();
        }
        this.state = this.getInitialState();
    }

    // Get current zoom level as percentage
    getZoomPercentage(): number {
        return Math.round(this.state.scale * 100);
    }

    // Get annotation count
    getAnnotationCount(): number {
        return this.state.annotationGroups.length;
    }

    // Get selected annotation count
    getSelectedCount(): number {
        if (!this.state.selectedAnnotation) return 0;
        return Array.isArray(this.state.selectedAnnotation) ? this.state.selectedAnnotation.length : 1;
    }
}

// Factory function to create KonvaManager instance
export function createKonvaManager(): KonvaManager {
    return new KonvaManager();
}

// Utility functions for KonvaJS operations
export const konvaUtils = {
    // Convert file path to Tauri-compatible URL
    convertFileSrc: (path: string): string => {
        return safeConvertFileSrc(path);
    },

    // Calculate optimal stage dimensions
    calculateStageDimensions: (container: HTMLElement): { width: number; height: number } => {
        const rect = container.getBoundingClientRect();
        return {
            width: Math.max(rect.width || 800, 800),
            height: Math.max(rect.height || 600, 600)
        };
    },

    // Get color scheme for annotation index
    getColorScheme: (index: number) => {
        const colors = [
            { fill: '#FF6B6B', stroke: '#E53E3E', text: '#FFFFFF' },
            { fill: '#4ECDC4', stroke: '#319795', text: '#FFFFFF' },
            { fill: '#45B7D1', stroke: '#3182CE', text: '#FFFFFF' },
            { fill: '#96CEB4', stroke: '#38A169', text: '#FFFFFF' },
            { fill: '#FFEAA7', stroke: '#D69E2E', text: '#000000' },
            { fill: '#DDA0DD', stroke: '#9F7AEA', text: '#FFFFFF' },
            { fill: '#98D8C8', stroke: '#4FD1C9', text: '#000000' },
        ];
        return colors[index % colors.length];
    }
};

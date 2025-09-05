<script lang="ts">
    import { onMount, tick } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { Accordion, AccordionItem } from 'flowbite-svelte';

    // State variables
    let directoryPath = "";
    let images = [];
    let loading = false;
    let loadingMore = false;
    let error = "";
    let viewMode = "grid"; // 'grid' or 'column'
    let selectedImage = null;
    let detailedImage = null;
    let annotating = false; // Tracks annotation status
    let annotationType = "bounding_box"; // Default annotation type
    let datasetSummary = null; // New variable for dataset summary

    // YOLO export settings
    let showExportModal = false;
    let exportSettings = {
        outputDir: "",
        trainRatio: 0.7,
        valRatio: 0.2,
        testRatio: 0.1,
        shapeType: "polygon",
    };
    let exportLoading = false;
    let exportError = "";
    let exportSuccess = "";
    let excludedLabels = new Set<string>();

    // --- Extract Labels State ---
    let showExtractModal = false;
    let extractSettings = {
        outputDir: "",
    };
    let selectedLabelsForExtract = new Set<string>();
    let extractLoading = false;
    let extractError = "";
    let extractSuccess = "";

    // Pagination
    let currentPage = 1;
    let pageSize = 30;
    let totalPages = 0;
    let totalImages = 0;

    // --- Crop and Remap Tool State ---
    let cropSourceDir: string | null = null;
    let cropOutputDir: string | null = null;
    let parentLabel: string = "person"; // Default value
    let cropLoading: boolean = false;
    let cropStatusMessage: string | null = null;
    let cropIsError: boolean = false;
    let cropToolOpen = false; // State for Accordion

    // Container element for the gallery
    let containerElement;

    onMount(() => {
        // Any initialization code can go here
    });

    // Handle keyboard shortcuts
    function handleKeydown(event) {
        // Close image view when Escape key is pressed
        if (event.key === "Escape" && selectedImage) {
            closeImageView();
        }
    }

    async function selectDirectory() {
        try {
            loading = true;
            error = "";

            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select Image Directory",
            });

            if (selected) {
                directoryPath = selected;
                // Reset pagination
                currentPage = 1;
                images = [];

                // Validate the directory path
                if (!directoryPath.trim()) {
                    error = "Invalid directory path: path is empty";
                    loading = false;
                    return;
                }

                console.log("Selected directory:", directoryPath);

                try {
                    await loadImagesPage(1);
                } catch (loadErr) {
                    console.error("Error in loadImagesPage:", loadErr);
                    const errMsg =
                        loadErr?.message || String(loadErr) || "Unknown error";
                    error = `Failed to load images: ${errMsg}`;
                }
            }
        } catch (err) {
            console.error("Error selecting directory:", err);
            const errMsg = err?.message || String(err) || "Unknown error";
            error = `Failed to select directory: ${errMsg}`;
        } finally {
            loading = false;
        }
    }

    // Process images and prepare them for display
    function processImages(newImages, isFirstPage) {
        const processed = newImages.map((img, index) => {
            const actualIndex = isFirstPage ? index : images.length + index;
            return {
                ...img,
                previewUrl: convertFileSrc(img.path),
                isLoaded: false,
                displayIndex: actualIndex,
            };
        });

        return processed;
    }

    async function loadImagesPage(page) {
        try {
            loading = true;
            error = "";

            console.log(
                `Loading page ${page} with page size ${pageSize} from directory ${directoryPath}`,
            );

            // Validate inputs
            if (!directoryPath) {
                throw new Error("Directory path is empty");
            }

            // Call Rust function to get paginated images
            console.log("Invoking get_paginated_images with params:", {
                path: directoryPath,
                page,
                pageSize,
            });

            const result = await invoke("get_paginated_images", {
                path: directoryPath,
                page,
                pageSize,
            });

            console.log(
                "Raw result:",
                result ? result.substring(0, 100) + "..." : "null or undefined",
            );

            if (result) {
                const data = JSON.parse(result);
                console.log(
                    `Received data: ${data.images.length} images, total: ${data.total}`,
                );

                // Update pagination info
                totalImages = data.total;
                totalPages = data.total_pages;
                currentPage = page; // Ensure current page is set

                // Process images - reset for each page
                const newImages = processImages(data.images, true);
                console.log(`Processed ${newImages.length} images`);

                // Always reset the images array for pagination
                images = newImages;

                // Generate dataset summary on first load
                if (page === 1) {
                    generateLabelMeSummary();
                }

                await tick();

                // Setup lazy loading for new images
                setupLazyLoading();

                // Scroll to top when changing pages
                window.scrollTo(0, 0);
            } else {
                console.error("Empty result received from backend");
                throw new Error("Empty result from server");
            }
        } catch (err) {
            console.error("Error loading images:", err);
            // Improved error handling to provide more context
            const errorMessage =
                err?.message ||
                (typeof err === "string" ? err : "Unknown error");
            error = `Failed to load images: ${errorMessage}`;

            if (err?.stack) {
                console.error("Stack trace:", err.stack);
            }

            images = [];
        } finally {
            loading = false;
            loadingMore = false;
        }
    }

    function setupLazyLoading() {
        // Use Intersection Observer to lazy load images
        const imageElements = document.querySelectorAll(
            ".lazy-image:not(.observed)",
        );

        const imgObserver = new IntersectionObserver(
            (entries) => {
                entries.forEach((entry) => {
                    if (entry.isIntersecting) {
                        const img = entry.target;
                        const index = parseInt(img.getAttribute("data-index"));
                        const imageSrc = img.getAttribute("data-src");

                        // Directly set the src attribute which triggers the load event
                        if (imageSrc) {
                            img.src = imageSrc;
                        }

                        // Stop observing this image
                        imgObserver.unobserve(img);
                    }
                });
            },
            { rootMargin: "200px 0px", threshold: 0.1 },
        );

        imageElements.forEach((img) => {
            imgObserver.observe(img);
            img.classList.add("observed");
        });
    }

    async function selectImage(image, index) {
        // If we don't have dimensions, fetch the full details
        if (!image.dimensions) {
            try {
                const details = await invoke("get_image_details", {
                    path: image.path,
                });
                if (details) {
                    const detailedData = JSON.parse(details);
                    // Update the image with full details
                    images[index] = {
                        ...images[index],
                        dimensions: detailedData.dimensions,
                    };
                }
            } catch (err) {
                console.error("Error fetching image details:", err);
            }
        }

        selectedImage = images[index];

        // Wait for image to load then draw annotations if available
        if (selectedImage.annotations && selectedImage.annotations.length > 0) {
            setTimeout(() => {
                drawAnnotations();
            }, 100);
        }
    }

    function closeImageView() {
        selectedImage = null;
    }

    function formatFileSize(bytes) {
        if (bytes < 1024) return bytes + " B";
        if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
        return (bytes / 1048576).toFixed(1) + " MB";
    }

    function getVisibleImages() {
        // Simple implementation - for a more advanced solution, calculate based on scroll position
        return images;
    }

    // Handle view mode change
    function changeViewMode(mode) {
        viewMode = mode;
    }

    // Helper function to generate pagination page numbers
    function generatePageNumbers(current, total) {
        // Show first page, last page, current page, and one page before and after current
        let pages = [];

        // Always include first page
        pages.push(1);

        // If current page is more than 3, show ellipsis
        if (current > 3) {
            pages.push("...");
        }

        // Show page before current if it exists and isn't first page
        if (current > 2) {
            pages.push(current - 1);
        }

        // Show current page if it's not first or last
        if (current !== 1 && current !== total) {
            pages.push(current);
        }

        // Show page after current if it exists and isn't last page
        if (current < total - 1) {
            pages.push(current + 1);
        }

        // If current page is less than total-2, show ellipsis
        if (current < total - 2) {
            pages.push("...");
        }

        // Always include last page if different from first
        if (total > 1) {
            pages.push(total);
        }

        // Remove duplicates (can happen with small page ranges)
        return [...new Set(pages)];
    }

    // Function for handling annotation type change
    function setAnnotationType(type) {
        annotationType = type;
    }

    // Function for auto-annotating images
    async function annotateImages() {
        try {
            if (!directoryPath || images.length === 0) {
                error = "Please select a directory with images first";
                return;
            }

            annotating = true;
            error = "";

            console.log(
                `Starting ${annotationType} annotation for images in directory:`,
                directoryPath,
            );
            console.log("Params:", {
                path: directoryPath,
                page: currentPage,
                pageSize,
                annotationType,
            });

            // Call Rust function to annotate images with chosen annotation type
            const result = await invoke("auto_annotate_images", {
                path: directoryPath,
                page: currentPage,
                pageSize,
                annotationType,
            });

            console.log("Annotation result:", result);

            if (result) {
                const data = JSON.parse(result);
                console.log("Parsed annotation data:", data);

                // Update images with annotation results
                if (data.annotated_images && data.annotated_images.length > 0) {
                    // Merge annotation data with existing images
                    images = images.map((img, index) => {
                        const annotatedImg = data.annotated_images.find(
                            (ai) => ai.path === img.path,
                        );
                        if (annotatedImg) {
                            console.log(
                                `Annotations for ${img.name}:`,
                                annotatedImg.annotations,
                            );
                            return {
                                ...img,
                                annotations: annotatedImg.annotations,
                                annotated:
                                    annotatedImg.has_json &&
                                    annotatedImg.annotations.length > 0,
                                hasJson: annotatedImg.has_json,
                            };
                        }
                        return img;
                    });

                    // Count how many images have JSON files
                    const imagesWithJson = data.annotated_images.filter(
                        (img) => img.has_json,
                    ).length;
                    const imagesWithAnnotations = data.annotated_images.filter(
                        (img) => img.annotations.length > 0,
                    ).length;

                    console.log(
                        `Found ${imagesWithJson} images with JSON files, ${imagesWithAnnotations} with matching ${annotationType} annotations`,
                    );
                } else {
                    console.log("No annotations found");
                }
            }
        } catch (err) {
            console.error("Error annotating images:", err);
            const errorMessage =
                err?.message ||
                (typeof err === "string" ? err : "Unknown error");
            error = `Failed to annotate images: ${errorMessage}`;

            if (err?.stack) {
                console.error("Stack trace:", err.stack);
            }
        } finally {
            annotating = false;
        }
    }

    // New function to draw annotations on the selected image
    function drawAnnotations() {
        if (!selectedImage || !selectedImage.annotations) return;

        const imageElement = document.getElementById("selected-image");
        const canvasElement = document.getElementById("annotation-canvas");

        if (!imageElement || !canvasElement) return;

        // Wait for image to be loaded
        if (!imageElement.complete) {
            imageElement.onload = () => drawAnnotations();
            return;
        }

        const canvas = canvasElement as HTMLCanvasElement;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        // Set canvas size to match the actual displayed image size
        canvas.width = imageElement.clientWidth;
        canvas.height = imageElement.clientHeight;

        // Clear previous drawings
        ctx.clearRect(0, 0, canvas.width, canvas.height);

        // Calculate scale factors for the image
        const naturalWidth = imageElement.naturalWidth;
        const naturalHeight = imageElement.naturalHeight;
        const scaleX = canvas.width / naturalWidth;
        const scaleY = canvas.height / naturalHeight;

        // Draw each annotation
        selectedImage.annotations.forEach((annotation, index) => {
            if (!annotation.points) return;

            // Assign a color based on the index
            const hue = (index * 30) % 360;
            ctx.strokeStyle = `hsl(${hue}, 100%, 50%)`;
            ctx.fillStyle = `hsla(${hue}, 100%, 50%, 0.2)`;
            ctx.lineWidth = 3;

            // Start drawing
            ctx.beginPath();

            // Draw based on annotation type
            if (annotation.shape_type === "rectangle") {
                // For rectangles, we expect 2 points (top-left and bottom-right)
                if (annotation.points.length >= 2) {
                    const topLeft = annotation.points[0];
                    const bottomRight = annotation.points[1];

                    const x = topLeft[0] * scaleX;
                    const y = topLeft[1] * scaleY;
                    const width = (bottomRight[0] - topLeft[0]) * scaleX;
                    const height = (bottomRight[1] - topLeft[1]) * scaleY;

                    ctx.rect(x, y, width, height);
                }
            } else if (annotation.shape_type === "polygon") {
                // For polygons, we need at least 3 points
                if (annotation.points.length >= 3) {
                    // Move to the first point
                    ctx.moveTo(
                        annotation.points[0][0] * scaleX,
                        annotation.points[0][1] * scaleY,
                    );

                    // Draw lines to each subsequent point
                    for (let i = 1; i < annotation.points.length; i++) {
                        ctx.lineTo(
                            annotation.points[i][0] * scaleX,
                            annotation.points[i][1] * scaleY,
                        );
                    }

                    // Close the polygon
                    ctx.closePath();
                }
            }

            // Fill and stroke the path
            ctx.fill();
            ctx.stroke();

            // Add a label
            const firstPoint = annotation.points[0];
            const labelX = firstPoint[0] * scaleX;
            const labelY = firstPoint[1] * scaleY - 5;

            ctx.font = "14px Arial";
            ctx.fillStyle = `hsl(${hue}, 100%, 35%)`;
            ctx.fillText(annotation.label, labelX, labelY);
        });
    }

    // Function to generate LabelMe summary for the dataset
    async function generateLabelMeSummary() {
        try {
            if (!directoryPath || images.length === 0) {
                return;
            }

            console.log(
                "Generating LabelMe summary for directory:",
                directoryPath,
            );

            // Call Rust function to generate dataset summary
            const result = await invoke("get_labelme_summary", {
                path: directoryPath,
            });

            if (result) {
                datasetSummary = JSON.parse(result);
                console.log("Dataset summary:", datasetSummary);
            }
        } catch (err) {
            console.error("Error generating LabelMe summary:", err);
            const errorMessage =
                err?.message ||
                (typeof err === "string" ? err : "Unknown error");
            console.warn(`Failed to generate LabelMe summary: ${errorMessage}`);
        }
    }

    // Function to open YOLO export modal
    function openExportModal() {
        excludedLabels = new Set<string>();
        showExportModal = true;
        exportSuccess = "";
        exportError = "";
    }

    // Function to close YOLO export modal
    function closeExportModal() {
        showExportModal = false;
    }

    // Function to select output directory for export
    async function selectExportDirectory() {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select Export Directory",
            });

            if (selected) {
                exportSettings.outputDir = selected;
                exportError = "";
            }
        } catch (err) {
            console.error("Error selecting export directory:", err);
            exportError = "Failed to select export directory";
        }
    }

    // Function to toggle label exclusion
    function toggleLabelExclusion(label: string) {
        if (excludedLabels.has(label)) {
            excludedLabels.delete(label);
        } else {
            excludedLabels.add(label);
        }
        excludedLabels = excludedLabels;
    }

    // Function to export dataset to YOLO format
    async function exportToYolo() {
        try {
            if (!directoryPath) {
                exportError = "Please select a source directory first";
                return;
            }

            if (!exportSettings.outputDir) {
                exportError = "Please select an output directory";
                return;
            }

            // Validate ratio values sum to 1
            const sum =
                exportSettings.trainRatio +
                exportSettings.valRatio +
                exportSettings.testRatio;
            if (Math.abs(sum - 1.0) > 0.01) {
                exportError = "Split ratios must sum to 1.0";
                return;
            }

            exportLoading = true;
            exportError = "";
            exportSuccess = "";

            // Determine included labels
            const allLabels = datasetSummary?.label_counts ? Object.keys(datasetSummary.label_counts) : [];
            const includedLabels = allLabels.filter(label => !excludedLabels.has(label));

            if (includedLabels.length === 0) {
                exportError = "No labels selected for export. Please include at least one label.";
                exportLoading = false;
                return;
            }

            const classNamesStr = includedLabels.join(',');

            console.log("Exporting to YOLO format with settings:", {
                sourceDir: directoryPath,
                outputDir: exportSettings.outputDir,
                trainRatio: exportSettings.trainRatio,
                valRatio: exportSettings.valRatio,
                testRatio: exportSettings.testRatio,
                shape: exportSettings.shapeType,
                specificLabels: true,
                classNamesStr: classNamesStr,
            });

            // Call Rust backend function to export
            const result = await invoke("export_to_yolo_new", {
                sourceDir: directoryPath,
                outputDir: exportSettings.outputDir,
                trainRatio: exportSettings.trainRatio,
                valRatio: exportSettings.valRatio,
                testRatio: exportSettings.testRatio,
                shape: exportSettings.shapeType,
                specificLabels: true,
                classNamesStr: classNamesStr,
            });

            console.log("Export result:", result);
            exportSuccess = String(result);

            // Close modal after successful export
            setTimeout(() => {
                if (exportSuccess) {
                    closeExportModal();
                }
            }, 2000);
        } catch (err) {
            console.error("Error exporting to YOLO:", err);
            exportError = err?.message || "Failed to export dataset";
        } finally {
            exportLoading = false;
        }
    }

    // --- Extract Labels Functions ---
    function openExtractModal() {
        extractSettings.outputDir = "";
        extractError = "";
        extractSuccess = "";

        // Initialize the Set, optionally pre-select all available labels
        if (datasetSummary?.label_counts) {
             // Start with all labels selected
            selectedLabelsForExtract = new Set(Object.keys(datasetSummary.label_counts));
            // Or start with none selected:
            // selectedLabelsForExtract = new Set<string>();
        } else {
            selectedLabelsForExtract = new Set<string>();
        }

        showExtractModal = true;
    }

    function closeExtractModal() {
        showExtractModal = false;
    }

    // Helper function to toggle label selection for extraction
    function toggleLabelForExtract(label: string) {
        if (selectedLabelsForExtract.has(label)) {
            selectedLabelsForExtract.delete(label);
        } else {
            selectedLabelsForExtract.add(label);
        }
        // Trigger reactivity by reassigning the set
        selectedLabelsForExtract = selectedLabelsForExtract;
    }

    async function selectExtractDirectory() {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select Output Directory for Extracted Labels",
            });

            if (selected) {
                extractSettings.outputDir = selected;
                extractError = ""; // Clear error on successful selection
            }
        } catch (err) {
            console.error("Error selecting extract output directory:", err);
            extractError = "Failed to select output directory";
        }
    }

    async function runExtractLabels() {
        if (!directoryPath) {
            extractError = "Please select a source directory first.";
            return;
        }
        if (!extractSettings.outputDir) {
            extractError = "Please select an output directory.";
            return;
        }
        // Check if the Set is empty instead of trimming the string
        if (selectedLabelsForExtract.size === 0) {
            extractError = "Please select at least one label to extract.";
            return;
        }

        extractLoading = true;
        extractError = "";
        extractSuccess = "";

        // Convert the Set to a comma-separated string
        const labelsStrToExtract = Array.from(selectedLabelsForExtract).join(',');

        try {
            console.log("Running label extraction with settings:", {
                sourceDir: directoryPath,
                outputDir: extractSettings.outputDir,
                labelsStr: labelsStrToExtract, // Use the generated string
            });

            const result = await invoke("extract_labels", {
                sourceDir: directoryPath,
                outputDir: extractSettings.outputDir,
                labelsStr: labelsStrToExtract, // Pass the generated string
            });

            console.log("Extract labels result:", result);
            extractSuccess = String(result);

            // Optionally close modal on success after a delay
            // setTimeout(() => {
            //     if (extractSuccess) closeExtractModal();
            // }, 2000);

        } catch (err) {
            console.error("Error extracting labels:", err);
            extractError = err?.message || "Failed to extract labels.";
        } finally {
            extractLoading = false;
        }
    }

    // --- Crop and Remap Tool Functions ---
    async function selectCropDirectory(type: 'source' | 'output') {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: `Select Crop ${type === 'source' ? 'Source' : 'Output'} Directory`,
            });

            if (selected && typeof selected === 'string') {
                if (type === 'source') {
                    cropSourceDir = selected;
                } else {
                    cropOutputDir = selected;
                }
                // Clear messages when a directory is selected
                cropStatusMessage = null;
                cropIsError = false;
            }
        } catch (err) {
            console.error("Error selecting crop directory:", err);
            cropStatusMessage = `Failed to select directory: ${err instanceof Error ? err.message : String(err)}`;
            cropIsError = true;
        }
    }

    async function runCropAndRemap() {
        if (!cropSourceDir || !cropOutputDir || !parentLabel) {
            cropStatusMessage = "Please select source directory, output directory, and enter a parent label for cropping.";
            cropIsError = true;
            return;
        }

        cropLoading = true;
        cropStatusMessage = null;
        cropIsError = false;

        try {
            const message = await invoke("crop_and_remap_annotations", {
                sourceDir: cropSourceDir,
                outputDir: cropOutputDir,
                parentLabel: parentLabel
            });
            cropStatusMessage = String(message);
            cropIsError = false;

            // --- Load results into viewer ---
            console.log(`Crop & Remap successful. Loading results from: ${cropOutputDir}`);
            directoryPath = cropOutputDir; // Set main viewer path
            currentPage = 1;       // Reset pagination
            images = [];           // Clear current images
            datasetSummary = null; // Clear old summary
            selectedImage = null;  // Close modal if open
            error = "";            // Clear main error message

            await loadImagesPage(1); // Load the new images

        } catch (err) {
            console.error("Error running crop/remap processing:", err);
            cropStatusMessage = `Processing failed: ${err instanceof Error ? err.message : String(err)}`;
            cropIsError = true;
        } finally {
            cropLoading = false;
        }
    }
</script>

<svelte:head>
    <title>Image Viewer</title>
    <meta
        name="description"
        content="Efficient image viewer for large image collections"
    />
</svelte:head>

<svelte:window on:keydown={handleKeydown} />

<div class="container mx-auto px-4 py-8">
    <div class="max-w-6xl mx-auto">
        <div class="mb-8">
            <h1 class="text-3xl font-bold text-gray-800 mb-6">Image Viewer</h1>

            <div class="flex flex-wrap items-center gap-4 mb-6">
                <!-- Controls that appear *after* directory selection -->
                {#if directoryPath}
                    <!-- Button to select/change directory (now appears only *after* initial selection) -->
                    <button
                        on:click={selectDirectory} 
                        class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                        disabled={loading}
                    >
                        {loading ? "Loading..." : "Change Directory"} <!-- Changed text slightly -->
                    </button>

                    {#if images.length > 0} 
                        <!-- Annotation controls -->
                        <div class="flex items-center gap-2">
                            <select
                                bind:value={annotationType}
                                class="bg-white border border-gray-300 rounded-md shadow-sm py-2 px-3 text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                            >
                                <option value="bounding_box">Bounding Boxes</option>
                                <option value="polygon">Polygons</option>
                            </select>

                            <button
                                on:click={annotateImages}
                                class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
                                disabled={annotating}
                            >
                                {annotating ? "Annotating..." : "Load Annotations"}
                            </button>

                            <!-- YOLO Export Button -->
                            <button
                                on:click={openExportModal}
                                class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-yellow-600 hover:bg-yellow-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-yellow-500"
                            >
                                Export to YOLO
                            </button>

                            <!-- Extract Labels Button -->
                            <button
                                on:click={openExtractModal}
                                class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-teal-600 hover:bg-teal-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-teal-500"
                                disabled={!directoryPath || images.length === 0}
                            >
                                Extract Labels
                            </button>

                        </div>
                    {/if}

                    <!-- Directory Path Display -->
                    <div class="text-sm text-gray-600 flex-1 truncate">
                        <span class="font-medium">Directory:</span>
                        {directoryPath}
                    </div>

                    {#if images.length > 0} 
                        <!-- View Mode Controls -->
                        <div class="flex items-center space-x-4 ml-auto">
                            <span class="text-sm text-gray-600">View:</span>
                            <button
                                class={`px-3 py-1 rounded-md text-sm ${viewMode === "grid" ? "bg-indigo-100 text-indigo-700" : "text-gray-700 hover:bg-gray-100"}`}
                                on:click={() => changeViewMode("grid")}
                            >
                                Grid
                            </button>
                            <button
                                class={`px-3 py-1 rounded-md text-sm ${viewMode === "column" ? "bg-indigo-100 text-indigo-700" : "text-gray-700 hover:bg-gray-100"}`}
                                on:click={() => changeViewMode("column")}
                            >
                                Column
                            </button>
                        </div>
                    {/if}
                {/if} <!-- End of #if directoryPath -->
            </div>

            <!-- NEW: Crop and Remap Tool Accordion -->
             <Accordion class="mb-6 border border-gray-200 rounded-lg" bind:open={cropToolOpen}>
                <AccordionItem>
                    <span slot="header" class="text-lg font-medium text-gray-700">Crop & Remap Tool</span>
                     <div class="p-4 bg-gray-50 space-y-4">
                        <!-- Source Directory -->
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">Crop Source Directory</label>
                            <div class="flex items-center gap-2">
                                <input
                                    type="text"
                                    readonly
                                    placeholder="Select source directory for cropping..."
                                    value={cropSourceDir || ''}
                                    class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-100 text-gray-600 text-sm truncate"
                                />
                                <button
                                    on:click={() => selectCropDirectory('source')}
                                    class="px-4 py-2 bg-gray-200 hover:bg-gray-300 text-gray-700 rounded-md border border-gray-300 text-sm"
                                >
                                    Browse...
                                </button>
                            </div>
                        </div>

                        <!-- Output Directory -->
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">Crop Output Directory</label>
                            <div class="flex items-center gap-2">
                                <input
                                    type="text"
                                    readonly
                                    placeholder="Select output directory for cropped results..."
                                    value={cropOutputDir || ''}
                                    class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-100 text-gray-600 text-sm truncate"
                                />
                                <button
                                    on:click={() => selectCropDirectory('output')}
                                    class="px-4 py-2 bg-gray-200 hover:bg-gray-300 text-gray-700 rounded-md border border-gray-300 text-sm"
                                >
                                    Browse...
                                </button>
                            </div>
                             <p class="text-xs text-gray-500 mt-1">Results will be loaded into the viewer automatically from this directory after processing.</p>
                        </div>

                        <!-- Parent Label Input -->
                        <div>
                            <label for="parentLabelInput" class="block text-sm font-medium text-gray-700 mb-1">Parent Label</label>
                            <input
                                type="text"
                                id="parentLabelInput"
                                bind:value={parentLabel}
                                placeholder="e.g., person, car"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500 text-sm"
                            />
                            <p class="text-xs text-gray-500 mt-1">The label of the object to crop around (only the first found instance per image will be used).</p>
                        </div>

                        <!-- Run Button -->
                        <div class="pt-2">
                            <button
                                on:click={runCropAndRemap}
                                disabled={cropLoading || !cropSourceDir || !cropOutputDir || !parentLabel}
                                class="w-full inline-flex justify-center items-center px-6 py-3 border border-transparent text-base font-medium rounded-md shadow-sm text-white bg-purple-600 hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500 disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                {#if cropLoading}
                                    <div class="mr-2 animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"></div>
                                    Processing Crop & Remap...
                                {:else}
                                    Run Crop & Remap and Load Results
                                {/if}
                            </button>
                        </div>

                         <!-- Status Messages -->
                        {#if cropStatusMessage}
                            <div class={`p-3 rounded-md text-sm mt-4 ${cropIsError ? 'bg-red-50 text-red-700' : 'bg-green-50 text-green-700'}`}>
                                <p class="font-medium">{cropIsError ? 'Error!' : 'Status:'}</p>
                                <p>{cropStatusMessage}</p>
                            </div>
                        {/if}
                    </div>
                </AccordionItem>
            </Accordion>

            {#if error}
                <div class="bg-red-50 text-red-700 p-4 rounded-md mb-6">
                    {error}
                </div>
            {/if}

            <!-- Dataset Summary Card -->
            {#if datasetSummary}
                <div class="bg-white rounded-lg shadow-md p-4 mb-6">
                    <h2 class="text-xl font-semibold text-gray-800 mb-3">
                        Dataset Summary
                    </h2>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div class="bg-blue-50 p-3 rounded-md">
                            <h3 class="text-sm font-medium text-blue-700">
                                File Statistics
                            </h3>
                            <p class="text-sm mt-1">
                                Total Images: <span class="font-medium"
                                    >{datasetSummary.total_images}</span
                                >
                            </p>
                            <p class="text-sm mt-1">
                                Images with Annotations: <span
                                    class="font-medium"
                                    >{datasetSummary.images_with_annotations}</span
                                >
                            </p>
                            <p class="text-sm mt-1">
                                Annotation Coverage: <span class="font-medium"
                                    >{(
                                        (datasetSummary.images_with_annotations /
                                            datasetSummary.total_images) *
                                        100
                                    ).toFixed(1)}%</span
                                >
                            </p>
                        </div>

                        <div class="bg-green-50 p-3 rounded-md">
                            <h3 class="text-sm font-medium text-green-700">
                                Annotation Statistics
                            </h3>
                            <p class="text-sm mt-1">
                                Total Annotations: <span class="font-medium"
                                    >{datasetSummary.total_annotations}</span
                                >
                            </p>
                            <p class="text-sm mt-1">
                                Average per Image: <span class="font-medium"
                                    >{(
                                        datasetSummary.total_annotations /
                                        datasetSummary.images_with_annotations
                                    ).toFixed(1)}</span
                                >
                            </p>
                            <p class="text-sm mt-1">
                                Annotation Types: <span class="font-medium"
                                    >{datasetSummary.annotation_types.join(
                                        ", ",
                                    )}</span
                                >
                            </p>
                        </div>

                        <div class="bg-purple-50 p-3 rounded-md">
                            <h3 class="text-sm font-medium text-purple-700">
                                Label Statistics
                            </h3>
                            <p class="text-sm mt-1">
                                Unique Labels: <span class="font-medium"
                                    >{datasetSummary.unique_labels}</span
                                >
                            </p>
                            <p class="text-sm mt-1">Label Counts:</p>
                            <!-- Make the list scrollable if it exceeds a certain height -->
                            <ul class="text-xs mt-1 space-y-1 max-h-32 overflow-y-auto pr-2"> 
                                {#each Object.entries(datasetSummary.label_counts) as [label, count]}  <!-- Removed .slice(0, 3) -->
                                    <li class="flex justify-between">
                                        <span class="font-medium truncate pr-2">{label}</span>
                                        <span>{count}</span>
                                    </li>
                                {/each}
                            </ul>
                        </div>
                    </div>
                </div>
            {/if}

            {#if totalImages > 0}
                <div class="text-sm text-gray-600 mb-4">
                    Showing {Math.min(images.length, totalImages)} of {totalImages}
                    images
                </div>
            {/if}
        </div>

        {#if loading && images.length === 0}
            <div class="flex justify-center items-center py-12">
                <div
                    class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"
                ></div>
            </div>
        {:else if annotating}
            <div class="flex flex-col justify-center items-center py-12">
                <div
                    class="animate-spin rounded-full h-12 w-12 border-b-2 border-green-600 mb-4"
                ></div>
                <p class="text-gray-600">
                    Loading {annotationType} annotations, please wait...
                </p>
            </div>
        {:else if !loading && directoryPath && images.length === 0 && !error}
             <div class="text-center py-12">
                 <p class="text-gray-500">No images found in this directory.</p>
             </div>
        {:else if !loading && !directoryPath && !error}
            <!-- Restored Enhanced Placeholder Area -->
            <div class="text-center py-16 px-6 border-2 border-dashed border-gray-300 rounded-lg mt-8">
                <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                    <path vector-effect="non-scaling-stroke" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
                </svg>
                <h3 class="mt-2 text-lg font-medium text-gray-900">No Directory Selected</h3>
                <p class="mt-1 text-sm text-gray-500">Select a directory containing your images and LabelMe (.json) annotations to begin.</p>
                <div class="mt-6">
                    <button
                        type="button"
                        on:click={selectDirectory} 
                        class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="-ml-1 mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                             <path stroke-linecap="round" stroke-linejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                        </svg>
                         Select Directory
                    </button>
                </div>
                 <p class="mt-4 text-sm text-gray-500">Alternatively, you can use the Crop & Remap tool above to process a dataset.</p>
            </div>
        {:else if images.length > 0}
            <div bind:this={containerElement}>
                <!-- Grid View -->
                {#if viewMode === "grid"}
                    <div
                        class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4"
                    >
                        {#each getVisibleImages() as image, index (image.path)}
                            <button type="button"
                                class="bg-white rounded-lg shadow-md overflow-hidden hover:shadow-lg transition-shadow duration-200 cursor-pointer relative text-left p-0 border-none block w-full"
                                on:click={() => selectImage(image, index)}
                                aria-label={`View details for ${image.name}`}
                            >
                                <div class="relative pb-[75%]">
                                    <!-- First page images load directly, others lazy load -->
                                    <img
                                        class={`absolute inset-0 w-full h-full object-cover transition-opacity duration-300 ${image.displayIndex < pageSize ? "opacity-100" : "lazy-image opacity-0"}`}
                                        data-src={image.previewUrl}
                                        data-index={index}
                                        alt={image.name}
                                        src={image.displayIndex < pageSize
                                            ? image.previewUrl
                                            : "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1 1'%3E%3C/svg%3E"}
                                        on:load={(e) => {
                                            if (
                                                e.target.src !==
                                                "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1 1'%3E%3C/svg%3E"
                                            ) {
                                                e.target.classList.add(
                                                    "loaded",
                                                );
                                            }
                                        }}
                                    />

                                    <!-- Status badges -->
                                    <div
                                        class="absolute top-2 right-2 flex flex-col gap-1"
                                    >
                                        {#if image.hasJson}
                                            <div
                                                class="bg-blue-500 text-white text-xs px-2 py-1 rounded-full shadow"
                                            >
                                                JSON
                                            </div>
                                        {/if}
                                        {#if image.annotated}
                                            <div
                                                class="bg-green-500 text-white text-xs px-2 py-1 rounded-full shadow"
                                            >
                                                {annotationType ===
                                                "bounding_box"
                                                    ? "Boxes"
                                                    : "Polygons"}
                                            </div>
                                        {/if}
                                    </div>
                                </div>
                                <div class="p-3">
                                    <p class="text-sm text-gray-800 truncate">
                                        {image.name}
                                    </p>
                                    {#if image.size}
                                        <p class="text-xs text-gray-500 mt-1">
                                            {formatFileSize(image.size)}
                                        </p>
                                    {/if}
                                    {#if image.annotations && image.annotations.length > 0}
                                        <p class="text-xs text-green-600 mt-1">
                                            {image.annotations.length}
                                            {annotationType === "bounding_box"
                                                ? "box"
                                                : "polygon"}{image.annotations
                                                .length !== 1
                                                ? "es"
                                                : ""}
                                        </p>
                                    {/if}
                                </div>
                            </button>
                        {/each}
                    </div>
                {:else}
                    <!-- Column View -->
                    <div class="space-y-4">
                        {#each getVisibleImages() as image, index (image.path)}
                            <button type="button"
                                class="bg-white rounded-lg shadow-md overflow-hidden hover:shadow-lg transition-shadow duration-200 cursor-pointer relative w-full text-left p-0 border-none block"
                                on:click={() => selectImage(image, index)}
                                aria-label={`View details for ${image.name}`}
                            >
                                <div class="flex flex-col sm:flex-row">
                                    <div class="sm:w-48 h-48 relative">
                                        <img
                                            class={`w-full h-full object-cover transition-opacity duration-300 ${image.displayIndex < pageSize ? "opacity-100" : "lazy-image opacity-0"}`}
                                            data-src={image.previewUrl}
                                            data-index={index}
                                            alt={image.name}
                                            src={image.displayIndex < pageSize
                                                ? image.previewUrl
                                                : "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1 1'%3E%3C/svg%3E"}
                                            on:load={(e) => {
                                                if (
                                                    e.target.src !==
                                                    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1 1'%3E%3C/svg%3E"
                                                ) {
                                                    e.target.classList.add(
                                                        "loaded",
                                                    );
                                                }
                                            }}
                                        />

                                        <!-- Status badges -->
                                        <div
                                            class="absolute top-2 right-2 flex flex-col gap-1"
                                        >
                                            {#if image.hasJson}
                                                <div
                                                    class="bg-blue-500 text-white text-xs px-2 py-1 rounded-full shadow"
                                                >
                                                    JSON
                                                </div>
                                            {/if}
                                            {#if image.annotated}
                                                <div
                                                    class="bg-green-500 text-white text-xs px-2 py-1 rounded-full shadow"
                                                >
                                                    {annotationType ===
                                                    "bounding_box"
                                                        ? "Boxes"
                                                        : "Polygons"}
                                                </div>
                                            {/if}
                                        </div>
                                    </div>
                                    <div class="p-4 flex-1">
                                        <h3
                                            class="text-lg font-medium text-gray-800"
                                        >
                                            {image.name}
                                        </h3>
                                        {#if image.size}
                                            <p
                                                class="text-sm text-gray-500 mt-1"
                                            >
                                                Size: {formatFileSize(
                                                    image.size,
                                                )}
                                            </p>
                                        {/if}
                                        {#if image.dimensions}
                                            <p
                                                class="text-sm text-gray-500 mt-1"
                                            >
                                                Dimensions: {image.dimensions
                                                    .width} × {image.dimensions
                                                    .height}
                                            </p>
                                        {/if}
                                        {#if image.created}
                                            <p
                                                class="text-sm text-gray-500 mt-1"
                                            >
                                                Created: {new Date(
                                                    image.created,
                                                ).toLocaleString()}
                                            </p>
                                        {/if}
                                        {#if image.annotations && image.annotations.length > 0}
                                            <div class="mt-2">
                                                <p
                                                    class="text-sm font-medium text-green-600"
                                                >
                                                    {image.annotations.length}
                                                    {annotationType ===
                                                    "bounding_box"
                                                        ? "bounding box"
                                                        : "polygon"}{image
                                                        .annotations.length !==
                                                    1
                                                        ? "es"
                                                        : ""}:
                                                </p>
                                                <ul
                                                    class="mt-1 list-disc list-inside text-sm text-gray-600"
                                                >
                                                    {#each image.annotations.slice(0, 3) as annotation}
                                                        <li class="truncate">
                                                            {annotation.label}
                                                        </li>
                                                    {/each}
                                                    {#if image.annotations.length > 3}
                                                        <li
                                                            class="text-gray-500"
                                                        >
                                                            +{image.annotations
                                                                .length - 3} more
                                                        </li>
                                                    {/if}
                                                </ul>
                                            </div>
                                        {/if}
                                    </div>
                                </div>
                            </button>
                        {/each}
                    </div>
                {/if}

                <!-- Pagination -->
                {#if !loading && images.length > 0 && totalPages > 1}
                    <div class="flex items-center justify-center my-8">
                        <nav
                            class="relative z-0 inline-flex rounded-md shadow-sm -space-x-px"
                            aria-label="Pagination"
                        >
                            <!-- Previous page button -->
                            <button
                                class="relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50"
                                on:click={() => {
                                    if (currentPage > 1) {
                                        currentPage--;
                                        loadImagesPage(currentPage);
                                    }
                                }}
                                disabled={currentPage === 1 ||
                                    loading ||
                                    loadingMore}
                            >
                                <span class="sr-only">Previous</span>
                                <!-- Previous icon -->
                                <svg
                                    class="h-5 w-5"
                                    xmlns="http://www.w3.org/2000/svg"
                                    viewBox="0 0 20 20"
                                    fill="currentColor"
                                    aria-hidden="true"
                                >
                                    <path
                                        fill-rule="evenodd"
                                        d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z"
                                        clip-rule="evenodd"
                                    />
                                </svg>
                            </button>

                            <!-- Page numbers -->
                            {#each generatePageNumbers(currentPage, totalPages) as pageNum}
                                {#if pageNum === "..."}
                                    <span
                                        class="relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium"
                                    >
                                        ...
                                    </span>
                                {:else}
                                    <button
                                        class="relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium {currentPage ===
                                        pageNum
                                            ? 'z-10 bg-indigo-50 border-indigo-500 text-indigo-600'
                                            : 'text-gray-500 hover:bg-gray-50'}"
                                        on:click={() => {
                                            if (currentPage !== pageNum) {
                                                currentPage = pageNum;
                                                loadImagesPage(currentPage);
                                            }
                                        }}
                                        disabled={loading || loadingMore}
                                    >
                                        {pageNum}
                                    </button>
                                {/if}
                            {/each}

                            <!-- Next page button -->
                            <button
                                class="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50"
                                on:click={() => {
                                    if (currentPage < totalPages) {
                                        currentPage++;
                                        loadImagesPage(currentPage);
                                    }
                                }}
                                disabled={currentPage === totalPages ||
                                    loading ||
                                    loadingMore}
                            >
                                <span class="sr-only">Next</span>
                                <!-- Next icon -->
                                <svg
                                    class="h-5 w-5"
                                    xmlns="http://www.w3.org/2000/svg"
                                    viewBox="0 0 20 20"
                                    fill="currentColor"
                                    aria-hidden="true"
                                >
                                    <path
                                        fill-rule="evenodd"
                                        d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
                                        clip-rule="evenodd"
                                    />
                                </svg>
                            </button>
                        </nav>
                    </div>
                {/if}

                {#if loadingMore}
                    <div class="h-10 my-8 flex justify-center">
                        <div
                            class="animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-600"
                        ></div>
                    </div>
                {/if}
            </div>
        {/if}
    </div>
</div>

<!-- Image Viewer Modal with Annotation Support -->
{#if selectedImage}
    <div
        class="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50 p-4"
    >
        <div
            class="relative max-w-6xl w-full bg-white rounded-lg shadow-xl overflow-hidden"
        >
            <div class="flex justify-between items-center p-4 border-b">
                <h3 class="text-lg font-medium text-gray-800 truncate">
                    {selectedImage.name}
                </h3>
                <button
                    on:click={closeImageView}
                    class="text-gray-400 hover:text-gray-500"
                >
                    ✕
                </button>
            </div>
            <div
                class="p-4 flex flex-col items-center justify-center max-h-[calc(100vh-8rem)] overflow-auto"
            >
                <div class="relative">
                    <!-- Main image -->
                    <img
                        id="selected-image"
                        src={selectedImage.previewUrl}
                        alt={selectedImage.name}
                        class="max-w-full max-h-[calc(100vh-12rem)] object-contain"
                    />

                    <!-- Annotation canvas layer -->
                    {#if selectedImage.annotations && selectedImage.annotations.length > 0}
                        <canvas
                            id="annotation-canvas"
                            class="absolute top-0 left-0 w-full h-full pointer-events-none"
                        ></canvas>
                    {/if}
                </div>

                <!-- Display annotations in modal if available -->
                {#if selectedImage.annotations && selectedImage.annotations.length > 0}
                    <div
                        class="mt-4 p-3 bg-green-50 rounded-md border border-green-100 max-w-xl w-full"
                    >
                        <h4 class="text-md font-medium text-green-800 mb-2">
                            {annotationType === "bounding_box"
                                ? "Bounding Box"
                                : "Polygon"} Annotations
                        </h4>
                        <ul class="space-y-1">
                            {#each selectedImage.annotations as annotation}
                                <li class="flex justify-between text-sm">
                                    <span class="text-gray-700"
                                        >{annotation.label}</span
                                    >
                                    <span class="text-gray-500 font-medium"
                                        >{annotation.shape_type}</span
                                    >
                                </li>
                            {/each}
                        </ul>
                    </div>
                {/if}
            </div>
            <div class="p-4 border-t bg-gray-50">
                <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                    {#if selectedImage.size}
                        <div>
                            <span class="text-sm text-gray-500">Size:</span>
                            <span class="text-sm font-medium ml-1"
                                >{formatFileSize(selectedImage.size)}</span
                            >
                        </div>
                    {/if}
                    {#if selectedImage.dimensions}
                        <div>
                            <span class="text-sm text-gray-500"
                                >Dimensions:</span
                            >
                            <span class="text-sm font-medium ml-1"
                                >{selectedImage.dimensions.width} × {selectedImage
                                    .dimensions.height}</span
                            >
                        </div>
                    {/if}
                    {#if selectedImage.created}
                        <div>
                            <span class="text-sm text-gray-500">Created:</span>
                            <span class="text-sm font-medium ml-1"
                                >{new Date(
                                    selectedImage.created,
                                ).toLocaleString()}</span
                            >
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    </div>
{/if}

<!-- YOLO Export Modal -->
{#if showExportModal}
    <div
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
    >
        <div class="max-w-2xl w-full bg-white rounded-lg shadow-xl overflow-hidden flex flex-col max-h-[90vh]">
            <!-- Header -->
            <div class="px-6 py-4 border-b">
                <h3 class="text-xl font-bold">Export to YOLO Format</h3>
            </div>

            <!-- Form -->
            <div class="px-6 py-4 overflow-y-auto">
                {#if exportSuccess}
                    <div class="bg-green-50 text-green-700 p-3 rounded-md mb-4">
                        {exportSuccess}
                    </div>
                {/if}
                {#if exportError}
                    <div class="bg-red-50 text-red-700 p-3 rounded-md mb-4">
                        {exportError}
                    </div>
                {/if}

                <!-- Source Directory -->
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="sourceDirInput">Source Directory</label>
                    <input
                        type="text"
                        id="sourceDirInput" 
                        value={directoryPath}
                        readonly
                        class="w-full px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-500"
                    />
                </div>

                <!-- Output Directory -->
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="outputDirInput">Output Directory</label>
                    <div class="flex gap-2">
                        <input
                            type="text"
                            id="outputDirInput"
                            bind:value={exportSettings.outputDir}
                            readonly
                            class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-500"
                        />
                        <button
                            on:click={selectExportDirectory}
                            class="px-3 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md border border-gray-300"
                        >
                            Browse...
                        </button>
                    </div>
                </div>

                <!-- Shape Type -->
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="shapeTypeSelect">Shape Type</label>
                    <select
                        id="shapeTypeSelect"
                        bind:value={exportSettings.shapeType}
                        class="w-full px-3 py-2 border border-gray-300 rounded-md"
                    >
                        <option value="polygon">Polygon</option>
                        <option value="bounding_box">Bounding Box</option>
                    </select>
                </div>

                <!-- Data Split Ratios -->
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="trainRatioInput">Train Ratio</label>
                    <input
                        type="number"
                        id="trainRatioInput"
                        bind:value={exportSettings.trainRatio}
                        min="0"
                        max="1"
                        step="0.1"
                        class="w-full px-3 py-2 border border-gray-300 rounded-md"
                    />
                </div>
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="valRatioInput">Validation Ratio</label>
                    <input
                        type="number"
                        id="valRatioInput"
                        bind:value={exportSettings.valRatio}
                        min="0"
                        max="1"
                        step="0.1"
                        class="w-full px-3 py-2 border border-gray-300 rounded-md"
                    />
                </div>
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="testRatioInput">Test Ratio</label>
                    <input
                        type="number"
                        id="testRatioInput"
                        bind:value={exportSettings.testRatio}
                        min="0"
                        max="1"
                        step="0.1"
                        class="w-full px-3 py-2 border border-gray-300 rounded-md"
                    />
                </div>

                <!-- Label Selection Section -->
                <div class="mb-4">
                    <label class="block text-gray-700 mb-2 font-medium">Labels to Export</label>
                    {#if datasetSummary?.label_counts && Object.keys(datasetSummary.label_counts).length > 0}
                        <p class="text-sm text-gray-500 mb-3">Click on a label to exclude it from the export. By default, all labels are included.</p>
                        <div class="flex flex-wrap gap-2">
                            {#each Object.entries(datasetSummary.label_counts) as [label, count]}
                                <button
                                    type="button"
                                    class={`px-3 py-1 rounded-full text-sm border transition-colors duration-150
                                        ${!excludedLabels.has(label)
                                            ? 'bg-blue-100 text-blue-800 border-blue-300 hover:bg-blue-200'
                                            : 'bg-gray-100 text-gray-600 border-gray-300 hover:bg-gray-200 line-through'
                                        }
                                    `}
                                    on:click={() => toggleLabelExclusion(label)}
                                >
                                    {label} ({count})
                                </button>
                            {/each}
                        </div>
                    {:else if datasetSummary}
                         <p class="text-sm text-gray-500">No labels found in the dataset summary.</p>
                    {:else}
                         <p class="text-sm text-gray-500">Loading labels...</p>
                    {/if}
                </div>
            </div>

            <!-- Actions -->
            <div class="px-6 py-4 border-t bg-gray-50 flex justify-end space-x-2 mt-auto">
                <button
                    class="px-4 py-2 border rounded-md hover:bg-gray-100"
                    on:click={closeExportModal}
                    disabled={exportLoading}
                >
                    Cancel
                </button>
                <button
                    class="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-md disabled:opacity-50 flex items-center"
                    on:click={exportToYolo}
                    disabled={exportLoading || !exportSettings.outputDir || !datasetSummary?.label_counts}
                >
                    {#if exportLoading}
                        <div
                            class="mr-2 animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"
                        ></div>
                        Exporting...
                    {:else}
                        Export
                    {/if}
                </button>
            </div>
        </div>
    </div>
{/if}

<!-- Extract Labels Modal -->
{#if showExtractModal}
    <div
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
    >
        <div class="max-w-xl w-full bg-white rounded-lg shadow-xl overflow-hidden flex flex-col max-h-[90vh]">
            <!-- Header -->
            <div class="px-6 py-4 border-b">
                <h3 class="text-xl font-bold">Extract Specific Labels</h3>
            </div>

            <!-- Form -->
            <div class="px-6 py-4 overflow-y-auto">
                {#if extractSuccess}
                    <div class="bg-green-50 text-green-700 p-3 rounded-md mb-4">
                        {extractSuccess}
                    </div>
                {/if}
                {#if extractError}
                    <div class="bg-red-50 text-red-700 p-3 rounded-md mb-4">
                        {extractError}
                    </div>
                {/if}

                <!-- Source Directory -->
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="extractSourceDir">Source Directory</label>
                    <input
                        type="text"
                        id="extractSourceDir" 
                        value={directoryPath}
                        readonly
                        class="w-full px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-500"
                    />
                </div>

                <!-- Output Directory -->
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="extractOutputDir">Output Directory</label>
                    <div class="flex gap-2">
                        <input
                            type="text"
                            id="extractOutputDir"
                            bind:value={extractSettings.outputDir}
                            readonly
                            placeholder="Select directory for extracted files..."
                            class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-500"
                        />
                        <button
                            on:click={selectExtractDirectory}
                            class="px-3 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md border border-gray-300"
                        >
                            Browse...
                        </button>
                    </div>
                </div>

                <!-- Labels Input -->
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">Labels to Extract (check to include)</label>
                    <div class="max-h-60 overflow-y-auto border border-gray-200 rounded-md p-2 space-y-1">
                        {#each Object.entries(datasetSummary.label_counts) as [label, count]}
                            <label class="flex items-center space-x-2 cursor-pointer text-sm p-1 hover:bg-gray-50 rounded">
                                <input
                                    type="checkbox"
                                    checked={selectedLabelsForExtract.has(label)}
                                    on:change={() => toggleLabelForExtract(label)}
                                    class="rounded border-gray-300 text-indigo-600 shadow-sm focus:border-indigo-300 focus:ring focus:ring-offset-0 focus:ring-indigo-200 focus:ring-opacity-50"
                                />
                                <span>{label} ({count})</span>
                            </label>
                        {:else}
                            <p class="text-sm text-gray-500 italic p-2">No labels found in dataset summary.</p>
                        {/each}
                    </div>
                    <p class="text-xs text-gray-500 mt-1">Only annotations with the selected labels will be kept.</p>
                </div>
            </div>

            <!-- Actions -->
            <div class="px-6 py-4 border-t bg-gray-50 flex justify-end space-x-2 mt-auto">
                <button
                    class="px-4 py-2 border rounded-md hover:bg-gray-100"
                    on:click={closeExtractModal}
                    disabled={extractLoading}
                >
                    Cancel
                </button>
                <button
                    class="bg-teal-600 hover:bg-teal-700 text-white px-4 py-2 rounded-md disabled:opacity-50 flex items-center"
                    on:click={runExtractLabels}
                    disabled={extractLoading || !extractSettings.outputDir || selectedLabelsForExtract.size === 0}
                >
                    {#if extractLoading}
                        <div
                            class="mr-2 animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"
                        ></div>
                        Extracting...
                    {:else}
                        Run Extraction
                    {/if}
                </button>
            </div>
        </div>
    </div>
{/if}

<style>
    /* Add styles for lazy loading images */
    .lazy-image {
        opacity: 0;
        transition: opacity 0.3s ease-in-out;
    }

    .lazy-image.loaded {
        opacity: 1;
    }

    /* Placeholder shimmer effect */
    .lazy-image:not(.loaded) {
        background: linear-gradient(
            90deg,
            rgba(229, 232, 235, 0.6) 25%,
            rgba(215, 219, 223, 0.6) 37%,
            rgba(229, 232, 235, 0.6) 63%
        );
        background-size: 400% 100%;
        animation: shimmer 1.4s ease infinite;
    }

    /* Ensure images take up their container space */
    img.lazy-image {
        width: 100%;
        height: 100%;
        object-fit: cover;
        position: absolute;
        top: 0;
        left: 0;
    }

    @keyframes shimmer {
        0% {
            background-position: 100% 50%;
        }
        100% {
            background-position: 0 50%;
        }
    }
</style>

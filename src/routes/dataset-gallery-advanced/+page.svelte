<script lang="ts">
    import { onMount, tick } from "svelte";
    import { invoke, convertFileSrc } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import DatasetSummaryCard from "./components/DatasetSummaryCard.svelte";
    import ModalAnnotationViewer from "./components/ModalAnnotationViewer.svelte";
    import ImageGallery from "./components/ImageGallery.svelte";
    import ImageViewerModal from "./components/ImageViewerModal.svelte";
    import ExportModal from "./components/ExportModal.svelte";
    import {
        fetchPaginatedImages,
        fetchImageDetails,
        fetchDatasetSummary,
        performAutoAnnotation,
        performCropAndRemap,
        performDatasetExport,
    } from "./datasetService";
    import type {
        ProcessedImage,
        ImageDetailData,
        DatasetSummary,
        AutoAnnotationResult,
        AnnotatedImageInfo,
        DatasetExportParams,
    } from "./datasetService";

    // State variables
    let directoryPath = "";
    let images: ProcessedImage[] = []; // Use the ProcessedImage type
    let loading = false;
    let loadingMore = false;
    let error = "";
    let viewMode = "grid"; // 'grid' or 'column'
    let selectedImage: ProcessedImage | null = null; // Ensure selectedImage also uses ProcessedImage
    let annotating = false; // Tracks manual annotation status
    let autoAnnotating = false; // Tracks automatic annotation loading
    let annotationType = "bounding_box"; // Default annotation type
    let datasetSummary: DatasetSummary | null = null; // Use the DatasetSummary type
    let autoAnnotationEnabled = true; // Enable/disable automatic annotation loading

    // Modal Annotation Viewer State
    let showAnnotationModal = false;
    let modalSelectedImage: ProcessedImage | null = null;

    // Unified Export Modal State
    let showActualExportModal = false; // New state for controlling ExportModal visibility
    let pageExportLoading = false; // For main page UI during invoke
    let pageExportError = "";
    let pageExportSuccess = "";

    // Pagination
    let currentPage = 1;
    let pageSize = 30;
    let totalPages = 0;
    let totalImages = 0;

    // --- Crop and Remap Tool State --- (Removed, now internal to CropRemapTool.svelte)
    // let cropSourceDir: string | null = null;
    // let cropOutputDir: string | null = null;
    // let parentLabel: string = "person";
    // let cropLoading: boolean = false;
    // let cropStatusMessage: string | null = null;
    // let cropIsError: boolean = false;
    // let cropToolOpen = false; // State for Accordion (removed with crop tool)

    // Container element for the gallery
    // let containerElement; // Removed: Handled by ImageGallery or no longer needed

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

    async function loadImagesPage(page) {
        try {
            loading = true;
            error = "";

            console.log(
                `Page: Loading page ${page} for directory ${directoryPath}`,
            );

            if (!directoryPath) {
                throw new Error("Directory path is empty for loadImagesPage.");
            }

            const result = await fetchPaginatedImages(
                directoryPath,
                page,
                pageSize,
            );

            images = result.processedImages;
            totalImages = result.totalImages;
            totalPages = result.totalPages;
            currentPage = page;

            console.log(
                `Page: Loaded ${images.length} images. Total: ${totalImages}, Total Pages: ${totalPages}`,
            );

            if (page === 1) {
                generateLabelMeSummary(); // Still generate summary on first page load
            }

            // Auto-load annotations if enabled and images exist
            if (autoAnnotationEnabled && images.length > 0) {
                await autoLoadAnnotationsForPage(page);
            }

            await tick();
            // setupLazyLoading(); // This call is now in ImageGallery.svelte's afterUpdate
            // The ImageGallery component will handle its own lazy loading setup internally
            // when its `images` prop updates.

            if (typeof window !== "undefined") {
                window.scrollTo(0, 0);
            }
        } catch (err) {
            console.error("Page: Error loading images:", err);
            const errorMessage =
                err?.message ||
                (typeof err === "string" ? err : "Unknown error");
            error = `Failed to load images: ${errorMessage}`;

            if (err?.stack) {
                console.error("Stack trace:", err.stack);
            }
            images = []; // Clear images on error
            totalImages = 0;
            totalPages = 0;
        } finally {
            loading = false;
            loadingMore = false; // Ensure loadingMore is also reset
        }
    }

    async function selectImage(imageDetail: {
        image: ProcessedImage;
        index: number;
    }) {
        const { image, index } = imageDetail;
        let currentImageInState = images[index];

        // If we don't have dimensions (or other critical details), fetch the full details
        if (!currentImageInState.dimensions) {
            try {
                console.log(
                    `Page: Fetching details for ${currentImageInState.path}`,
                );
                const detailedData: ImageDetailData = await fetchImageDetails(
                    currentImageInState.path,
                );

                // Merge the detailed data into the existing image object in the array
                images[index] = { ...currentImageInState, ...detailedData };
                currentImageInState = images[index]; // Update local reference
                console.log(
                    `Page: Details loaded and merged for ${currentImageInState.name}`,
                    images[index],
                );
            } catch (err) {
                console.error("Page: Error fetching image details:", err);
                // Optionally set an error message for the user, though the modal will show what it has
                error = `Failed to load full details for ${currentImageInState.name}: ${err.message}`;
            }
        }

        // Ensure the image has a previewUrl for the editor
        if (!currentImageInState.previewUrl) {
            console.log('Page: Image missing previewUrl, generating it...');
            currentImageInState.previewUrl = convertFileSrc(currentImageInState.path);
        }

        // Ensure annotations array exists
        if (!currentImageInState.annotations) {
            currentImageInState.annotations = [];
        }

        // Open modal annotation viewer
        modalSelectedImage = currentImageInState;
        showAnnotationModal = true;

        console.log(`Page: Opening modal annotation viewer for ${currentImageInState.name}`);
        console.log('Page: Modal image object:', {
            name: currentImageInState.name,
            path: currentImageInState.path,
            previewUrl: currentImageInState.previewUrl,
            hasPreviewUrl: !!currentImageInState.previewUrl,
            annotations: currentImageInState.annotations?.length || 0,
            dimensions: currentImageInState.dimensions
        });
    }

    function closeImageView() {
        selectedImage = null;
    }

    // Modal Annotation Viewer Handlers
    function handleModalSave(event: CustomEvent) {
        const { image, annotations } = event.detail;
        console.log('Page: Saving annotations from modal for', image.name, annotations);

        // Update the image in the images array with new annotations
        const imageIndex = images.findIndex(img => img.path === image.path);
        if (imageIndex !== -1) {
            images[imageIndex] = { ...images[imageIndex], annotations };
        }

        // Close modal
        handleModalClose();
    }

    function handleModalClose() {
        console.log('Page: Closing annotation modal');

        // Close modal and reset state
        showAnnotationModal = false;
        modalSelectedImage = null;
    }

    // Handle view mode change
    function changeViewMode(mode) {
        viewMode = mode;
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
                `Page: Starting auto-annotation (all types) for images in directory: ${directoryPath}`,
            );
            // Params sent to service are based on what the backend command `auto_annotate_images` expects.
            // Backend now returns all annotation types (rectangles and polygons) in a single call.
            const result: AutoAnnotationResult = await performAutoAnnotation(
                directoryPath,
                currentPage,
                pageSize,
            );

            console.log("Page: Annotation service result:", result);

            if (
                result &&
                result.annotated_images &&
                result.annotated_images.length > 0
            ) {
                // Merge annotation data with existing images
                images = images.map((img: ProcessedImage) => {
                    const annotatedImgData: AnnotatedImageInfo | undefined =
                        result.annotated_images.find(
                            (ai) => ai.path === img.path,
                        );
                    if (annotatedImgData) {
                        console.log(
                            `Page: Updating annotations for ${img.name} from service data.`,
                        );
                        return {
                            ...img,
                            annotations:
                                annotatedImgData.annotations || img.annotations, // Fallback to existing if new is undefined
                            annotated: !!(
                                annotatedImgData.has_json &&
                                annotatedImgData.annotations &&
                                annotatedImgData.annotations.length > 0
                            ),
                            hasJson:
                                annotatedImgData.has_json !== undefined
                                    ? annotatedImgData.has_json
                                    : img.hasJson,
                        };
                    }
                    return img;
                });

                const imagesWithJson = result.annotated_images.filter(
                    (img) => img.has_json,
                ).length;
                const imagesWithActualAnnotations =
                    result.annotated_images.filter(
                        (img) => img.annotations && img.annotations.length > 0,
                    ).length;

                console.log(
                    `Page: From annotation result - ${imagesWithJson} images have JSON, ${imagesWithActualAnnotations} have actual annotations. Images array updated.`,
                );
                // Optionally, re-generate dataset summary if annotations changed significantly
                // await generateLabelMeSummary();
            } else {
                console.log(
                    "Page: No new annotations found or returned from service.",
                );
            }
        } catch (err) {
            console.error("Page: Error annotating images:", err);
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

    // Function to automatically load annotations for the current page
    async function autoLoadAnnotationsForPage(page: number) {
        try {
            autoAnnotating = true;
            console.log(`Auto-loading ALL annotations (bounding boxes + polygons) for page ${page}`);

            // Load all annotation types in a single call
            const result = await performAutoAnnotation(directoryPath, page, pageSize);

            console.log("Annotation result:", result);

            // Use result directly (no merging needed)
            const mergedAnnotatedImages = result.annotated_images;

            if (mergedAnnotatedImages && mergedAnnotatedImages.length > 0) {
                // Merge annotation data with existing images
                images = images.map((img: ProcessedImage) => {
                    const annotatedImgData: AnnotatedImageInfo | undefined =
                        mergedAnnotatedImages.find(
                            (ai) => ai.path === img.path,
                        );
                    if (annotatedImgData) {
                        console.log(
                            `Auto-updating annotations for ${img.name} (all types loaded in single call).`,
                        );

                        // Combine existing annotations with new ones (avoid duplicates)
                        const existingAnnotations = img.annotations || [];
                        const newAnnotations = annotatedImgData.annotations || [];
                        const combinedAnnotations = [...existingAnnotations];

                        // Add new annotations, avoiding duplicates by checking label and shape_type
                        newAnnotations.forEach(newAnn => {
                            const exists = combinedAnnotations.some(existing =>
                                existing.label === newAnn.label &&
                                existing.shape_type === newAnn.shape_type &&
                                JSON.stringify(existing.points) === JSON.stringify(newAnn.points)
                            );
                            if (!exists) {
                                combinedAnnotations.push(newAnn);
                            }
                        });

                        return {
                            ...img,
                            annotations: combinedAnnotations,
                            annotated: combinedAnnotations.length > 0,
                            hasJson:
                                annotatedImgData.has_json !== undefined
                                    ? annotatedImgData.has_json
                                    : img.hasJson,
                        };
                    }
                    return img;
                });

                const imagesWithJson = mergedAnnotatedImages.filter(
                    (img) => img.has_json,
                ).length;
                const imagesWithActualAnnotations =
                    mergedAnnotatedImages.filter(
                        (img) => img.annotations && img.annotations.length > 0,
                    ).length;
                const totalAnnotationsLoaded = mergedAnnotatedImages.reduce(
                    (total, img) => total + (img.annotations?.length || 0), 0
                );

                console.log(
                    `Auto-loaded ALL annotations: ${imagesWithJson} images have JSON, ${imagesWithActualAnnotations} have annotations, ${totalAnnotationsLoaded} total annotations loaded.`,
                );
            } else {
                console.log("Auto-annotation: No annotations found for either bounding boxes or polygons.");
            }
        } catch (autoErr) {
            console.warn("Auto-annotation failed (non-blocking):", autoErr);
            // Don't set global error - auto-loading is optional enhancement
            // Users can still manually load annotations if needed
        } finally {
            autoAnnotating = false;
        }
    }

    // Function to generate LabelMe summary for the dataset
    async function generateLabelMeSummary() {
        try {
            if (!directoryPath) {
                // Check directoryPath directly, images.length check removed as summary can be generated even for empty dir
                console.warn(
                    "Page: Directory path not set, cannot generate LabelMe summary.",
                );
                return;
            }

            pageExportError = ""; // Clear any previous export errors
            pageExportSuccess = "";

            console.log(
                `Page: Generating LabelMe summary for directory: ${directoryPath}`,
            );

            const summaryData = await fetchDatasetSummary(directoryPath);
            datasetSummary = summaryData;
            console.log("Page: Dataset summary updated:", datasetSummary);
        } catch (err) {
            console.error("Page: Error generating LabelMe summary:", err);
            const errorMessage =
                err?.message ||
                (typeof err === "string" ? err : "Unknown error");
            // Display a less intrusive warning for summary generation failure,
            // as it might not be critical path for all users.
            console.warn(
                `Page: Failed to generate LabelMe summary: ${errorMessage}`,
            );
            datasetSummary = null; // Clear summary on error
            // error = `Failed to generate LabelMe summary: ${errorMessage}`; // Optionally set global error
        }
    }

    async function runUnifiedExport(exportDetails: any) {
        // Modified to accept details from event
        const {
            sourceDir,
            outputDir,
            mode,
            trainRatio,
            valRatio,
            testRatio,
            shapeType,
            includedLabels,
        } = exportDetails;

        if (!sourceDir) {
            pageExportError =
                "Source directory is missing. Please ensure it's selected.";
            return;
        }
        if (!outputDir) {
            pageExportError =
                "Output directory is missing. Please select an output location.";
            return;
        }
        if (!includedLabels || includedLabels.length === 0) {
            pageExportError =
                "No labels selected for export. Please select at least one label in the modal.";
            return;
        }

        pageExportLoading = true;
        pageExportError = "";
        pageExportSuccess = "";

        try {
            // Construct parameters for the service function
            const params: DatasetExportParams = {
                sourceDir,
                outputDir,
                mode,
                trainRatio,
                valRatio,
                testRatio,
                shapeType,
                includedLabels,
            };

            console.log(
                `Page: Calling performDatasetExport with params:`,
                params,
            );
            const resultMessage = await performDatasetExport(params);

            pageExportSuccess = resultMessage;
            console.log(`Page: Export successful. Message: ${resultMessage}`);

            if (pageExportSuccess) {
                showActualExportModal = false; // Close modal on successful export
            }
        } catch (err) {
            console.error(
                `Page: Error during unified export (mode ${mode}):`,
                err,
            );
            const errMsg =
                err?.message ||
                (typeof err === "string" ? err : "Unknown error during export");
            pageExportError = `Failed to export dataset: ${errMsg}`;
            pageExportSuccess = ""; // Clear success message on error
        } finally {
            pageExportLoading = false;
        }
    }

    // --- Crop and Remap Tool Functions --- (Removed, now internal to CropRemapTool.svelte)
    // async function selectCropDirectory(type: "source" | "output") { ... }
    // async function runCropAndRemap() { ... }
    // async function handleCropCompleted(event: CustomEvent<{ outputDir: string }>) { ... }

</script>

<svelte:head>
    <title>Dataset Gallery</title>
    <meta
        name="description"
        content="Efficient image viewer for large image collections"
    />
</svelte:head>

<svelte:window on:keydown={handleKeydown} />

<div class="container mx-auto px-4 py-8">
    <div class="max-w-6xl mx-auto">
        <div class="mb-8">
            <h1 class="text-3xl font-bold text-slate-800 mb-6">Dataset Gallery</h1>

            <div class="flex flex-wrap items-center gap-4 mb-6">
                <!-- Controls that appear *after* directory selection -->
                {#if directoryPath}
                    <!-- Button to select/change directory (now appears only *after* initial selection) -->
                    <button
                        on:click={selectDirectory}
                        class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                        disabled={loading}
                    >
                        {loading ? "Loading..." : "Change Directory"}
                        <!-- Changed text slightly -->
                    </button>

                    <!-- Directory Path Display -->
                    <div class="text-sm text-slate-600 flex-1 truncate">
                        <span class="font-medium">Directory:</span>
                        {directoryPath}
                    </div>

                    {#if images.length > 0} 
                        <!-- Annotation controls -->
                        <div class="flex items-center gap-2">
                            <select
                                bind:value={annotationType}
                                class="inline-flex items-center bg-white/80 backdrop-blur border border-slate-300 rounded-md shadow-sm py-2 pl-3 pr-8 text-sm font-medium transition-colors duration-150 ease-in-out focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 focus:border-indigo-500 hover:border-slate-400 appearance-none"
                            >
                                <option value="bounding_box"
                                    >Bounding Boxes</option
                                >
                                <option value="polygon">Polygons</option>
                            </select>

                            <!-- Auto-annotation toggle -->
                            <div class="flex items-center gap-2 px-3 py-2 bg-white/80 backdrop-blur border border-slate-300 rounded-md">
                                <input
                                    type="checkbox"
                                    id="autoAnnotate"
                                    bind:checked={autoAnnotationEnabled}
                                    class="rounded border-slate-300 text-indigo-600 focus:ring-indigo-500"
                                />
                                <label for="autoAnnotate" class="text-sm text-slate-600 cursor-pointer">
                                    Auto-load All
                                </label>
                            </div>

                            <button
                                on:click={annotateImages}
                                class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 transition-colors"
                                disabled={annotating || autoAnnotating}
                            >
                                {annotating
                                    ? "Annotating..."
                                    : "Load Annotations"}
                            </button>

                            <!-- Unified Export Button -->
                            <button
                                on:click={() => {
                                    showActualExportModal = true;
                                    pageExportError = "";
                                    pageExportSuccess = "";
                                }}
                                class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
                                disabled={!directoryPath || images.length === 0}
                            >
                                Export Dataset
                            </button>

                        </div>
                        
                        <!-- View Mode Controls -->
                        <div class="flex items-center space-x-3 ml-auto">
                            <span class="text-sm text-slate-600">View:</span>
                            <button
                                class={`px-3 py-1 rounded text-sm transition-colors ${viewMode === "grid" ? "bg-indigo-100 text-indigo-700" : "text-slate-700 hover:bg-slate-100"}`}
                                on:click={() => changeViewMode("grid")}
                            >
                                Grid
                            </button>
                            <button
                                class={`px-3 py-1 rounded text-sm transition-colors ${viewMode === "column" ? "bg-indigo-100 text-indigo-700" : "text-slate-700 hover:bg-slate-100"}`}
                                on:click={() => changeViewMode("column")}
                            >
                                Column
                            </button>
                        </div>
                    {/if}
                {/if}
                <!-- End of #if directoryPath -->
            </div>


            {#if error}
                <div class="bg-red-50/80 backdrop-blur text-red-800 border border-red-200 p-4 rounded-md mb-6">
                    {error}
                </div>
            {/if}

            <!-- New Layout Structure: Summary -> Annotation Editor -> Gallery -->
            {#if directoryPath}
                <!-- 1. Dataset Summary Card (always show when directory is loaded) -->
                <div class="mb-8">
                    <DatasetSummaryCard {datasetSummary} />
                </div>

                <!-- 2. Modal Annotation Viewer (opens when image is selected) -->
                <!-- Modal is now rendered at the bottom of the page -->

                <!-- 3. Image Gallery and Loading States -->
                {#if loading && images.length === 0}
                    <div class="flex justify-center items-center py-12">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
                    </div>
                {:else if annotating}
                    <div class="flex flex-col justify-center items-center py-12">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-green-600 mb-4"></div>
                        <p class="text-gray-600">
                            Loading {annotationType} annotations, please wait...
                        </p>
                        <p class="text-xs text-gray-500 mt-1">
                            Processing existing annotations from JSON files
                        </p>
                    </div>
                {:else if autoAnnotating}
                    <div class="flex flex-col justify-center items-center py-12">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mb-4"></div>
                        <p class="text-gray-600">
                            Automatically loading ALL annotations...
                        </p>
                        <p class="text-xs text-gray-500 mt-1">
                            Loading bounding boxes and polygons • This may take a moment for large images
                        </p>
                    </div>
                {:else if !loading && images.length === 0 && !error}
                    <div class="text-center py-12">
                        <p class="text-gray-500">No images found in this directory.</p>
                    </div>
                {:else if images.length > 0}
                    <!-- Image count display -->
                    <div class="text-sm text-slate-600 mb-4">
                        Showing {Math.min(images.length, totalImages)} of {totalImages} images
                    </div>

                    <!-- Image Gallery Component -->
                    <ImageGallery
                        bind:images
                        {totalImages}
                        bind:currentPage
                        {totalPages}
                        {pageSize}
                        {viewMode}
                        {annotationType}
                        {loading}
                        {loadingMore}
                        on:selectImage={(event) => selectImage(event.detail)}
                        on:loadPage={(event) => loadImagesPage(event.detail)}
                    />
                {/if}
            {:else if !loading && !error}
                <!-- Enhanced Placeholder Area when no directory is selected -->
                <div class="text-center py-16 px-6 border-2 border-dashed border-gray-300 rounded-lg mt-8">
                    <svg
                        class="mx-auto h-12 w-12 text-gray-400"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        aria-hidden="true"
                    >
                        <path
                            vector-effect="non-scaling-stroke"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z"
                        />
                    </svg>
                    <h3 class="mt-2 text-lg font-medium text-gray-900">
                        No Directory Selected
                    </h3>
                    <p class="mt-1 text-sm text-gray-500">
                        Select a directory containing your images and LabelMe (.json) annotations to begin viewing and editing your dataset.
                    </p>
                    <div class="mt-6">
                        <button
                            type="button"
                            on:click={selectDirectory}
                            class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                class="-ml-1 mr-2 h-5 w-5"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                                stroke-width="2"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                                />
                            </svg>
                            Select Directory
                        </button>
                    </div>
                </div>
            {/if}
        </div>
    </div>
</div>

<!-- Image Viewer Modal with Annotation Support -->
<ImageViewerModal {selectedImage} {annotationType} on:close={closeImageView} />

<!-- Unified Export Modal -->
<ExportModal
    bind:showModal={showActualExportModal}
    currentDirectoryPath={directoryPath}
    currentDatasetSummary={datasetSummary}
    on:closeModal={() => {
        showActualExportModal = false;
    }}
    on:runExport={(event) => runUnifiedExport(event.detail)}
/>

<!-- Modal Annotation Viewer -->
<ModalAnnotationViewer
    showModal={showAnnotationModal}
    selectedImage={modalSelectedImage}
    {autoAnnotationEnabled}
    on:save={handleModalSave}
    on:close={handleModalClose}
/>

<style>
</style>

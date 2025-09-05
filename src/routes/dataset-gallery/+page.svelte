<script lang="ts">
    import { onMount, tick } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    // import { convertFileSrc } from "@tauri-apps/api/core"; // No longer needed directly here
    import { Accordion, AccordionItem } from "flowbite-svelte";
    import DatasetSummaryCard from "$lib/DatasetSummaryCard.svelte";
    import CropRemapTool from "$lib/CropRemapTool.svelte";
    import ImageGallery from "$lib/ImageGallery.svelte";
    import ImageViewerModal from "$lib/ImageViewerModal.svelte";
    import ExportModal from "$lib/ExportModal.svelte";
    import {
        fetchPaginatedImages,
        fetchImageDetails,
        fetchDatasetSummary,
        performAutoAnnotation,
        performCropAndRemap,
        performDatasetExport,
    } from "$lib/services/datasetService";
    import type {
        ProcessedImage,
        ImageDetailData,
        DatasetSummary,
        AutoAnnotationResult,
        AnnotatedImageInfo,
        DatasetExportParams,
    } from "$lib/services/datasetService";

    // State variables
    let directoryPath = "";
    let images: ProcessedImage[] = []; // Use the ProcessedImage type
    let loading = false;
    let loadingMore = false;
    let error = "";
    let viewMode = "grid"; // 'grid' or 'column'
    let selectedImage: ProcessedImage | null = null; // Ensure selectedImage also uses ProcessedImage
    let annotating = false; // Tracks annotation status
    let annotationType = "bounding_box"; // Default annotation type
    let datasetSummary: DatasetSummary | null = null; // Use the DatasetSummary type

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
    let cropToolOpen = false; // State for Accordion (remains for parent control if needed)

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

        selectedImage = currentImageInState; // Set the potentially updated image as selected

        // The ImageViewerModal component will handle drawing annotations
        // using its own `afterUpdate` or `on:load` logic when `selectedImage` prop changes.
    }

    function closeImageView() {
        selectedImage = null;
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
                `Page: Starting ${annotationType} annotation for images in directory: ${directoryPath}`,
            );
            // Params sent to service are based on what the backend command `auto_annotate_images` expects.
            // If it operates on the whole directory, page/pageSize might be less relevant here
            // or used by backend to know which part of a potentially larger dataset is currently in view.
            const result: AutoAnnotationResult = await performAutoAnnotation(
                directoryPath,
                currentPage,
                pageSize,
                annotationType,
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

    async function handleCropCompleted(event: CustomEvent<{ outputDir: string }>) {
        const { outputDir } = event.detail;
        console.log(`Page: Crop & Remap completed. New output directory: ${outputDir}`);

        // Clear any existing page-level errors or success messages
        error = "";
        pageExportError = "";
        pageExportSuccess = "";
        // The CropRemapTool will show its own status message, 
        // so we don't need to set one on the main page for the crop operation itself.

        directoryPath = outputDir;
        currentPage = 1;
        images = [];
        datasetSummary = null;
        selectedImage = null;

        // Close the crop tool accordion if it's open
        cropToolOpen = false; 

        try {
            await loadImagesPage(1); // Load images from the new directory
        } catch (loadErr) {
            console.error("Page: Error loading images after crop:", loadErr);
            const errMsg =
                loadErr?.message || String(loadErr) || "Unknown error";
            error = `Failed to load images from cropped directory: ${errMsg}`;
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
                        {loading ? "Loading..." : "Change Directory"}
                        <!-- Changed text slightly -->
                    </button>

                    <!-- Directory Path Display -->
                    <div class="text-sm text-gray-600 flex-1 truncate">
                        <span class="font-medium">Directory:</span>
                        {directoryPath}
                    </div>

                    {#if images.length > 0} 
                        <!-- Annotation controls -->
                        <div class="flex items-center gap-2">
                            <select
                                bind:value={annotationType}
                                class="inline-flex items-center bg-white border border-gray-300 rounded-md shadow-sm py-2 pl-3 pr-8 text-sm font-medium transition-colors duration-150 ease-in-out focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 focus:border-indigo-500 hover:border-gray-400 appearance-none"
                            >
                                <option value="bounding_box"
                                    >Bounding Boxes</option
                                >
                                <option value="polygon">Polygons</option>
                            </select>

                            <button
                                on:click={annotateImages}
                                class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
                                disabled={annotating}
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
                                class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                                disabled={!directoryPath || images.length === 0}
                            >
                                Export Dataset
                            </button>
                        </div>
                        
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
                {/if}
                <!-- End of #if directoryPath -->
            </div>

            <!-- Use the Crop and Remap Tool Component -->
            <CropRemapTool
                bind:cropToolOpen
                on:cropCompleted={handleCropCompleted}
            />

            {#if error}
                <div class="bg-red-50 text-red-700 p-4 rounded-md mb-6">
                    {error}
                </div>
            {/if}

            <!-- Use the Dataset Summary Card Component -->
            <DatasetSummaryCard {datasetSummary} />

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
            <div
                class="text-center py-16 px-6 border-2 border-dashed border-gray-300 rounded-lg mt-8"
            >
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
                    Select a directory containing your images and LabelMe
                    (.json) annotations to begin.
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
                <p class="mt-4 text-sm text-gray-500">
                    Alternatively, you can use the Crop & Remap tool above to
                    process a dataset.
                </p>
            </div>
        {:else if images.length > 0}
            <!-- Integrate ImageGallery component -->
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

<style>
</style>

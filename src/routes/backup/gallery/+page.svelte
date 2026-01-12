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
            console.log("Page: Image missing previewUrl, generating it...");
            currentImageInState.previewUrl = convertFileSrc(
                currentImageInState.path,
            );
        }

        // Ensure annotations array exists
        if (!currentImageInState.annotations) {
            currentImageInState.annotations = [];
        }

        // Open modal annotation viewer
        modalSelectedImage = currentImageInState;
        showAnnotationModal = true;

        console.log(
            `Page: Opening modal annotation viewer for ${currentImageInState.name}`,
        );
        console.log("Page: Modal image object:", {
            name: currentImageInState.name,
            path: currentImageInState.path,
            previewUrl: currentImageInState.previewUrl,
            hasPreviewUrl: !!currentImageInState.previewUrl,
            annotations: currentImageInState.annotations?.length || 0,
            dimensions: currentImageInState.dimensions,
        });
    }

    function closeImageView() {
        selectedImage = null;
    }

    // Modal Annotation Viewer Handlers
    function handleModalSave(event: CustomEvent) {
        const { image, annotations } = event.detail;
        console.log(
            "Page: Saving annotations from modal for",
            image.name,
            annotations,
        );

        // Update the image in the images array with new annotations
        const imageIndex = images.findIndex((img) => img.path === image.path);
        if (imageIndex !== -1) {
            images[imageIndex] = { ...images[imageIndex], annotations };
        }

        // Close modal
        handleModalClose();
    }

    function handleModalClose() {
        console.log("Page: Closing annotation modal");

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
            console.log(
                `Auto-loading ALL annotations (bounding boxes + polygons) for page ${page}`,
            );

            // Load all annotation types in a single call
            const result = await performAutoAnnotation(
                directoryPath,
                page,
                pageSize,
            );

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
                        const newAnnotations =
                            annotatedImgData.annotations || [];
                        const combinedAnnotations = [...existingAnnotations];

                        // Add new annotations, avoiding duplicates by checking label and shape_type
                        newAnnotations.forEach((newAnn) => {
                            const exists = combinedAnnotations.some(
                                (existing) =>
                                    existing.label === newAnn.label &&
                                    existing.shape_type === newAnn.shape_type &&
                                    JSON.stringify(existing.points) ===
                                        JSON.stringify(newAnn.points),
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
                    (total, img) => total + (img.annotations?.length || 0),
                    0,
                );

                console.log(
                    `Auto-loaded ALL annotations: ${imagesWithJson} images have JSON, ${imagesWithActualAnnotations} have annotations, ${totalAnnotationsLoaded} total annotations loaded.`,
                );
            } else {
                console.log(
                    "Auto-annotation: No annotations found for either bounding boxes or polygons.",
                );
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
            <h1 class="text-3xl font-bold text-base-content mb-6">
                Dataset Gallery
            </h1>

            <!-- Navbar Toolbar -->
            <div
                class="navbar bg-base-100 rounded-box border border-base-300 mb-6 py-3 px-4 gap-4"
            >
                <!-- Section 1: Directory Selection -->
                <div
                    class="bg-base-200 rounded-lg px-3 py-2 border border-base-300"
                >
                    <div class="flex items-center gap-3">
                        <button
                            on:click={selectDirectory}
                            class="btn btn-primary btn-sm gap-2"
                            disabled={loading}
                        >
                            {#if loading}
                                <span class="loading loading-spinner loading-sm"
                                ></span>
                            {:else}
                                <span class="material-symbols-rounded text-lg"
                                    >folder_open</span
                                >
                            {/if}
                            {loading
                                ? "Loading..."
                                : directoryPath
                                  ? "Change"
                                  : "Select"}
                        </button>

                        {#if directoryPath}
                            <div
                                class="text-sm text-base-content/70 truncate max-w-[200px]"
                            >
                                {directoryPath.split("/").slice(-2).join("/")}
                            </div>
                        {:else}
                            <div class="text-sm text-base-content/50 italic">
                                No folder
                            </div>
                        {/if}
                    </div>
                </div>

                <!-- Section 2: Annotation Options -->
                <div
                    class="bg-base-200 rounded-lg px-3 py-2 border border-base-300 hidden lg:block"
                >
                    <div class="flex items-center gap-4">
                        <div class="join">
                            <button
                                class={`btn btn-sm join-item gap-1 ${annotationType === "bounding_box" ? "btn-active btn-primary" : "btn-ghost"}`}
                                on:click={() =>
                                    (annotationType = "bounding_box")}
                                disabled={!directoryPath || images.length === 0}
                                title="Bounding Boxes"
                            >
                                <span class="material-symbols-rounded text-lg"
                                    >crop_square</span
                                >
                                <span class="hidden xl:inline">Boxes</span>
                            </button>
                            <button
                                class={`btn btn-sm join-item gap-1 ${annotationType === "polygon" ? "btn-active btn-primary" : "btn-ghost"}`}
                                on:click={() => (annotationType = "polygon")}
                                disabled={!directoryPath || images.length === 0}
                                title="Polygons"
                            >
                                <span class="material-symbols-rounded text-lg"
                                    >hexagon</span
                                >
                                <span class="hidden xl:inline">Polygon</span>
                            </button>
                        </div>

                        <div class="divider divider-horizontal mx-0"></div>

                        <label class="label cursor-pointer gap-2">
                            <span class="text-sm">Auto-load</span>
                            <input
                                type="checkbox"
                                bind:checked={autoAnnotationEnabled}
                                class="toggle toggle-primary toggle-sm"
                                disabled={!directoryPath}
                            />
                        </label>
                    </div>
                </div>

                <!-- Spacer -->
                <div class="flex-1"></div>

                <!-- Section 3: Actions -->
                <div
                    class="bg-base-200 rounded-lg px-3 py-2 border border-base-300"
                >
                    <div class="flex items-center gap-2">
                        <button
                            on:click={annotateImages}
                            class="btn btn-ghost btn-sm gap-1"
                            disabled={!directoryPath ||
                                images.length === 0 ||
                                annotating ||
                                autoAnnotating}
                            title="Load Annotations"
                        >
                            {#if annotating}
                                <span class="loading loading-spinner loading-sm"
                                ></span>
                            {:else}
                                <span class="material-symbols-rounded text-lg"
                                    >label</span
                                >
                            {/if}
                            <span class="hidden sm:inline text-sm">Load</span>
                        </button>

                        <button
                            on:click={() => {
                                showActualExportModal = true;
                                pageExportError = "";
                                pageExportSuccess = "";
                            }}
                            class="btn btn-ghost btn-sm gap-1"
                            disabled={!directoryPath || images.length === 0}
                            title="Export Dataset"
                        >
                            <span class="material-symbols-rounded text-lg"
                                >download</span
                            >
                            <span class="hidden sm:inline text-sm">Export</span>
                        </button>
                    </div>
                </div>

                <!-- Section 4: View Mode -->
                <div class="bg-base-200 rounded-lg p-1 border border-base-300">
                    <div class="join">
                        <button
                            class={`btn btn-sm join-item ${viewMode === "grid" ? "btn-active btn-primary" : "btn-ghost"}`}
                            on:click={() => changeViewMode("grid")}
                            disabled={!directoryPath}
                            title="Grid View"
                        >
                            <span class="material-symbols-rounded text-lg"
                                >grid_view</span
                            >
                        </button>
                        <button
                            class={`btn btn-sm join-item ${viewMode === "column" ? "btn-active btn-primary" : "btn-ghost"}`}
                            on:click={() => changeViewMode("column")}
                            disabled={!directoryPath}
                            title="Column View"
                        >
                            <span class="material-symbols-rounded text-lg"
                                >view_agenda</span
                            >
                        </button>
                    </div>
                </div>
            </div>

            {#if error}
                <div class="alert alert-error mb-6">
                    <span class="material-symbols-rounded">error</span>
                    <span>{error}</span>
                </div>
            {/if}

            <!-- Main Content Area - Always Visible -->
            <!-- 1. Dataset Summary Card - Always show with empty state -->
            <div class="mb-8">
                <DatasetSummaryCard {datasetSummary} />
            </div>

            <!-- 2. Image Gallery and Loading States -->
            {#if loading && images.length === 0}
                <div class="flex justify-center items-center py-12">
                    <span
                        class="loading loading-spinner loading-lg text-primary"
                    ></span>
                </div>
            {:else if annotating}
                <div class="flex flex-col justify-center items-center py-12">
                    <span
                        class="loading loading-spinner loading-lg text-success mb-4"
                    ></span>
                    <p class="text-base-content">
                        Loading {annotationType} annotations, please wait...
                    </p>
                    <p class="text-xs text-base-content/60 mt-1">
                        Processing existing annotations from JSON files
                    </p>
                </div>
            {:else if autoAnnotating}
                <div class="flex flex-col justify-center items-center py-12">
                    <span
                        class="loading loading-spinner loading-lg text-info mb-4"
                    ></span>
                    <p class="text-base-content">
                        Automatically loading ALL annotations...
                    </p>
                    <p class="text-xs text-base-content/60 mt-1">
                        Loading bounding boxes and polygons • This may take a
                        moment for large images
                    </p>
                </div>
            {:else if !directoryPath}
                <!-- Empty state when no directory is selected -->
                <div class="card bg-base-200/50 border border-base-300">
                    <div class="card-body items-center text-center py-12">
                        <span
                            class="material-symbols-rounded text-5xl text-base-content/30"
                            >photo_library</span
                        >
                        <p class="text-base-content/50 mt-2">
                            Select a directory to load images
                        </p>
                    </div>
                </div>
            {:else if !loading && images.length === 0 && !error}
                <div class="text-center py-12">
                    <p class="text-base-content/60">
                        No images found in this directory.
                    </p>
                </div>
            {:else if images.length > 0}
                <!-- Image count display -->
                <div class="text-sm text-base-content/70 mb-4">
                    Showing {Math.min(images.length, totalImages)} of {totalImages}
                    images
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

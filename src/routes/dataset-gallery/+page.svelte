<script lang="ts">
    import { onMount, tick } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
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
    let images: ProcessedImage[] = [];
    let loading = false;
    let loadingMore = false;
    let error = "";
    let viewMode = "grid";
    let selectedImage: ProcessedImage | null = null;
    let annotating = false;
    let annotationType = "bounding_box";
    let datasetSummary: DatasetSummary | null = null;

    // Export Modal State
    let showActualExportModal = false;
    let pageExportLoading = false;
    let pageExportError = "";
    let pageExportSuccess = "";

    // Pagination
    let currentPage = 1;
    let pageSize = 30;
    let totalPages = 0;
    let totalImages = 0;

    // Crop Tool State
    let cropToolOpen = false;

    onMount(() => {});

    function handleKeydown(event) {
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
                currentPage = 1;
                images = [];

                if (!directoryPath.trim()) {
                    error = "Invalid directory path: path is empty";
                    loading = false;
                    return;
                }

                try {
                    await loadImagesPage(1);
                } catch (loadErr) {
                    const errMsg =
                        loadErr?.message || String(loadErr) || "Unknown error";
                    error = `Failed to load images: ${errMsg}`;
                }
            }
        } catch (err) {
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

            if (page === 1) {
                generateLabelMeSummary();
            }

            await tick();

            if (typeof window !== "undefined") {
                window.scrollTo(0, 0);
            }
        } catch (err) {
            const errorMessage =
                err?.message ||
                (typeof err === "string" ? err : "Unknown error");
            error = `Failed to load images: ${errorMessage}`;
            images = [];
            totalImages = 0;
            totalPages = 0;
        } finally {
            loading = false;
            loadingMore = false;
        }
    }

    async function selectImage(imageDetail: {
        image: ProcessedImage;
        index: number;
    }) {
        const { image, index } = imageDetail;
        let currentImageInState = images[index];

        if (!currentImageInState.dimensions) {
            try {
                const detailedData: ImageDetailData = await fetchImageDetails(
                    currentImageInState.path,
                );
                images[index] = { ...currentImageInState, ...detailedData };
                currentImageInState = images[index];
            } catch (err) {
                error = `Failed to load full details for ${currentImageInState.name}: ${err.message}`;
            }
        }

        selectedImage = currentImageInState;
    }

    function closeImageView() {
        selectedImage = null;
    }

    function changeViewMode(mode) {
        viewMode = mode;
    }

    async function annotateImages() {
        try {
            if (!directoryPath || images.length === 0) {
                error = "Please select a directory with images first";
                return;
            }

            annotating = true;
            error = "";

            const result: AutoAnnotationResult = await performAutoAnnotation(
                directoryPath,
                currentPage,
                pageSize,
            );

            if (result?.annotated_images?.length > 0) {
                images = images.map((img: ProcessedImage) => {
                    const annotatedImgData: AnnotatedImageInfo | undefined =
                        result.annotated_images.find(
                            (ai) => ai.path === img.path,
                        );
                    if (annotatedImgData) {
                        return {
                            ...img,
                            annotations:
                                annotatedImgData.annotations || img.annotations,
                            annotated: !!(
                                annotatedImgData.has_json &&
                                annotatedImgData.annotations?.length > 0
                            ),
                            hasJson:
                                annotatedImgData.has_json !== undefined
                                    ? annotatedImgData.has_json
                                    : img.hasJson,
                        };
                    }
                    return img;
                });
            }
        } catch (err) {
            const errorMessage =
                err?.message ||
                (typeof err === "string" ? err : "Unknown error");
            error = `Failed to annotate images: ${errorMessage}`;
        } finally {
            annotating = false;
        }
    }

    async function generateLabelMeSummary() {
        try {
            if (!directoryPath) return;

            pageExportError = "";
            pageExportSuccess = "";

            const summaryData = await fetchDatasetSummary(directoryPath);
            datasetSummary = summaryData;
        } catch (err) {
            datasetSummary = null;
        }
    }

    async function runUnifiedExport(exportDetails: any) {
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
            pageExportError = "Source directory is missing.";
            return;
        }
        if (!outputDir) {
            pageExportError = "Output directory is missing.";
            return;
        }
        if (!includedLabels || includedLabels.length === 0) {
            pageExportError = "No labels selected for export.";
            return;
        }

        pageExportLoading = true;
        pageExportError = "";
        pageExportSuccess = "";

        try {
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

            const resultMessage = await performDatasetExport(params);
            pageExportSuccess = resultMessage;

            if (pageExportSuccess) {
                showActualExportModal = false;
            }
        } catch (err) {
            const errMsg =
                err?.message ||
                (typeof err === "string" ? err : "Unknown error during export");
            pageExportError = `Failed to export dataset: ${errMsg}`;
            pageExportSuccess = "";
        } finally {
            pageExportLoading = false;
        }
    }

    async function handleCropCompleted(
        event: CustomEvent<{ outputDir: string }>,
    ) {
        const { outputDir } = event.detail;

        error = "";
        pageExportError = "";
        pageExportSuccess = "";

        directoryPath = outputDir;
        currentPage = 1;
        images = [];
        datasetSummary = null;
        selectedImage = null;
        cropToolOpen = false;

        try {
            await loadImagesPage(1);
        } catch (loadErr) {
            const errMsg =
                loadErr?.message || String(loadErr) || "Unknown error";
            error = `Failed to load images from cropped directory: ${errMsg}`;
        }
    }
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
        <!-- Header -->
        <div class="mb-8">
            <h1 class="text-3xl font-bold mb-6 flex items-center gap-3">
                <span class="material-symbols-rounded text-primary text-4xl"
                    >photo_library</span
                >
                Dataset Gallery
            </h1>

            {#if directoryPath}
                <!-- Controls Section -->
                <div class="flex flex-wrap items-center gap-3 mb-6">
                    <!-- Change Directory Button -->
                    <button
                        on:click={selectDirectory}
                        class="btn btn-primary btn-sm"
                        disabled={loading}
                    >
                        <span class="material-symbols-rounded icon-sm"
                            >folder_open</span
                        >
                        {loading ? "Loading..." : "Change Directory"}
                    </button>

                    <!-- Directory Path -->
                    <div
                        class="badge badge-lg badge-ghost flex-1 truncate max-w-md"
                    >
                        <span class="material-symbols-rounded icon-sm mr-1"
                            >folder</span
                        >
                        {directoryPath}
                    </div>

                    {#if images.length > 0}
                        <!-- Annotation Controls -->
                        <div class="join">
                            <select
                                bind:value={annotationType}
                                class="select select-sm select-bordered join-item"
                            >
                                <option value="bounding_box"
                                    >Bounding Boxes</option
                                >
                                <option value="polygon">Polygons</option>
                            </select>

                            <button
                                on:click={annotateImages}
                                class="btn btn-success btn-sm join-item"
                                disabled={annotating}
                            >
                                <span class="material-symbols-rounded icon-sm">
                                    {annotating
                                        ? "hourglass_empty"
                                        : "auto_fix_high"}
                                </span>
                                {annotating ? "Loading..." : "Load Annotations"}
                            </button>
                        </div>

                        <!-- Export Button -->
                        <button
                            on:click={() => {
                                showActualExportModal = true;
                                pageExportError = "";
                                pageExportSuccess = "";
                            }}
                            class="btn btn-info btn-sm"
                            disabled={!directoryPath || images.length === 0}
                        >
                            <span class="material-symbols-rounded icon-sm"
                                >upload</span
                            >
                            Export Dataset
                        </button>

                        <!-- Crop & Remap Toggle -->
                        <button
                            on:click={() => {
                                cropToolOpen = !cropToolOpen;
                                if (cropToolOpen) {
                                    setTimeout(() => {
                                        const element =
                                            document.querySelector(
                                                "[data-crop-tool]",
                                            );
                                        if (element)
                                            element.scrollIntoView({
                                                behavior: "smooth",
                                                block: "start",
                                            });
                                    }, 100);
                                }
                            }}
                            class="btn btn-secondary btn-sm"
                        >
                            <span class="material-symbols-rounded icon-sm"
                                >crop</span
                            >
                            {cropToolOpen ? "Hide" : "Show"} Crop & Remap
                        </button>

                        <!-- View Mode Toggle -->
                        <div class="join ml-auto">
                            <button
                                class="btn btn-sm join-item {viewMode === 'grid'
                                    ? 'btn-active'
                                    : ''}"
                                on:click={() => changeViewMode("grid")}
                            >
                                <span class="material-symbols-rounded icon-sm"
                                    >grid_view</span
                                >
                            </button>
                            <button
                                class="btn btn-sm join-item {viewMode ===
                                'column'
                                    ? 'btn-active'
                                    : ''}"
                                on:click={() => changeViewMode("column")}
                            >
                                <span class="material-symbols-rounded icon-sm"
                                    >view_list</span
                                >
                            </button>
                        </div>
                    {/if}
                </div>
            {/if}

            <!-- Crop and Remap Tool -->
            <CropRemapTool
                bind:cropToolOpen
                on:cropCompleted={handleCropCompleted}
            />

            <!-- Error Alert -->
            {#if error}
                <div class="alert alert-error mb-6">
                    <span class="material-symbols-rounded">error</span>
                    <span>{error}</span>
                    <button
                        class="btn btn-ghost btn-sm"
                        on:click={() => (error = "")}
                    >
                        <span class="material-symbols-rounded">close</span>
                    </button>
                </div>
            {/if}

            <!-- Dataset Summary Card -->
            <DatasetSummaryCard {datasetSummary} />

            <!-- Image Count -->
            {#if totalImages > 0}
                <div class="text-sm opacity-70 mb-4">
                    Showing {Math.min(images.length, totalImages)} of {totalImages}
                    images
                </div>
            {/if}
        </div>

        <!-- Content States -->
        {#if loading && images.length === 0}
            <div class="flex justify-center items-center py-12">
                <span class="loading loading-spinner loading-lg text-primary"
                ></span>
            </div>
        {:else if annotating}
            <div class="flex flex-col justify-center items-center py-12">
                <span
                    class="loading loading-spinner loading-lg text-success mb-4"
                ></span>
                <p class="opacity-70">
                    Loading {annotationType} annotations, please wait...
                </p>
            </div>
        {:else if !loading && directoryPath && images.length === 0 && !error}
            <div class="text-center py-12">
                <span class="material-symbols-rounded text-6xl opacity-30"
                    >image_not_supported</span
                >
                <p class="mt-4 opacity-70">
                    No images found in this directory.
                </p>
            </div>
        {:else if !loading && !directoryPath && !error}
            <!-- Empty State / Select Directory -->
            <div class="card bg-base-200 shadow-xl">
                <div class="card-body items-center text-center py-16">
                    <span
                        class="material-symbols-rounded text-6xl text-primary opacity-50"
                        >folder_open</span
                    >
                    <h2 class="card-title mt-4">No Directory Selected</h2>
                    <p class="opacity-70 max-w-md">
                        Select a directory containing your images and LabelMe
                        (.json) annotations to begin.
                    </p>
                    <div class="card-actions mt-6">
                        <button
                            class="btn btn-primary"
                            on:click={selectDirectory}
                        >
                            <span class="material-symbols-rounded"
                                >folder_open</span
                            >
                            Select Directory
                        </button>
                    </div>
                    <p class="text-sm opacity-50 mt-4">
                        Alternatively, you can use the Crop & Remap tool to
                        process a dataset.
                    </p>
                </div>
            </div>
        {:else if images.length > 0}
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

<!-- Image Viewer Modal -->
<ImageViewerModal {selectedImage} {annotationType} on:close={closeImageView} />

<!-- Export Modal -->
<ExportModal
    bind:showModal={showActualExportModal}
    currentDirectoryPath={directoryPath}
    currentDatasetSummary={datasetSummary}
    on:closeModal={() => {
        showActualExportModal = false;
    }}
    on:runExport={(event) => runUnifiedExport(event.detail)}
/>

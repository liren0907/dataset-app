<script lang="ts">
    import { onMount, tick } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import DatasetSummaryCard from "$lib/DatasetSummaryCard.svelte";
    import CropRemapTool from "$lib/CropRemapTool.svelte";
    import ImageGallery from "$lib/legacy-components/ImageGallery.svelte";
    import ImageViewerModal from "$lib/legacy-components/ImageViewerModal.svelte";
    import ExportModal from "$lib/legacy-components/ExportModal.svelte";

    // New Components
    import GalleryControls from "./components/GalleryControls.svelte";
    import EmptyState from "./components/EmptyState.svelte";
    import ErrorDisplay from "./components/ErrorDisplay.svelte";

    import {
        fetchPaginatedImages,
        fetchImageDetails,
        fetchDatasetSummary,
        performAutoAnnotation,
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
                directoryPath = selected as string;
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

    function toggleCropTool() {
        cropToolOpen = !cropToolOpen;
        if (cropToolOpen) {
            setTimeout(() => {
                const element = document.querySelector("[data-crop-tool]");
                if (element) {
                    element.scrollIntoView({
                        behavior: "smooth",
                        block: "start",
                    });
                }
            }, 100);
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
                <GalleryControls
                    {directoryPath}
                    {images}
                    {loading}
                    {annotating}
                    bind:annotationType
                    {cropToolOpen}
                    {viewMode}
                    on:selectDirectory={selectDirectory}
                    on:annotate={annotateImages}
                    on:export={() => {
                        showActualExportModal = true;
                        pageExportError = "";
                        pageExportSuccess = "";
                    }}
                    on:toggleCrop={toggleCropTool}
                    on:changeViewMode={(e) => changeViewMode(e.detail)}
                />
            {/if}

            <!-- Crop and Remap Tool -->
            <CropRemapTool
                bind:cropToolOpen
                on:cropCompleted={handleCropCompleted}
            />

            <ErrorDisplay {error} on:close={() => (error = "")} />

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
            <EmptyState on:selectDirectory={selectDirectory} />
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

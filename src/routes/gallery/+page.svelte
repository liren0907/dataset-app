<script lang="ts">
    import { onMount } from "svelte";
    import DatasetSummary from "$lib/components/gallery/DatasetSummary.svelte";
    import ImagePreviewPanel from "$lib/components/gallery/ImagePreviewPanel.svelte";
    import ImageGallery from "$lib/components/gallery/ImageGallery.svelte";
    import ExportModal from "$lib/components/gallery/ExportModal.svelte";
    import ModalAnnotationViewer from "$lib/components/gallery/ModalAnnotationViewer.svelte";
    import CropRemapTool from "$lib/components/gallery/CropRemapTool.svelte";
    import AdvancedCropRemapTool from "$lib/components/gallery/AdvancedCropRemapTool.svelte";
    import HierarchicalCrop from "$lib/components/gallery/HierarchicalCrop.svelte";
    import GalleryNavbar from "$lib/components/gallery/GalleryNavbar.svelte";
    import GalleryEmptyState from "$lib/components/gallery/GalleryEmptyState.svelte";
    import CroppedDatasetCard from "$lib/components/gallery/CroppedDatasetCard.svelte";
    import CroppedDatasetSummary from "$lib/components/gallery/CroppedDatasetSummary.svelte";
    import CroppedDatasetPreviewModal from "$lib/components/gallery/CroppedDatasetPreviewModal.svelte";
    import KonvaViewer from "$lib/components/gallery/KonvaViewer.svelte";
    import { IconButton, Toast } from "$lib/components/ui";
    import { confirm as tauriConfirm } from "@tauri-apps/plugin-dialog";
    import { generateAnnotatedPreviews } from "$lib/services/gallery/datasetService";
    import type { KonvaImageData } from "$lib/services/gallery/konvaService";

    // Import runes-based stores
    import { imageState, setMockMode, selectDirectory, loadImagesPage, generateLabelMeSummary } from "$lib/stores/gallery/imageStore.svelte";
    import { uiState, resetSelection } from "$lib/stores/gallery/uiStore.svelte";
    import { annotationState, annotateImages, autoLoadAnnotationsForPage } from "$lib/stores/gallery/annotationStore.svelte";
    import { exportState, runUnifiedExport, handleCropCompleted, openCroppedDatasetInGallery, removeCroppedDataset, clearAllCroppedDatasets, openCropModalWithLabel, closeCropModal, switchToOriginal, runCropInBackground, validateCroppedDatasets } from "$lib/stores/gallery/exportStore.svelte";

    const PREVIEW_SAMPLE_COUNT = 8;

    let showCroppedPreviewModal = $state(false);
    let croppedPreviewLoading = $state(false);
    let croppedPreviewError = $state("");
    let croppedPreviewImages: KonvaImageData[] = $state([]);
    let croppedPreviewOutputPath = $state("");
    let selectedPreviewImage: KonvaImageData | null = $state(null);
    let reopenPreviewAfterKonva = $state(false);
    const previewCache: Map<string, KonvaImageData[]> = new Map();

    // --- Helper: Trigger Auto-Annotation ---
    async function triggerAutoAnnotationIfNeeded(
        page: number = imageState.currentPage,
    ) {
        if (
            annotationState.autoAnnotationEnabled &&
            imageState.images.length > 0
        ) {
            await autoLoadAnnotationsForPage(page);
        }
    }

    onMount(async () => {
        const isTauri = typeof window !== "undefined" && "__TAURI__" in window;
        if (!isTauri) {
            console.log("Browser environment detected (No Tauri).");
            await setMockMode(true);
        } else {
            await validateCroppedDatasets();
        }
    });

    // Keyboard shortcuts
    function handleKeydown(event: KeyboardEvent) {
        if (event.key === "Escape" && uiState.selectedImage) {
            uiState.selectedImage = null;
            uiState.showAnnotationModal = false;
        }
    }

    async function handlePageChange(page: number) {
        await loadImagesPage(page);
        await triggerAutoAnnotationIfNeeded(page);
    }

    async function openCroppedPreview(outputPath: string) {
        croppedPreviewOutputPath = outputPath;
        croppedPreviewError = "";
        showCroppedPreviewModal = true;
        selectedPreviewImage = null;

        const cached = previewCache.get(outputPath);
        if (cached) {
            croppedPreviewImages = cached;
            return;
        }

        const isTauri = typeof window !== "undefined" && "__TAURI__" in window;
        if (!isTauri) {
            croppedPreviewImages = [];
            croppedPreviewError =
                "Preview is only available in the Tauri desktop app.";
            return;
        }

        croppedPreviewLoading = true;
        croppedPreviewImages = [];
        try {
            const images = await generateAnnotatedPreviews(
                outputPath,
                PREVIEW_SAMPLE_COUNT,
            );
            if (!images.length) {
                croppedPreviewError = "No previews available for this dataset.";
            } else {
                croppedPreviewImages = images;
                previewCache.set(outputPath, images);
            }
        } catch (err: any) {
            croppedPreviewError =
                err?.message || "Failed to generate previews.";
        } finally {
            croppedPreviewLoading = false;
        }
    }

    function closeCroppedPreview() {
        showCroppedPreviewModal = false;
        croppedPreviewLoading = false;
        croppedPreviewError = "";
        croppedPreviewImages = [];
        croppedPreviewOutputPath = "";
        selectedPreviewImage = null;
        reopenPreviewAfterKonva = false;
    }

    function handleSelectPreviewImage(image: KonvaImageData) {
        selectedPreviewImage = image;
        reopenPreviewAfterKonva = true;
        showCroppedPreviewModal = false;
    }

    function handlePreviewViewerClose() {
        selectedPreviewImage = null;
        if (reopenPreviewAfterKonva) {
            showCroppedPreviewModal = true;
            reopenPreviewAfterKonva = false;
        }
    }

    async function confirmClearAll(): Promise<boolean> {
        const message =
            "Clear all cropped dataset records? (Temp files will not be deleted)";
        const isTauri = typeof window !== "undefined" && "__TAURI__" in window;
        if (isTauri) {
            return await tauriConfirm(message, {
                title: "Clear Cropped Datasets",
                kind: "warning",
                okLabel: "Clear",
                cancelLabel: "Cancel",
            });
        }
        return typeof window !== "undefined" ? window.confirm(message) : false;
    }

    // Type-safe setters for store union types
    function setAnnotationType(val: string) {
        annotationState.annotationType = val as "bounding_box" | "polygon";
    }
    function setViewMode(val: string) {
        uiState.viewMode = val as "grid" | "column";
    }
    function setEditMode(val: string) {
        uiState.editMode = val as "modal" | "sidebar";
    }

    // Helper to get currentExportDataset (dynamically added property)
    function getCurrentExportDataset(): any {
        return (exportState as any).currentExportDataset;
    }
    function openExportForDataset(dataset: any) {
        exportState.showActualExportModal = true;
        (exportState as any).currentExportDataset = dataset;
    }

    // Trigger auto-annotation on initial directory load
    let lastLoadedDirectory = "";
    $effect(() => {
        if (
            imageState.directoryPath &&
            imageState.images.length > 0 &&
            imageState.directoryPath !== lastLoadedDirectory
        ) {
            lastLoadedDirectory = imageState.directoryPath;
            triggerAutoAnnotationIfNeeded();
        }
    });
</script>

<svelte:head>
    <title>Dataset Gallery</title>
    <meta
        name="description"
        content="Efficient image viewer for large image collections"
    />
</svelte:head>

<svelte:window onkeydown={handleKeydown} />

<div class="container mx-auto px-4 py-8">
    <div class="max-w-6xl mx-auto">
        <div class="mb-8">
            <h1 class="text-3xl font-bold text-base-content mb-6">
                Dataset Gallery
            </h1>

            <!-- New Gallery Navbar -->
            <GalleryNavbar
                isMockMode={imageState.isMockMode}
                loading={imageState.loading}
                directoryPath={imageState.directoryPath}
                images={imageState.images}
                annotationType={annotationState.annotationType}
                autoAnnotationEnabled={annotationState.autoAnnotationEnabled}
                annotating={annotationState.annotating}
                showCropTool={exportState.showCropTool}
                showAdvancedCropTool={exportState.showAdvancedCropTool}
                viewMode={uiState.viewMode}
                editMode={uiState.editMode}
                ontogglemockmode={() =>
                    setMockMode(!imageState.isMockMode)}
                onselectdirectory={selectDirectory}
                onsetannotationtype={setAnnotationType}
                ontoggleautoannotation={() =>
                    (annotationState.autoAnnotationEnabled =
                        !annotationState.autoAnnotationEnabled)}
                onannotateimages={annotateImages}
                onopenexportmodal={() => {
                    exportState.showActualExportModal = true;
                    exportState.pageExportError = "";
                    exportState.pageExportSuccess = "";
                }}
                ontogglecroptool={() =>
                    (exportState.showCropTool = !exportState.showCropTool)}
                ontoggleadvancedcroptool={() =>
                    (exportState.showAdvancedCropTool =
                        !exportState.showAdvancedCropTool)}
                onsetviewmode={setViewMode}
                onseteditmode={setEditMode}
            />

            {#if imageState.error}
                <div class="alert alert-error mb-6">
                    <span class="material-symbols-rounded">error</span>
                    <span>{imageState.error}</span>
                </div>
            {/if}

            <!-- Main Content Area -->

            <!-- Active Cropped Dataset Indicator -->
            {#if exportState.activeCroppedDatasetPath}
                {@const activeDataset = exportState.croppedDatasets.find(
                    (d) => d.tempPath === exportState.activeCroppedDatasetPath,
                )}
                <div
                    class="mb-6 p-4 rounded-xl bg-primary/5 border border-primary/20"
                >
                    <div class="flex items-center justify-between">
                        <div class="flex items-center gap-3">
                            <span class="material-symbols-rounded text-primary"
                                >crop</span
                            >
                            <div>
                                <div class="font-semibold text-base-content">
                                    Viewing Cropped Dataset: <span
                                        class="text-primary"
                                        >{activeDataset?.parentLabel ||
                                            "Unknown"}</span
                                    >
                                </div>
                                <div class="text-sm text-base-content/60">
                                    {activeDataset?.imageCount || 0} images from
                                    <span class="font-mono text-xs"
                                        >{exportState.activeCroppedDatasetPath
                                            ?.split("/")
                                            .pop()}</span
                                    >
                                </div>
                            </div>
                        </div>
                        <IconButton
                            icon="arrow_back"
                            label="Back to Original"
                            tooltip="Switch back to original dataset"
                            variant="ghost"
                            onclick={() =>
                                switchToOriginal(
                                    exportState.originalDirectoryPath,
                                )}
                        />
                    </div>
                </div>
            {/if}

            <!-- 1. Dataset Summary (Original) -->
            {#if !exportState.activeCroppedDatasetPath}
                <div class="mb-8">
                    <DatasetSummary
                        datasetSummary={imageState.datasetSummary}
                        oninitiatecrop={(data) =>
                            openCropModalWithLabel(data.label)}
                    />

                    <!-- Hierarchical Crop Toggle Button -->
                    {#if imageState.directoryPath && imageState.datasetSummary}
                        <div class="mt-4 flex justify-end">
                            <IconButton
                                icon="account_tree"
                                label="Hierarchical Crop"
                                tooltip="Crop by parent label and remap children"
                                active={exportState.showHierarchicalCrop}
                                variant={exportState.showHierarchicalCrop
                                    ? "soft"
                                    : "ghost"}
                                onclick={() =>
                                    (exportState.showHierarchicalCrop =
                                        !exportState.showHierarchicalCrop)}
                            />
                        </div>
                    {/if}

                    <!-- Hierarchical Crop Tool -->
                    {#if exportState.showHierarchicalCrop}
                        <div class="mt-4">
                            <HierarchicalCrop
                                currentDirectory={imageState.directoryPath}
                                cropToolOpen={exportState.showHierarchicalCrop}
                                preSelectedParentLabel={exportState.cropModalParentLabel}
                                oncropstart={(detail) => {
                                    runCropInBackground(detail);
                                }}
                            />
                        </div>
                    {/if}
                </div>
            {/if}

            <!-- Crop Processing Progress Bar -->
            {#if exportState.cropProcessing}
                <div
                    class="mb-6 p-4 bg-base-200 rounded-xl border border-primary/20"
                >
                    <div class="flex items-center justify-between mb-2">
                        <div class="flex items-center gap-3">
                            <span
                                class="loading loading-spinner loading-sm text-primary"
                            ></span>
                            <span class="font-medium text-primary"
                                >{exportState.cropProgressMessage}</span
                            >
                        </div>
                        {#if exportState.cropProgressTotal > 0}
                            <span class="text-sm font-mono text-primary/80">
                                {Math.round(
                                    (exportState.cropProgressCurrent /
                                        exportState.cropProgressTotal) *
                                        100,
                                )}%
                            </span>
                        {/if}
                    </div>
                    <div
                        class="w-full bg-base-300 rounded-full h-2 overflow-hidden"
                    >
                        <div
                            class="bg-primary h-full rounded-full transition-all duration-300"
                            style="width: {exportState.cropProgressTotal > 0
                                ? (exportState.cropProgressCurrent /
                                      exportState.cropProgressTotal) *
                                      100 +
                                  '%'
                                : '100%'}"
                            class:animate-pulse={exportState.cropProgressTotal ===
                                0}
                        ></div>
                    </div>

                    <p class="text-xs text-base-content/50 mt-2">
                        Processing in background... You can continue using the
                        app.
                    </p>
                </div>
            {/if}

            <!-- Cropped Datasets Section -->
            {#if exportState.croppedDatasets.length > 0}
                <div class="mb-8">
                    <div class="flex items-center justify-between mb-4">
                        <div class="flex items-center gap-2">
                            <span class="material-symbols-rounded text-success"
                                >check_circle</span
                            >
                            <h3 class="font-bold text-base-content">
                                Cropped Datasets ({exportState.croppedDatasets
                                    .length})
                            </h3>
                        </div>
                        <IconButton
                            icon="delete_sweep"
                            label="Clear All"
                            tooltip="Clear all cropped dataset records"
                            variant="ghost"
                            size="sm"
                            onclick={async () => {
                                if (await confirmClearAll()) {
                                    clearAllCroppedDatasets();
                                }
                            }}
                        />
                    </div>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {#each exportState.croppedDatasets as dataset (dataset.tempPath)}
                            <CroppedDatasetCard
                                tempPath={dataset.tempPath}
                                exportedPath={dataset.exportedPath}
                                imageCount={dataset.imageCount}
                                parentLabel={dataset.parentLabel}
                                childLabels={dataset.childLabels}
                                createdAt={dataset.createdAt}
                                onpreview={(data) =>
                                    openCroppedPreview(data.tempPath)}
                                onopeningallery={(data) =>
                                    openCroppedDatasetInGallery(
                                        data.tempPath,
                                    )}
                                onremove={(data) =>
                                    removeCroppedDataset(
                                        data.tempPath,
                                    )}
                                onexport={() => openExportForDataset(dataset)}
                            />
                        {/each}
                    </div>
                </div>
            {/if}

            <!-- 2. Image Gallery and Loading States -->
            <div
                class="flex flex-col lg:flex-row gap-6 items-start h-[calc(100vh-200px)]"
            >
                <!-- Gallery Grid Area -->
                <div
                    class="flex-1 w-full h-full overflow-hidden flex flex-col transition-all duration-300"
                >
                    {#if imageState.loading || !imageState.directoryPath || imageState.images.length === 0}
                        <!-- Shared Empty/Loading State Component -->
                        <GalleryEmptyState
                            loading={imageState.loading}
                            directoryPath={imageState.directoryPath}
                            images={imageState.images}
                            onselectdirectory={selectDirectory}
                        />
                    {:else}
                        <!-- Image Gallery Component -->
                        <div
                            class="h-full overflow-y-auto pr-2 rounded-box border border-base-300 bg-base-100"
                            class:pointer-events-none={uiState.showAnnotationModal}
                        >
                            <ImageGallery
                                images={imageState.images}
                                viewMode={uiState.viewMode}
                                currentPage={imageState.currentPage}
                                totalPages={imageState.totalPages}
                                pageSize={imageState.pageSize}
                                selectedImage={uiState.selectedImage}
                                onloadpage={(page) => handlePageChange(page)}
                                onimageclick={(data) => {
                                    if (uiState.editMode === "modal") {
                                        uiState.selectedImage = data.image;
                                        uiState.showAnnotationModal = true;
                                    } else {
                                        uiState.selectedImage = data.image;
                                    }
                                }}
                            />
                        </div>
                    {/if}
                </div>

                <!-- Right Sidebar Panel -->
                {#if uiState.selectedImage && uiState.editMode === "sidebar"}
                    <div
                        class="w-full lg:w-[450px] xl:w-[500px] h-full flex-none bg-base-100 rounded-box shadow-xl border border-base-300 overflow-hidden flex flex-col animate-in slide-in-from-right-4 duration-300"
                    >
                        <ImagePreviewPanel
                            selectedImage={uiState.selectedImage}
                            onclose={() => (uiState.selectedImage = null)}
                        />
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>

<!-- Unified Export Modal -->
<ExportModal
    bind:showModal={exportState.showActualExportModal}
    currentDirectoryPath={getCurrentExportDataset()?.tempPath ||
        imageState.directoryPath}
    currentDatasetSummary={imageState.datasetSummary}
    onclosemodal={() => {
        exportState.showActualExportModal = false;
    }}
    onrunexport={(detail) => {
        const dataset = getCurrentExportDataset();
        runUnifiedExport({
            ...detail,
            sourceDir: dataset?.tempPath || detail.sourceDir,
        });
    }}
/>

<!-- Cropped Dataset Preview Modal -->
<CroppedDatasetPreviewModal
    isOpen={showCroppedPreviewModal}
    loading={croppedPreviewLoading}
    error={croppedPreviewError}
    outputPath={croppedPreviewOutputPath}
    images={croppedPreviewImages}
    sampleCount={PREVIEW_SAMPLE_COUNT}
    onclose={closeCroppedPreview}
    onselectimage={(data) => handleSelectPreviewImage(data.image)}
/>

<!-- Konva Viewer for Preview -->
<KonvaViewer
    showModal={selectedPreviewImage !== null}
    imageData={selectedPreviewImage}
    onclose={handlePreviewViewerClose}
/>

<!-- Modal Annotation Viewer (Pop-out Mode) -->
{#if uiState.showAnnotationModal && uiState.selectedImage}
    <ModalAnnotationViewer
        showModal={uiState.showAnnotationModal}
        selectedImage={uiState.selectedImage}
        autoAnnotationEnabled={annotationState.autoAnnotationEnabled}
        isMockMode={imageState.isMockMode}
        onclose={() => {
            resetSelection();
        }}
        onsave={(data) => {
            console.log("Annotation retained/saved via modal");
        }}
    />
{/if}

<!-- Crop & Remap Modal -->
<CropRemapTool
    isOpen={exportState.showCropTool}
    oncropcompleted={(data) =>
        handleCropCompleted(data.outputDir)}
    onclose={() => (exportState.showCropTool = false)}
/>

<!-- Advanced Crop & Remap Tool -->
{#if exportState.showAdvancedCropTool}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
        class="fixed inset-0 bg-black/50 z-40 flex items-center justify-center p-8"
        onclick={(e) => { if (e.target === e.currentTarget) closeCropModal(); }}
        onkeydown={(e) => e.key === "Escape" && closeCropModal()}
        role="dialog"
        aria-modal="true"
        tabindex="-1"
    >
        <div
            class="bg-base-100 rounded-2xl shadow-2xl max-w-5xl w-full max-h-[90vh] overflow-y-auto"
        >
            <div
                class="flex items-center justify-between p-4 border-b border-base-300"
            >
                <h2 class="text-xl font-bold flex items-center gap-2">
                    <span class="material-symbols-rounded text-primary"
                        >crop</span
                    >
                    Advanced Crop & Remap
                    {#if exportState.cropModalParentLabel}
                        <span class="badge badge-primary"
                            >{exportState.cropModalParentLabel}</span
                        >
                    {/if}
                </h2>
                <button
                    class="btn btn-sm btn-ghost btn-square"
                    onclick={() => closeCropModal()}
                >
                    <span class="material-symbols-rounded">close</span>
                </button>
            </div>
            <AdvancedCropRemapTool
                currentDirectory={imageState.directoryPath}
                cropToolOpen={exportState.showAdvancedCropTool}
                preSelectedParentLabel={exportState.cropModalParentLabel}
                oncropcompleted={(data) => {
                    handleCropCompleted(data.outputDir, {
                        parentLabel: exportState.cropModalParentLabel,
                        childLabels: data.childLabels || [],
                        imageCount: data.imageCount || 0,
                    });
                }}
            />
        </div>
    </div>
{/if}

<!-- Toast Notifications -->
<Toast />

<style>
</style>

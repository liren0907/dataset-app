<script lang="ts">
    import { onMount } from "svelte";
    import DatasetSummary from "./components/DatasetSummary.svelte";
    import ImagePreviewPanel from "./components/ImagePreviewPanel.svelte";
    import ImageGallery from "./components/ImageGallery.svelte";
    import ExportModal from "./components/ExportModal.svelte";
    import ModalAnnotationViewer from "./components/ModalAnnotationViewer.svelte";
    import CropRemapTool from "./components/CropRemapTool.svelte";
    import AdvancedCropRemapTool from "./components/AdvancedCropRemapTool.svelte";
    import HierarchicalCrop from "./components/HierarchicalCrop.svelte";
    import GalleryNavbar from "./components/GalleryNavbar.svelte";
    import GalleryEmptyState from "./components/GalleryEmptyState.svelte";
    import CroppedDatasetCard from "./components/CroppedDatasetCard.svelte";
    import CroppedDatasetSummary from "./components/CroppedDatasetSummary.svelte";
    import CroppedDatasetPreviewModal from "./components/CroppedDatasetPreviewModal.svelte";
    import KonvaViewer from "./components/KonvaViewer.svelte";
    import { IconButton } from "$lib/components/ui";
    import { generateAnnotatedPreviews } from "./services/datasetService";
    import type { KonvaImageData } from "./services/konvaService";

    // Import separated stores
    import { imageStore } from "./stores/imageStore";
    import { uiStore } from "./stores/uiStore";
    import { annotationStore } from "./stores/annotationStore";
    import { exportStore } from "./stores/exportStore";

    const PREVIEW_SAMPLE_COUNT = 8;

    let showCroppedPreviewModal: boolean = false;
    let croppedPreviewLoading: boolean = false;
    let croppedPreviewError: string = "";
    let croppedPreviewImages: KonvaImageData[] = [];
    let croppedPreviewOutputPath: string = "";
    let selectedPreviewImage: KonvaImageData | null = null;
    let reopenPreviewAfterKonva: boolean = false;
    const previewCache: Map<string, KonvaImageData[]> = new Map();

    // --- Helper: Trigger Auto-Annotation ---
    async function triggerAutoAnnotationIfNeeded(
        page: number = $imageStore.currentPage,
    ) {
        if (
            $annotationStore.autoAnnotationEnabled &&
            $imageStore.images.length > 0
        ) {
            await annotationStore.autoLoadAnnotationsForPage(page);
        }
    }

    onMount(async () => {
        const isTauri = typeof window !== "undefined" && "__TAURI__" in window;
        if (!isTauri) {
            console.log("🌍 Browser environment detected (No Tauri).");
            await imageStore.setMockMode(true);
        }
    });

    // Keyboard shortcuts
    function handleKeydown(event: KeyboardEvent) {
        if (event.key === "Escape" && $uiStore.selectedImage) {
            $uiStore.selectedImage = null;
        }
    }

    async function handlePageChange(event: CustomEvent) {
        const page = event.detail.page;
        await imageStore.loadImagesPage(page);
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

    // Trigger auto-annotation on initial directory load
    let lastLoadedDirectory = "";
    $: if (
        $imageStore.directoryPath &&
        $imageStore.images.length > 0 &&
        $imageStore.directoryPath !== lastLoadedDirectory
    ) {
        lastLoadedDirectory = $imageStore.directoryPath;
        triggerAutoAnnotationIfNeeded();
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
        <div class="mb-8">
            <h1 class="text-3xl font-bold text-base-content mb-6">
                Dataset Gallery
            </h1>

            <!-- New Gallery Navbar -->
            <GalleryNavbar
                isMockMode={$imageStore.isMockMode}
                loading={$imageStore.loading}
                directoryPath={$imageStore.directoryPath}
                images={$imageStore.images}
                annotationType={$annotationStore.annotationType}
                autoAnnotationEnabled={$annotationStore.autoAnnotationEnabled}
                annotating={$annotationStore.annotating}
                showCropTool={$exportStore.showCropTool}
                showAdvancedCropTool={$exportStore.showAdvancedCropTool}
                viewMode={$uiStore.viewMode}
                editMode={$uiStore.editMode}
                on:toggleMockMode={() =>
                    imageStore.setMockMode(!$imageStore.isMockMode)}
                on:selectDirectory={imageStore.selectDirectory}
                on:setAnnotationType={(e) =>
                    ($annotationStore.annotationType = e.detail)}
                on:toggleAutoAnnotation={() =>
                    ($annotationStore.autoAnnotationEnabled =
                        !$annotationStore.autoAnnotationEnabled)}
                on:annotateImages={annotationStore.annotateImages}
                on:openExportModal={() => {
                    $exportStore.showActualExportModal = true;
                    $exportStore.pageExportError = "";
                    $exportStore.pageExportSuccess = "";
                }}
                on:toggleCropTool={() =>
                    ($exportStore.showCropTool = !$exportStore.showCropTool)}
                on:toggleAdvancedCropTool={() =>
                    ($exportStore.showAdvancedCropTool =
                        !$exportStore.showAdvancedCropTool)}
                on:setViewMode={(e) => ($uiStore.viewMode = e.detail)}
                on:setEditMode={(e) => ($uiStore.editMode = e.detail)}
            />

            {#if $imageStore.error}
                <div class="alert alert-error mb-6">
                    <span class="material-symbols-rounded">error</span>
                    <span>{$imageStore.error}</span>
                </div>
            {/if}

            <!-- Main Content Area -->

            <!-- Active Cropped Dataset Indicator -->
            {#if $exportStore.activeCroppedDatasetPath}
                {@const activeDataset = $exportStore.croppedDatasets.find(
                    (d) =>
                        d.outputPath === $exportStore.activeCroppedDatasetPath,
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
                                        >{$exportStore.activeCroppedDatasetPath
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
                            on:click={() =>
                                exportStore.switchToOriginal(
                                    $exportStore.originalDirectoryPath,
                                )}
                        />
                    </div>
                </div>
            {/if}

            <!-- 1. Dataset Summary (Original) -->
            {#if !$exportStore.activeCroppedDatasetPath}
                <div class="mb-8">
                    <DatasetSummary
                        datasetSummary={$imageStore.datasetSummary}
                        on:initiateCrop={(e) =>
                            exportStore.openCropModalWithLabel(e.detail.label)}
                    />

                    <!-- Hierarchical Crop Toggle Button -->
                    {#if $imageStore.directoryPath && $imageStore.datasetSummary}
                        <div class="mt-4 flex justify-end">
                            <IconButton
                                icon="account_tree"
                                label="Hierarchical Crop"
                                tooltip="Crop by parent label and remap children"
                                active={$exportStore.showHierarchicalCrop}
                                variant={$exportStore.showHierarchicalCrop
                                    ? "soft"
                                    : "ghost"}
                                on:click={() =>
                                    ($exportStore.showHierarchicalCrop =
                                        !$exportStore.showHierarchicalCrop)}
                            />
                        </div>
                    {/if}

                    <!-- Hierarchical Crop Tool -->
                    {#if $exportStore.showHierarchicalCrop}
                        <div class="mt-4">
                            <HierarchicalCrop
                                currentDirectory={$imageStore.directoryPath}
                                cropToolOpen={$exportStore.showHierarchicalCrop}
                                preSelectedParentLabel={$exportStore.cropModalParentLabel}
                                on:cropCompleted={(e) => {
                                    exportStore.handleCropCompleted(
                                        e.detail.outputDir,
                                        {
                                            parentLabel:
                                                e.detail.parentLabel || "",
                                            childLabels:
                                                e.detail.childLabels || [],
                                            imageCount:
                                                e.detail.imageCount || 0,
                                        },
                                    );
                                }}
                            />
                        </div>
                    {/if}
                </div>
            {/if}

            <!-- Cropped Datasets Section -->
            {#if $exportStore.croppedDatasets.length > 0}
                <div class="mb-8">
                    <div class="flex items-center gap-2 mb-4">
                        <span class="material-symbols-rounded text-success"
                            >check_circle</span
                        >
                        <h3 class="font-bold text-base-content">
                            Cropped Datasets ({$exportStore.croppedDatasets
                                .length})
                        </h3>
                    </div>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {#each $exportStore.croppedDatasets as dataset (dataset.outputPath)}
                            <CroppedDatasetCard
                                outputPath={dataset.outputPath}
                                imageCount={dataset.imageCount}
                                parentLabel={dataset.parentLabel}
                                childLabels={dataset.childLabels}
                                createdAt={dataset.createdAt}
                                on:preview={(e) =>
                                    openCroppedPreview(e.detail.outputPath)}
                                on:openInGallery={(e) =>
                                    exportStore.openCroppedDatasetInGallery(
                                        e.detail.outputPath,
                                    )}
                                on:remove={(e) =>
                                    exportStore.removeCroppedDataset(
                                        e.detail.outputPath,
                                    )}
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
                    {#if $imageStore.loading || !$imageStore.directoryPath || $imageStore.images.length === 0}
                        <!-- Shared Empty/Loading State Component -->
                        <GalleryEmptyState
                            loading={$imageStore.loading}
                            directoryPath={$imageStore.directoryPath}
                            images={$imageStore.images}
                            on:selectDirectory={imageStore.selectDirectory}
                        />
                    {:else}
                        <!-- Image Gallery Component -->
                        <div
                            class="h-full overflow-y-auto pr-2 rounded-box border border-base-300 bg-base-100"
                            class:pointer-events-none={$uiStore.showAnnotationModal}
                        >
                            <ImageGallery
                                images={$imageStore.images}
                                viewMode={$uiStore.viewMode}
                                currentPage={$imageStore.currentPage}
                                totalPages={$imageStore.totalPages}
                                pageSize={$imageStore.pageSize}
                                selectedImage={$uiStore.selectedImage}
                                on:pageChange={handlePageChange}
                                on:imageClick={(e) => {
                                    if ($uiStore.editMode === "modal") {
                                        $uiStore.selectedImage = e.detail.image;
                                        $uiStore.showAnnotationModal = true;
                                    } else {
                                        $uiStore.selectedImage = e.detail.image;
                                    }
                                }}
                            />
                        </div>
                    {/if}
                </div>

                <!-- Right Sidebar Panel -->
                {#if $uiStore.selectedImage && $uiStore.editMode === "sidebar"}
                    <div
                        class="w-full lg:w-[450px] xl:w-[500px] h-full flex-none bg-base-100 rounded-box shadow-xl border border-base-300 overflow-hidden flex flex-col animate-in slide-in-from-right-4 duration-300"
                    >
                        <ImagePreviewPanel
                            selectedImage={$uiStore.selectedImage}
                            on:close={() => ($uiStore.selectedImage = null)}
                        />
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>

<!-- Unified Export Modal -->
<ExportModal
    bind:showModal={$exportStore.showActualExportModal}
    currentDirectoryPath={$imageStore.directoryPath}
    currentDatasetSummary={$imageStore.datasetSummary}
    on:closeModal={() => {
        $exportStore.showActualExportModal = false;
    }}
    on:runExport={(event) => exportStore.runUnifiedExport(event.detail)}
/>

<!-- Cropped Dataset Preview Modal -->
<CroppedDatasetPreviewModal
    isOpen={showCroppedPreviewModal}
    loading={croppedPreviewLoading}
    error={croppedPreviewError}
    outputPath={croppedPreviewOutputPath}
    images={croppedPreviewImages}
    sampleCount={PREVIEW_SAMPLE_COUNT}
    on:close={closeCroppedPreview}
    on:selectImage={(e) => handleSelectPreviewImage(e.detail.image)}
/>

<!-- Konva Viewer for Preview -->
<KonvaViewer
    showModal={selectedPreviewImage !== null}
    imageData={selectedPreviewImage}
    on:close={handlePreviewViewerClose}
/>

<!-- Modal Annotation Viewer (Pop-out Mode) -->
{#if $uiStore.showAnnotationModal && $uiStore.selectedImage}
    <ModalAnnotationViewer
        bind:showModal={$uiStore.showAnnotationModal}
        selectedImage={$uiStore.selectedImage}
        autoAnnotationEnabled={$annotationStore.autoAnnotationEnabled}
        isMockMode={$imageStore.isMockMode}
        on:close={() => {
            $uiStore.showAnnotationModal = false;
            $uiStore.selectedImage = null;
        }}
        on:save={(e) => {
            console.log("Annotation retained/saved via modal");
        }}
    />
{/if}

<!-- Crop & Remap Modal -->
<CropRemapTool
    isOpen={$exportStore.showCropTool}
    on:cropCompleted={(e) =>
        exportStore.handleCropCompleted(e.detail.outputDir)}
    on:close={() => ($exportStore.showCropTool = false)}
/>

<!-- Advanced Crop & Remap Tool -->
{#if $exportStore.showAdvancedCropTool}
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div
        class="fixed inset-0 bg-black/50 z-40 flex items-center justify-center p-8"
        on:click|self={() => exportStore.closeCropModal()}
        on:keydown={(e) => e.key === "Escape" && exportStore.closeCropModal()}
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
                    {#if $exportStore.cropModalParentLabel}
                        <span class="badge badge-primary"
                            >{$exportStore.cropModalParentLabel}</span
                        >
                    {/if}
                </h2>
                <button
                    class="btn btn-sm btn-ghost btn-square"
                    on:click={() => exportStore.closeCropModal()}
                >
                    <span class="material-symbols-rounded">close</span>
                </button>
            </div>
            <AdvancedCropRemapTool
                currentDirectory={$imageStore.directoryPath}
                cropToolOpen={$exportStore.showAdvancedCropTool}
                preSelectedParentLabel={$exportStore.cropModalParentLabel}
                on:cropCompleted={(e) => {
                    exportStore.handleCropCompleted(e.detail.outputDir, {
                        parentLabel: $exportStore.cropModalParentLabel,
                        childLabels: e.detail.childLabels || [],
                        imageCount: e.detail.imageCount || 0,
                    });
                }}
            />
        </div>
    </div>
{/if}

<style>
</style>

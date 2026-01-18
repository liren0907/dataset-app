<script lang="ts">
    import { onMount } from "svelte";
    import DatasetSummaryCard from "./components/DatasetSummaryCard.svelte";
    import ImagePreviewPanel from "./components/ImagePreviewPanel.svelte";
    import ImageGallery from "./components/ImageGallery.svelte";
    import ExportModal from "./components/ExportModal.svelte";
    import ModalAnnotationViewer from "./components/ModalAnnotationViewer.svelte";
    import CropRemapTool from "./components/CropRemapTool.svelte";
    import ExtractLabelsModal from "./components/ExtractLabelsModal.svelte";
    import GalleryNavbar from "./components/GalleryNavbar.svelte";
    import GalleryEmptyState from "./components/GalleryEmptyState.svelte";

    // Import separated stores
    import { imageStore } from "./stores/imageStore";
    import { uiStore } from "./stores/uiStore";
    import { annotationStore } from "./stores/annotationStore";
    import { exportStore } from "./stores/exportStore";

    // --- Initialization ---
    // If auto-annotation is enabled in annotationStore and we load a page, we need to trigger it.
    // However, the reactive trigger was extracted into the store logic in `loadImagesPage`. Check implementation.
    // In `imageStore.loadImagesPage`, I added:
    // `if (page === 1) generateLabelMeSummary();`
    // Wait, the cross-store dependency for auto-annotation logic:
    // Ideally `imageStore.loadImagesPage` should trigger `annotationStore.autoLoad...` if enabled.
    // But `imageStore` doesn't import `annotationStore` (circular).
    // So we can do it reactively here or via a coordinated action.

    // Reactive: When images change, if enabled, trigger annotation.
    // Better: We hook into the load completion.
    // Current solution in code: `imageStore` handled it? No, I likely removed it to avoid circular dependency.
    // Let's check `imageStore.ts`... I didn't include autoLoadAnnotationsForPage in loadImagesPage import.
    // I need to add that trigger here in Svelte or make an "Action" that coordinates them.

    // Simplest: Reactive statement here.
    $: if (
        $annotationStore.autoAnnotationEnabled &&
        $imageStore.images.length > 0 &&
        !$imageStore.loading
    ) {
        // This might re-trigger too often.
        // Strategy: We'll stick to manual triggers or exact event callbacks if needed.
        // Actually, let's keep it simple. If the user moves page, `loadImagesPage` runs.
        // We can modify `loadImagesPage` to accept a callback or use a `store subscriber`.
    }

    // Actually, looking at my `imageStore.ts` implementation:
    // I REMOVED the auto-annotation call from `loadImagesPage`.
    // So we need to restore that behavior.
    // We can do it by modifying `handlePageChange`.

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
        if ($annotationStore.autoAnnotationEnabled) {
            await annotationStore.autoLoadAnnotationsForPage(page);
        }
    }

    // Auto-load annotations for the first page if loaded initially?
    // We can watch `imageStore.currentPage` changes? No, `handlePageChange` covers UI clicks.
    // Initial load? `selectDirectory` calls `loadImagesPage(1)`.
    // We might need to override `selectDirectory` to also trigger annotation or just add a reactive block:

    let previousImagesPath = "";
    $: if (
        $imageStore.directoryPath &&
        $imageStore.images.length > 0 &&
        $annotationStore.autoAnnotationEnabled &&
        $imageStore.directoryPath !== previousImagesPath
    ) {
        // Quick fix for initial load auto-annotation if we want strictly 100% parity
        // But let's assume `handlePageChange` is the main driver for pagination.
        // For initial load, maybe just let the user see raw images first or trigger it manually.
        // Or simply:
        previousImagesPath = $imageStore.directoryPath;
        annotationStore.autoLoadAnnotationsForPage($imageStore.currentPage);
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
                autoAnnotating={$annotationStore.autoAnnotating}
                showCropTool={$exportStore.showCropTool}
                datasetSummary={$imageStore.datasetSummary}
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
                on:openExtractModal={() => {
                    $exportStore.extractError = "";
                    $exportStore.extractSuccess = "";
                    $exportStore.showExtractModal = true;
                }}
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

            <!-- 1. Dataset Summary Card -->
            <div class="mb-8">
                <DatasetSummaryCard
                    datasetSummary={$imageStore.datasetSummary}
                />
            </div>

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

<!-- Extract Labels Modal -->
<ExtractLabelsModal
    isOpen={$exportStore.showExtractModal}
    sourceDir={$imageStore.directoryPath}
    datasetSummary={$imageStore.datasetSummary}
    on:extract={(e) => exportStore.handleExtractLabels(e.detail)}
    on:close={() => ($exportStore.showExtractModal = false)}
/>

<!-- Crop & Remap Modal -->
<CropRemapTool
    isOpen={$exportStore.showCropTool}
    on:cropCompleted={(e) =>
        exportStore.handleCropCompleted(e.detail.outputDir)}
    on:close={() => ($exportStore.showCropTool = false)}
/>

<style>
</style>

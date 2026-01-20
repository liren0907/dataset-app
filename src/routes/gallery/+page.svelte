<script lang="ts">
    import { onMount } from "svelte";
    import DatasetSummary from "./components/DatasetSummary.svelte";
    import ImagePreviewPanel from "./components/ImagePreviewPanel.svelte";
    import ImageGallery from "./components/ImageGallery.svelte";
    import ExportModal from "./components/ExportModal.svelte";
    import ModalAnnotationViewer from "./components/ModalAnnotationViewer.svelte";
    import CropRemapTool from "./components/CropRemapTool.svelte";
    import AdvancedCropRemapTool from "./components/AdvancedCropRemapTool.svelte";
    import GalleryNavbar from "./components/GalleryNavbar.svelte";
    import GalleryEmptyState from "./components/GalleryEmptyState.svelte";

    // Import separated stores
    import { imageStore } from "./stores/imageStore";
    import { uiStore } from "./stores/uiStore";
    import { annotationStore } from "./stores/annotationStore";
    import { exportStore } from "./stores/exportStore";

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

            <!-- 1. Dataset Summary -->
            <div class="mb-8">
                <DatasetSummary datasetSummary={$imageStore.datasetSummary} />
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

<!-- Crop & Remap Modal -->
<CropRemapTool
    isOpen={$exportStore.showCropTool}
    on:cropCompleted={(e) =>
        exportStore.handleCropCompleted(e.detail.outputDir)}
    on:close={() => ($exportStore.showCropTool = false)}
/>

<!-- Advanced Crop & Remap Tool -->
{#if $exportStore.showAdvancedCropTool}
    <div
        class="fixed inset-0 bg-black/50 z-40 flex items-center justify-center p-8"
    >
        <div
            class="bg-base-100 rounded-2xl shadow-2xl max-w-5xl w-full max-h-[90vh] overflow-y-auto"
        >
            <div
                class="flex items-center justify-between p-4 border-b border-base-300"
            >
                <h2 class="text-xl font-bold">Advanced Crop & Remap</h2>
                <button
                    class="btn btn-sm btn-ghost btn-square"
                    on:click={() => ($exportStore.showAdvancedCropTool = false)}
                >
                    <span class="material-symbols-rounded">close</span>
                </button>
            </div>
            <AdvancedCropRemapTool
                currentDirectory={$imageStore.directoryPath}
                cropToolOpen={$exportStore.showAdvancedCropTool}
                on:cropCompleted={(e) => {
                    exportStore.handleCropCompleted(e.detail.outputDir);
                    $exportStore.showAdvancedCropTool = false;
                }}
            />
        </div>
    </div>
{/if}

<style>
</style>

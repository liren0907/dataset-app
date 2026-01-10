<script lang="ts">
    import { onMount, tick } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { fade } from "svelte/transition";

    // Components
    import ViewerControls from "./components/ViewerControls.svelte";
    import CropTool from "./components/CropTool.svelte";
    import DatasetSummary from "./components/DatasetSummary.svelte";
    import ImageGrid from "./components/ImageGrid.svelte";
    import ImageViewerModal from "./components/ImageViewerModal.svelte";
    import ExportModal from "./components/ExportModal.svelte";
    import ExtractLabelsModal from "./components/ExtractLabelsModal.svelte";

    // --- State Variables ---
    let directoryPath = "";
    let images: any[] = [];
    let loading = false;
    let annotating = false;
    let error = "";
    let containerElement: HTMLElement;
    let selectedImage: any = null;
    let annotationType = "bounding_box"; // "bounding_box" or "polygon"

    // Pagination
    let currentPage = 1;
    let pageSize = 30; // Number of images to load initially/per page
    let totalImages = 0;
    let totalPages = 0;
    let loadingMore = false; // Flag for infinite scroll/pagination loading

    // View Mode
    let viewMode: "grid" | "column" = "grid";

    // Dataset Summary
    let datasetSummary: any = null;

    // --- Crop & Remap Tool State ---
    let cropToolOpen = false;
    let cropSourceDir: string | null = null;
    let cropOutputDir: string | null = null;
    let parentLabel: string = "person";
    let cropLoading = false;
    let cropStatusMessage: string | null = null;
    let cropIsError = false;

    // --- YOLO Export State ---
    let showExportModal = false;
    let exportSettings = {
        outputDir: "",
        trainRatio: 0.7,
        valRatio: 0.2,
        testRatio: 0.1,
        shapeType: "polygon", // 'polygon' or 'bounding_box'
    };
    let excludedLabels = new Set<string>();
    let exportLoading = false;
    let exportSuccess = "";
    let exportError = "";

    // --- Extract Labels State ---
    let showExtractModal = false;
    let extractSettings = {
        outputDir: "", // Logic will define this later
    };
    let selectedLabelsForExtract = new Set<string>();
    let extractLoading = false;
    let extractSuccess = "";
    let extractError = "";

    // --- Functions ---
    async function selectDirectory() {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select Dataset Directory",
            });

            if (selected) {
                directoryPath = selected as string;
                currentPage = 1; // Reset pagination
                images = []; // Clear current images
                datasetSummary = null; // Clear old summary
                error = "";
                await loadImagesPage(1);
            }
        } catch (err) {
            console.error("Error selecting directory:", err);
            error = "Failed to select directory.";
        }
    }

    async function generateLabelMeSummary() {
        if (!directoryPath) return;
        try {
            datasetSummary = await invoke("get_dataset_summary", {
                datasetPath: directoryPath,
            });

            // Initialize excluded labels for export (start empty -> export all)
            excludedLabels = new Set();

            // Initialize selected labels for extract (start all selected or empty)
            if (datasetSummary && datasetSummary.label_counts) {
                selectedLabelsForExtract = new Set(
                    Object.keys(datasetSummary.label_counts),
                );
            }
        } catch (err) {
            console.error("Error generating summary:", err);
            // Don't show user visible error for summary failure, just log it
        }
    }

    async function loadImagesPage(page: number) {
        if (!directoryPath) return;

        loading = true;
        error = "";
        loadingMore = page > 1;

        try {
            const response: any = await invoke("load_images_page", {
                directory: directoryPath,
                page: page,
                pageSize: pageSize,
            });

            console.log("Load images response:", response);

            if (page === 1) {
                images = []; // Clear images on first page load (new directory or refresh)
                totalImages = response.total_images;
                totalPages = response.total_pages;
                // Generate summary when first loading the directory
                generateLabelMeSummary();
            }

            const newImages = await Promise.all(
                response.images.map(async (img: any) => {
                    return {
                        ...img,
                        previewUrl: convertFileSrc(img.path),
                        displayIndex:
                            (page - 1) * pageSize +
                            response.images.indexOf(img), // Keep track of global index
                    };
                }),
            );

            if (page === 1) {
                images = newImages;
            } else {
                images = [...images, ...newImages];
            }

            currentPage = page;
        } catch (err) {
            console.error("Error loading images:", err);
            error = `Failed to load images: ${err instanceof Error ? err.message : String(err)}`;
        } finally {
            loading = false;
            loadingMore = false;
        }
    }

    // --- Annotation Logic ---
    async function annotateImages() {
        if (!directoryPath) return;

        annotating = true;
        try {
            const annotationsMap: any = await invoke(
                "load_annotations_for_images",
                {
                    directory: directoryPath,
                    annotationType: annotationType, // Pass preference if needed, or backend handles all
                },
            );

            // Map annotations back to images
            images = images.map((img) => {
                if (annotationsMap[img.path]) {
                    return {
                        ...img,
                        annotations: annotationsMap[img.path],
                        annotated: true,
                    };
                }
                return img;
            });
        } catch (err) {
            console.error("Error loading annotations:", err);
            error = "Failed to load annotations.";
        } finally {
            annotating = false;
        }
    }

    function handleSelectImage(event: CustomEvent) {
        const { image, index } = event.detail;
        selectedImage = image;
    }

    function handleChangeViewMode(event: CustomEvent) {
        viewMode = event.detail;
    }

    // --- Crop Tool Handlers ---
    async function selectCropDirectory(type: "source" | "output") {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title:
                    type === "source"
                        ? "Select Source Directory"
                        : "Select Output Directory",
            });

            if (selected) {
                if (type === "source") {
                    cropSourceDir = selected as string;
                } else {
                    cropOutputDir = selected as string;
                }
                cropStatusMessage = null;
                cropIsError = false;
            }
        } catch (err) {
            console.error("Error selecting crop directory:", err);
            cropStatusMessage = `Failed: ${err}`;
            cropIsError = true;
        }
    }

    async function runCropAndRemap() {
        if (!cropSourceDir || !cropOutputDir || !parentLabel) {
            cropStatusMessage = "Missing parameters.";
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
                parentLabel: parentLabel,
            });
            cropStatusMessage = String(message);
            cropIsError = false;

            // Auto-load results
            directoryPath = cropOutputDir;
            currentPage = 1;
            images = [];
            datasetSummary = null;
            selectedImage = null;
            error = "";
            await loadImagesPage(1);
        } catch (err) {
            console.error("Error running crop:", err);
            cropStatusMessage = `Failed: ${err}`;
            cropIsError = true;
        } finally {
            cropLoading = false;
        }
    }

    // --- Export Handlers ---
    function openExportModal() {
        if (!directoryPath) return;
        // Set default output dir to parent of current dir + "_yolo" if possible, or just empty
        // Simple logic:
        exportSettings.outputDir = "";
        showExportModal = true;
        exportSuccess = "";
        exportError = "";
    }

    async function selectExportDirectory() {
        const selected = await open({
            directory: true,
            multiple: false,
            title: "Select Export Output Directory",
        });
        if (selected) {
            exportSettings.outputDir = selected as string;
        }
    }

    function toggleLabelExclusion(event: CustomEvent) {
        const label = event.detail;
        if (excludedLabels.has(label)) {
            excludedLabels.delete(label);
            excludedLabels = excludedLabels; // Trigger reactivity
        } else {
            excludedLabels.add(label);
            excludedLabels = excludedLabels;
        }
    }

    async function exportToYolo() {
        if (!directoryPath || !exportSettings.outputDir) return;

        exportLoading = true;
        exportError = "";
        exportSuccess = "";

        try {
            const allLabels = Object.keys(datasetSummary.label_counts);
            const includedLabels = allLabels.filter(
                (l) => !excludedLabels.has(l),
            );

            const result = await invoke("export_dataset_to_yolo", {
                sourceDir: directoryPath,
                outputDir: exportSettings.outputDir,
                trainRatio: exportSettings.trainRatio,
                valRatio: exportSettings.valRatio,
                testRatio: exportSettings.testRatio,
                shapeType: exportSettings.shapeType,
                includedLabels: includedLabels,
            });

            exportSuccess = `Export successful! ${result}`;
        } catch (err) {
            console.error("Export failed:", err);
            exportError = `Export failed: ${err}`;
        } finally {
            exportLoading = false;
        }
    }

    // --- Extract Handlers ---
    function openExtractModal() {
        if (!directoryPath) return;
        extractSettings.outputDir = "";
        showExtractModal = true;
        extractSuccess = "";
        extractError = "";
    }

    async function selectExtractDirectory() {
        const selected = await open({
            directory: true,
            multiple: false,
            title: "Select Extraction Output Directory",
        });
        if (selected) {
            extractSettings.outputDir = selected as string;
        }
    }

    function toggleLabelForExtract(event: CustomEvent) {
        const label = event.detail;
        if (selectedLabelsForExtract.has(label)) {
            selectedLabelsForExtract.delete(label);
            selectedLabelsForExtract = selectedLabelsForExtract;
        } else {
            selectedLabelsForExtract.add(label);
            selectedLabelsForExtract = selectedLabelsForExtract;
        }
    }

    async function runExtractLabels() {
        if (
            !directoryPath ||
            !extractSettings.outputDir ||
            selectedLabelsForExtract.size === 0
        )
            return;

        extractLoading = true;
        extractError = "";
        extractSuccess = "";

        try {
            const labelsArray = Array.from(selectedLabelsForExtract);
            const result = await invoke("extract_labels_from_dataset", {
                sourceDir: directoryPath,
                outputDir: extractSettings.outputDir,
                targetLabels: labelsArray,
            });
            extractSuccess = `Extraction successful! ${result}`;
        } catch (err) {
            console.error("Extraction failed:", err);
            extractError = `Extraction failed: ${err}`;
        } finally {
            extractLoading = false;
        }
    }

    // Keydown for navigation (optional, kept from original if needed, but Grid handles clicks)
    function handleKeydown(event: KeyboardEvent) {
        if (selectedImage) {
            if (event.key === "Escape") {
                selectedImage = null;
            }
        }
    }
</script>

<svelte:head>
    <title>Image Viewer</title>
</svelte:head>

<svelte:window on:keydown={handleKeydown} />

<div class="container mx-auto px-4 py-8">
    <div class="max-w-6xl mx-auto">
        <div class="mb-8">
            <h1 class="text-3xl font-bold text-gray-800 mb-6">Image Viewer</h1>

            <ViewerControls
                {directoryPath}
                {loading}
                imagesLength={images.length}
                {annotating}
                bind:annotationType
                {viewMode}
                on:selectDirectory={selectDirectory}
                on:annotate={annotateImages}
                on:changeViewMode={handleChangeViewMode}
                on:openExport={openExportModal}
                on:openExtract={openExtractModal}
            />

            <CropTool
                bind:isOpen={cropToolOpen}
                sourceDir={cropSourceDir}
                outputDir={cropOutputDir}
                bind:parentLabel
                loading={cropLoading}
                statusMessage={cropStatusMessage}
                isError={cropIsError}
                on:selectSource={() => selectCropDirectory("source")}
                on:selectOutput={() => selectCropDirectory("output")}
                on:runCrop={runCropAndRemap}
            />

            {#if error}
                <div class="bg-red-50 text-red-700 p-4 rounded-md mb-6">
                    {error}
                </div>
            {/if}

            <DatasetSummary summary={datasetSummary} />

            <div
                class={!directoryPath
                    ? "flex flex-col items-center justify-center py-16 px-6 border-2 border-dashed border-gray-300 rounded-lg mt-8"
                    : ""}
            >
                {#if !directoryPath && !loading}
                    <!-- Placeholder State (Replicated simply) -->
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
                        Select a directory containing your images to begin.
                    </p>
                    <div class="mt-6">
                        <button
                            type="button"
                            on:click={selectDirectory}
                            class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                        >
                            Select Directory
                        </button>
                    </div>
                {:else}
                    <div class="w-full">
                        {#if totalImages > 0 && !loading}
                            <div class="text-sm text-gray-600 mb-4">
                                Showing {Math.min(images.length, totalImages)} of
                                {totalImages} images
                            </div>
                        {/if}

                        <ImageGrid
                            {images}
                            {viewMode}
                            {loading}
                            {loadingMore}
                            {annotationType}
                            {currentPage}
                            {totalPages}
                            on:selectImage={handleSelectImage}
                            on:loadPage={(e) => loadImagesPage(e.detail)}
                        />
                    </div>
                {/if}
            </div>

            {#if loading && images.length === 0}
                <div class="flex justify-center items-center py-12">
                    <div
                        class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"
                    ></div>
                </div>
            {/if}
        </div>
    </div>
</div>

{#if selectedImage}
    <ImageViewerModal
        image={selectedImage}
        {annotationType}
        on:close={() => (selectedImage = null)}
    />
{/if}

<ExportModal
    isOpen={showExportModal}
    bind:settings={exportSettings}
    loading={exportLoading}
    error={exportError}
    success={exportSuccess}
    {datasetSummary}
    {excludedLabels}
    sourceDir={directoryPath}
    on:close={() => (showExportModal = false)}
    on:selectDir={selectExportDirectory}
    on:toggleExclusion={toggleLabelExclusion}
    on:export={exportToYolo}
/>

<ExtractLabelsModal
    isOpen={showExtractModal}
    bind:settings={extractSettings}
    loading={extractLoading}
    error={extractError}
    success={extractSuccess}
    {datasetSummary}
    selectedLabels={selectedLabelsForExtract}
    sourceDir={directoryPath}
    on:close={() => (showExtractModal = false)}
    on:selectDir={selectExtractDirectory}
    on:toggleLabel={toggleLabelForExtract}
    on:extract={runExtractLabels}
/>

<script lang="ts">
    import { onMount, tick } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { fade } from "svelte/transition";

    import ViewerControls from "./components/ViewerControls.svelte";
    import CropTool from "./components/CropTool.svelte";
    import DatasetSummary from "./components/DatasetSummary.svelte";
    import ImageGrid from "./components/ImageGrid.svelte";
    import ImageViewerModal from "./components/ImageViewerModal.svelte";
    import ExportModal from "./components/ExportModal.svelte";
    import ExtractLabelsModal from "./components/ExtractLabelsModal.svelte";

    let directoryPath = $state("");
    let images: any[] = $state([]);
    let loading = $state(false);
    let annotating = $state(false);
    let error = $state("");
    let containerElement: HTMLElement;
    let selectedImage: any = $state(null);
    let annotationType = $state("bounding_box");

    let currentPage = $state(1);
    let pageSize = 30;
    let totalImages = $state(0);
    let totalPages = $state(0);
    let loadingMore = $state(false);

    let viewMode: "grid" | "column" = $state("grid");
    let datasetSummary: any = $state(null);

    let cropToolOpen = $state(false);
    let cropSourceDir: string | null = $state(null);
    let cropOutputDir: string | null = $state(null);
    let parentLabel: string = $state("person");
    let cropLoading = $state(false);
    let cropStatusMessage: string | null = $state(null);
    let cropIsError = $state(false);

    let showExportModal = $state(false);
    let exportSettings = $state({ outputDir: "", trainRatio: 0.7, valRatio: 0.2, testRatio: 0.1, shapeType: "polygon" });
    let excludedLabels = $state(new Set<string>());
    let exportLoading = $state(false);
    let exportSuccess = $state("");
    let exportError = $state("");

    let showExtractModal = $state(false);
    let extractSettings = $state({ outputDir: "" });
    let selectedLabelsForExtract = $state(new Set<string>());
    let extractLoading = $state(false);
    let extractSuccess = $state("");
    let extractError = $state("");

    async function selectDirectory() {
        try {
            const selected = await open({ directory: true, multiple: false, title: "Select Dataset Directory" });
            if (selected) {
                directoryPath = selected as string;
                currentPage = 1;
                images = [];
                datasetSummary = null;
                error = "";
                await loadImagesPage(1);
            }
        } catch (err) {
            error = "Failed to select directory.";
        }
    }

    async function generateLabelMeSummary() {
        if (!directoryPath) return;
        try {
            datasetSummary = await invoke("get_dataset_summary", { datasetPath: directoryPath });
            excludedLabels = new Set();
            if (datasetSummary && datasetSummary.label_counts) {
                selectedLabelsForExtract = new Set(Object.keys(datasetSummary.label_counts));
            }
        } catch (err) { /* silent */ }
    }

    async function loadImagesPage(page: number) {
        if (!directoryPath) return;
        loading = true;
        error = "";
        loadingMore = page > 1;
        try {
            const response: any = await invoke("load_images_page", { directory: directoryPath, page, pageSize });
            if (page === 1) {
                images = [];
                totalImages = response.total_images;
                totalPages = response.total_pages;
                generateLabelMeSummary();
            }
            const newImages = await Promise.all(
                response.images.map(async (img: any) => ({
                    ...img,
                    previewUrl: convertFileSrc(img.path),
                    displayIndex: (page - 1) * pageSize + response.images.indexOf(img),
                })),
            );
            if (page === 1) { images = newImages; } else { images = [...images, ...newImages]; }
            currentPage = page;
        } catch (err) {
            error = `Failed to load images: ${err instanceof Error ? err.message : String(err)}`;
        } finally {
            loading = false;
            loadingMore = false;
        }
    }

    async function annotateImages() {
        if (!directoryPath) return;
        annotating = true;
        try {
            const annotationsMap: any = await invoke("load_annotations_for_images", { directory: directoryPath, annotationType });
            images = images.map((img) => {
                if (annotationsMap[img.path]) { return { ...img, annotations: annotationsMap[img.path], annotated: true }; }
                return img;
            });
        } catch (err) {
            error = "Failed to load annotations.";
        } finally {
            annotating = false;
        }
    }

    function handleSelectImage(detail: { image: any; index: number }) {
        selectedImage = detail.image;
    }

    function handleChangeViewMode(mode: string) {
        viewMode = mode as "grid" | "column";
    }

    async function selectCropDirectory(type: "source" | "output") {
        try {
            const selected = await open({ directory: true, multiple: false, title: type === "source" ? "Select Source Directory" : "Select Output Directory" });
            if (selected) {
                if (type === "source") { cropSourceDir = selected as string; } else { cropOutputDir = selected as string; }
                cropStatusMessage = null;
                cropIsError = false;
            }
        } catch (err) {
            cropStatusMessage = `Failed: ${err}`;
            cropIsError = true;
        }
    }

    async function runCropAndRemap() {
        if (!cropSourceDir || !cropOutputDir || !parentLabel) { cropStatusMessage = "Missing parameters."; cropIsError = true; return; }
        cropLoading = true; cropStatusMessage = null; cropIsError = false;
        try {
            const message = await invoke("crop_and_remap_annotations", { sourceDir: cropSourceDir, outputDir: cropOutputDir, parentLabel });
            cropStatusMessage = String(message); cropIsError = false;
            directoryPath = cropOutputDir; currentPage = 1; images = []; datasetSummary = null; selectedImage = null; error = "";
            await loadImagesPage(1);
        } catch (err) { cropStatusMessage = `Failed: ${err}`; cropIsError = true; }
        finally { cropLoading = false; }
    }

    function openExportModal() {
        if (!directoryPath) return;
        exportSettings.outputDir = "";
        showExportModal = true;
        exportSuccess = ""; exportError = "";
    }

    async function selectExportDirectory() {
        const selected = await open({ directory: true, multiple: false, title: "Select Export Output Directory" });
        if (selected) { exportSettings.outputDir = selected as string; }
    }

    function toggleLabelExclusion(label: string) {
        if (excludedLabels.has(label)) { excludedLabels.delete(label); } else { excludedLabels.add(label); }
        excludedLabels = new Set(excludedLabels);
    }

    async function exportToYolo() {
        if (!directoryPath || !exportSettings.outputDir) return;
        exportLoading = true; exportError = ""; exportSuccess = "";
        try {
            const allLabels = Object.keys(datasetSummary.label_counts);
            const includedLabels = allLabels.filter((l) => !excludedLabels.has(l));
            const result = await invoke("export_dataset_to_yolo", {
                sourceDir: directoryPath, outputDir: exportSettings.outputDir,
                trainRatio: exportSettings.trainRatio, valRatio: exportSettings.valRatio,
                testRatio: exportSettings.testRatio, shapeType: exportSettings.shapeType, includedLabels,
            });
            exportSuccess = `Export successful! ${result}`;
        } catch (err) { exportError = `Export failed: ${err}`; }
        finally { exportLoading = false; }
    }

    function openExtractModal() {
        if (!directoryPath) return;
        extractSettings.outputDir = "";
        showExtractModal = true;
        extractSuccess = ""; extractError = "";
    }

    async function selectExtractDirectory() {
        const selected = await open({ directory: true, multiple: false, title: "Select Extraction Output Directory" });
        if (selected) { extractSettings.outputDir = selected as string; }
    }

    function toggleLabelForExtract(label: string) {
        if (selectedLabelsForExtract.has(label)) { selectedLabelsForExtract.delete(label); } else { selectedLabelsForExtract.add(label); }
        selectedLabelsForExtract = new Set(selectedLabelsForExtract);
    }

    async function runExtractLabels() {
        if (!directoryPath || !extractSettings.outputDir || selectedLabelsForExtract.size === 0) return;
        extractLoading = true; extractError = ""; extractSuccess = "";
        try {
            const labelsArray = Array.from(selectedLabelsForExtract);
            const result = await invoke("extract_labels_from_dataset", { sourceDir: directoryPath, outputDir: extractSettings.outputDir, targetLabels: labelsArray });
            extractSuccess = `Extraction successful! ${result}`;
        } catch (err) { extractError = `Extraction failed: ${err}`; }
        finally { extractLoading = false; }
    }

    function handleKeydown(event: KeyboardEvent) {
        if (selectedImage) { if (event.key === "Escape") { selectedImage = null; } }
    }
</script>

<svelte:head><title>Image Viewer</title></svelte:head>

<svelte:window onkeydown={handleKeydown} />

<div class="container mx-auto px-4 py-8">
    <div class="max-w-6xl mx-auto">
        <div class="mb-8">
            <h1 class="text-3xl font-bold text-gray-800 mb-6">Image Viewer</h1>

            <ViewerControls
                {directoryPath} {loading} imagesLength={images.length} {annotating}
                bind:annotationType {viewMode}
                onselectdirectory={selectDirectory}
                onannotate={annotateImages}
                onchangeviewmode={handleChangeViewMode}
                onopenexport={openExportModal}
                onopenextract={openExtractModal}
            />

            <CropTool
                bind:isOpen={cropToolOpen}
                sourceDir={cropSourceDir} outputDir={cropOutputDir}
                bind:parentLabel loading={cropLoading}
                statusMessage={cropStatusMessage} isError={cropIsError}
                onselectsource={() => selectCropDirectory("source")}
                onselectoutput={() => selectCropDirectory("output")}
                onruncrop={runCropAndRemap}
            />

            {#if error}
                <div class="bg-red-50 text-red-700 p-4 rounded-md mb-6">{error}</div>
            {/if}

            <DatasetSummary summary={datasetSummary} />

            <div class={!directoryPath ? "flex flex-col items-center justify-center py-16 px-6 border-2 border-dashed border-gray-300 rounded-lg mt-8" : ""}>
                {#if !directoryPath && !loading}
                    <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                        <path vector-effect="non-scaling-stroke" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
                    </svg>
                    <h3 class="mt-2 text-lg font-medium text-gray-900">No Directory Selected</h3>
                    <p class="mt-1 text-sm text-gray-500">Select a directory containing your images to begin.</p>
                    <div class="mt-6">
                        <button type="button" onclick={selectDirectory} class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">Select Directory</button>
                    </div>
                {:else}
                    <div class="w-full">
                        {#if totalImages > 0 && !loading}
                            <div class="text-sm text-gray-600 mb-4">Showing {Math.min(images.length, totalImages)} of {totalImages} images</div>
                        {/if}
                        <ImageGrid {images} {viewMode} {loading} {loadingMore} {annotationType} {currentPage} {totalPages}
                            onselectimage={handleSelectImage}
                            onloadpage={(page) => loadImagesPage(page)}
                        />
                    </div>
                {/if}
            </div>

            {#if loading && images.length === 0}
                <div class="flex justify-center items-center py-12">
                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
                </div>
            {/if}
        </div>
    </div>
</div>

{#if selectedImage}
    <ImageViewerModal image={selectedImage} {annotationType} onclose={() => (selectedImage = null)} />
{/if}

<ExportModal
    isOpen={showExportModal} bind:settings={exportSettings}
    loading={exportLoading} error={exportError} success={exportSuccess}
    {datasetSummary} {excludedLabels} sourceDir={directoryPath}
    onclose={() => (showExportModal = false)}
    onselectdir={selectExportDirectory}
    ontoggleexclusion={toggleLabelExclusion}
    onexport={exportToYolo}
/>

<ExtractLabelsModal
    isOpen={showExtractModal} bind:settings={extractSettings}
    loading={extractLoading} error={extractError} success={extractSuccess}
    {datasetSummary} selectedLabels={selectedLabelsForExtract} sourceDir={directoryPath}
    onclose={() => (showExtractModal = false)}
    onselectdir={selectExtractDirectory}
    ontogglelabel={toggleLabelForExtract}
    onextract={runExtractLabels}
/>

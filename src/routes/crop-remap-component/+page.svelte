<script lang="ts">
    import {
        analyzeDataset,
        selectDirectory,
        autoPreviewAnnotatedImages,
        runProcessing,
        suggestParentLabel,
        suggestChildLabels,
        type DatasetSummary,
        type PreviewImage,
    } from "./services/dataService";

    import DirectorySelect from "./components/DirectorySelect.svelte";
    import DatasetSummaryCard from "./components/DatasetSummary.svelte";
    import PreviewGrid from "./components/PreviewGrid.svelte";
    import RemapSettings from "./components/RemapSettings.svelte";
    import KonvaPreviewModal from "./components/KonvaPreviewModal.svelte";
    import ActionButtons from "./components/ActionButtons.svelte";

    let sourceDir: string | null = null;
    let outputDir: string | null = null;

    let datasetSummary: DatasetSummary | null = null;
    let availableLabels: string[] = [];

    // Remap Settings
    let selectedParentLabel: string = "person";
    let selectedChildLabels: string[] = [];
    let paddingFactor: number = 1.2;

    // State
    let loading: boolean = false;
    let analyzing: boolean = false;
    let previewLoading: boolean = false;

    let successMessage: string | null = null;
    let errorMessage: string | null = null;
    let validationError: string | null = null;

    let previewImages: PreviewImage[] = [];
    let showPreviewModal: boolean = false;
    let previewModalImage: PreviewImage | null = null;

    async function handleSelectSource() {
        try {
            const dir = await selectDirectory("source");
            if (dir) {
                sourceDir = dir;
                resetState();
                await handleAnalyze();
            }
        } catch (err) {
            errorMessage = String(err);
        }
    }

    async function handleSelectOutput() {
        try {
            const dir = await selectDirectory("output");
            if (dir) outputDir = dir;
        } catch (err) {
            errorMessage = String(err);
        }
    }

    function resetState() {
        datasetSummary = null;
        availableLabels = [];
        selectedChildLabels = [];
        previewImages = [];
        successMessage = null;
        errorMessage = null;
        validationError = null;
    }

    async function handleAnalyze() {
        if (!sourceDir) return;
        analyzing = true;
        try {
            datasetSummary = await analyzeDataset(sourceDir);
            availableLabels = Object.keys(datasetSummary.label_counts || {});

            if (availableLabels.length > 0) {
                selectedParentLabel = suggestParentLabel(
                    availableLabels,
                    datasetSummary,
                );
                selectedChildLabels = suggestChildLabels(availableLabels);

                // Auto preview
                previewLoading = true;
                previewImages = await autoPreviewAnnotatedImages(
                    sourceDir,
                    datasetSummary,
                );
                previewLoading = false;
            } else {
                errorMessage = "No labels found in dataset.";
            }
        } catch (err) {
            errorMessage = `Analysis failed: ${err}`;
            previewLoading = false;
        } finally {
            analyzing = false;
        }
    }

    function openPreview(e: CustomEvent<PreviewImage>) {
        previewModalImage = e.detail;
        showPreviewModal = true;
    }

    function closePreview() {
        showPreviewModal = false;
        previewModalImage = null;
    }

    function validate(): boolean {
        validationError = null;
        if (!sourceDir || !outputDir) {
            validationError =
                "Please select both source and output directories.";
            return false;
        }
        if (!datasetSummary) {
            validationError = "Please wait for dataset analysis.";
            return false;
        }
        if (!selectedParentLabel) {
            validationError = "Please select a parent label.";
            return false;
        }
        if (selectedChildLabels.length === 0) {
            validationError = "Please select at least one child label.";
            return false;
        }
        return true;
    }

    async function handleRun() {
        if (!validate()) return;

        loading = true;
        successMessage = null;
        errorMessage = null;

        try {
            successMessage = await runProcessing(
                sourceDir!,
                outputDir!,
                selectedParentLabel,
                selectedChildLabels,
                paddingFactor,
            );
        } catch (err) {
            errorMessage = String(err);
        } finally {
            loading = false;
        }
    }
</script>

<svelte:head>
    <title>Crop & Remap Annotations</title>
</svelte:head>

<div class="container mx-auto px-4 py-8">
    <div class="max-w-2xl mx-auto bg-white p-6 rounded-lg shadow-md">
        <h1 class="text-2xl font-bold text-gray-800 mb-6">
            Crop and Remap Tool
        </h1>

        <div class="space-y-6 mb-6">
            <DirectorySelect
                label="Source Directory"
                value={sourceDir}
                on:browse={handleSelectSource}
            />

            {#if analyzing}
                <div class="text-blue-600 text-sm animate-pulse">
                    Analyzing dataset...
                </div>
            {/if}

            {#if datasetSummary}
                <DatasetSummaryCard summary={datasetSummary} />

                <PreviewGrid
                    images={previewImages}
                    loading={previewLoading}
                    on:select={openPreview}
                />
            {/if}

            <DirectorySelect
                label="Output Directory"
                value={outputDir}
                on:browse={handleSelectOutput}
            />

            {#if datasetSummary}
                <RemapSettings
                    summary={datasetSummary}
                    {availableLabels}
                    bind:selectedParentLabel
                    bind:selectedChildLabels
                    bind:paddingFactor
                />
            {/if}
        </div>

        <ActionButtons
            {loading}
            disabled={!datasetSummary}
            {successMessage}
            {errorMessage}
            {validationError}
            on:click={handleRun}
        />
    </div>
</div>

{#if showPreviewModal && previewModalImage}
    <KonvaPreviewModal image={previewModalImage} on:close={closePreview} />
{/if}

<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import DirectorySelector from "./components/DirectorySelector.svelte";
    import AnalysisSection from "./components/AnalysisSection.svelte";
    import StatsDisplay from "./components/StatsDisplay.svelte";
    import LabelConfigurator from "./components/LabelConfigurator.svelte";
    import StatusAlerts from "./components/StatusAlerts.svelte";

    let sourceDir: string | null = null;
    let outputDir: string | null = null;

    // Dynamic label selection
    let datasetSummary: any = null;
    let availableLabels: string[] = [];
    let selectedParentLabel: string = "person";
    let selectedChildLabels: string[] = [];
    let datasetLoaded: boolean = false;
    let analyzing: boolean = false;
    let paddingFactor: number = 1.2;

    let loading: boolean = false;
    let successMessage: string | null = null;
    let errorMessage: string | null = null;

    async function selectDirectory(type: "source" | "output") {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: `Select ${type === "source" ? "Source" : "Output"} Directory`,
            });

            if (selected && typeof selected === "string") {
                if (type === "source") {
                    sourceDir = selected;
                    datasetSummary = null;
                    availableLabels = [];
                    datasetLoaded = false;
                    selectedChildLabels = [];
                } else {
                    outputDir = selected;
                }
                successMessage = null;
                errorMessage = null;
            }
        } catch (err) {
            errorMessage = `Failed to select directory: ${err instanceof Error ? err.message : String(err)}`;
        }
    }

    function suggestParentLabel(labels: string[]): string {
        const commonParents = [
            "person",
            "people",
            "human",
            "worker",
            "individual",
        ];

        for (const parent of commonParents) {
            if (labels.includes(parent)) return parent;
        }

        if (datasetSummary?.label_counts) {
            const sortedLabels = Object.entries(datasetSummary.label_counts)
                .sort(([, a], [, b]) => (b as number) - (a as number))
                .map(([label]) => label);
            return sortedLabels[0] || "person";
        }

        return labels[0] || "person";
    }

    function suggestChildLabels(labels: string[]): string[] {
        const safetyEquipment = [
            "safety_helmet",
            "helmet",
            "hard_hat",
            "reflective_vest",
            "vest",
            "safety_vest",
            "body_harness",
            "harness",
            "safety_harness",
            "gloves",
            "safety_gloves",
            "boots",
            "safety_boots",
        ];

        return labels.filter((label) =>
            safetyEquipment.some((safety) =>
                label.toLowerCase().includes(safety.toLowerCase()),
            ),
        );
    }

    async function analyzeDataset() {
        if (!sourceDir) return;

        try {
            analyzing = true;
            errorMessage = null;

            const result = await invoke("get_labelme_summary", {
                path: sourceDir,
            });
            datasetSummary = JSON.parse(result);

            availableLabels = Object.keys(datasetSummary.label_counts || {});

            if (availableLabels.length === 0) {
                errorMessage =
                    "No labels found in the dataset. Please check if the directory contains LabelMe JSON files.";
                return;
            }

            selectedParentLabel = suggestParentLabel(availableLabels);
            selectedChildLabels = suggestChildLabels(availableLabels);
            datasetLoaded = true;
        } catch (err) {
            errorMessage = `Failed to analyze dataset: ${err instanceof Error ? err.message : String(err)}`;
            datasetSummary = null;
            availableLabels = [];
            datasetLoaded = false;
        } finally {
            analyzing = false;
        }
    }

    function validateSelection(): string | null {
        if (!selectedParentLabel) return "Please select a parent label";
        if (selectedChildLabels.length === 0)
            return "Please select at least one child label";

        const parentCount =
            datasetSummary?.label_counts[selectedParentLabel] || 0;
        if (parentCount === 0)
            return `No instances of '${selectedParentLabel}' found in dataset`;

        return null;
    }

    async function runProcessing() {
        if (!sourceDir || !outputDir) {
            errorMessage =
                "Please select source directory and output directory.";
            return;
        }

        if (!datasetLoaded) {
            errorMessage = "Please analyze the dataset first.";
            return;
        }

        const validationError = validateSelection();
        if (validationError) {
            errorMessage = validationError;
            return;
        }

        loading = true;
        successMessage = null;
        errorMessage = null;

        try {
            const message = await invoke("crop_and_remap_annotations", {
                sourceDir: sourceDir,
                outputDir: outputDir,
                parentLabel: selectedParentLabel,
                requiredChildLabelsStr: selectedChildLabels.join(","),
                paddingFactor: paddingFactor,
            });
            successMessage = String(message);
        } catch (err) {
            errorMessage = `Processing failed: ${err instanceof Error ? err.message : String(err)}`;
        } finally {
            loading = false;
        }
    }
</script>

<svelte:head>
    <title>Crop & Remap Annotations</title>
    <meta
        name="description"
        content="Advanced crop tool with dynamic multi-label safety equipment detection."
    />
</svelte:head>

<div class="container mx-auto px-4 py-8">
    <div class="max-w-7xl mx-auto">
        <!-- Page Header -->
        <div class="mb-10 text-center">
            <h1
                class="text-3xl font-bold mb-2 flex items-center justify-center gap-3"
            >
                <span class="material-symbols-rounded text-primary">crop</span>
                Crop and Remap Tool
            </h1>
            <p class="text-opacity-60 max-w-2xl mx-auto">
                Advanced annotation processing with dynamic multi-label
                detection based on your dataset
            </p>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <!-- Block 1: Source & Analysis (Top Left - Wide) -->
            <div
                class="card bg-base-100 border border-base-content/40 col-span-1 lg:col-span-2 shadow-none rounded-2xl"
            >
                <div class="card-body">
                    <h2 class="card-title text-lg mb-4 flex items-center gap-2">
                        <span class="material-symbols-rounded text-secondary"
                            >folder_open</span
                        >
                        Dataset Source
                    </h2>
                    <DirectorySelector
                        label="Source Directory"
                        placeholder="Select source directory..."
                        value={sourceDir}
                        on:browse={() => selectDirectory("source")}
                    />

                    <div class="divider my-2">Analysis</div>

                    <AnalysisSection
                        {analyzing}
                        {datasetLoaded}
                        disabled={!sourceDir || analyzing}
                        on:analyze={analyzeDataset}
                    />
                </div>
            </div>

            <!-- Block 2: Statistics (Top Right - Narrow) -->
            <div
                class="card bg-base-100 border border-base-content/40 col-span-1 shadow-none rounded-2xl"
            >
                <div class="card-body">
                    <h2 class="card-title text-lg mb-4 flex items-center gap-2">
                        <span class="material-symbols-rounded text-secondary"
                            >analytics</span
                        >
                        Statistics
                    </h2>
                    {#if datasetSummary}
                        <StatsDisplay {datasetSummary} />
                    {:else}
                        <div
                            class="flex flex-col items-center justify-center h-48 opacity-40 text-center"
                        >
                            <span class="material-symbols-rounded text-4xl mb-2"
                                >bar_chart</span
                            >
                            <span class="text-sm"
                                >Load dataset to view stats</span
                            >
                        </div>
                    {/if}
                </div>
            </div>

            <!-- Block 3: Configuration (Bottom Left - Wide) -->
            <div
                class="card bg-base-100 border border-base-content/40 col-span-1 lg:col-span-2 shadow-none rounded-2xl"
            >
                <div class="card-body">
                    <h2 class="card-title text-lg mb-4 flex items-center gap-2">
                        <span class="material-symbols-rounded text-secondary"
                            >tune</span
                        >
                        Configuration
                    </h2>

                    <LabelConfigurator
                        {datasetLoaded}
                        {availableLabels}
                        {datasetSummary}
                        bind:selectedParentLabel
                        bind:selectedChildLabels
                        bind:paddingFactor
                    />

                    <div class="divider my-4">Output</div>

                    <DirectorySelector
                        label="Output Directory"
                        placeholder="Select output directory..."
                        value={outputDir}
                        on:browse={() => selectDirectory("output")}
                    />
                </div>
            </div>

            <!-- Block 4: Actions (Bottom Right - Narrow) -->
            <div
                class="card bg-base-100 border border-base-content/40 col-span-1 flex flex-col shadow-none rounded-2xl"
            >
                <div class="card-body">
                    <h2 class="card-title text-lg mb-4 flex items-center gap-2">
                        <span class="material-symbols-rounded text-secondary"
                            >rocket_launch</span
                        >
                        Actions
                    </h2>

                    <div class="flex-1 flex flex-col justify-center">
                        <button
                            on:click={runProcessing}
                            disabled={loading ||
                                !sourceDir ||
                                !outputDir ||
                                !datasetLoaded ||
                                !selectedParentLabel ||
                                selectedChildLabels.length === 0}
                            class="btn btn-primary btn-block btn-lg"
                        >
                            {#if loading}
                                <span class="loading loading-spinner"></span>
                                Processing...
                            {:else}
                                <span class="material-symbols-rounded"
                                    >play_arrow</span
                                >
                                Run Crop & Remap
                            {/if}
                        </button>

                        {#if datasetLoaded && validateSelection()}
                            <div class="alert alert-error mt-4 text-sm">
                                <span class="material-symbols-rounded"
                                    >warning</span
                                >
                                <span>{validateSelection()}</span>
                            </div>
                        {/if}
                    </div>

                    <div class="mt-4">
                        <StatusAlerts {successMessage} {errorMessage} />
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

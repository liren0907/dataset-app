<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";

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

    function getFilteredChildLabels(): string[] {
        return availableLabels.filter((label) => label !== selectedParentLabel);
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
    <div class="max-w-2xl mx-auto">
        <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <h1 class="card-title text-2xl flex items-center gap-3">
                    <span class="material-symbols-rounded text-primary"
                        >crop</span
                    >
                    Crop and Remap Tool
                </h1>
                <p class="text-sm opacity-70 mb-6">
                    Advanced annotation processing with dynamic multi-label
                    detection based on your dataset
                </p>

                <div class="space-y-6">
                    <!-- Source Directory -->
                    <div class="form-control">
                        <label class="label">
                            <span class="label-text font-medium"
                                >Source Directory</span
                            >
                        </label>
                        <div class="join w-full">
                            <input
                                type="text"
                                readonly
                                placeholder="Select source directory..."
                                value={sourceDir || ""}
                                class="input input-bordered join-item flex-1"
                            />
                            <button
                                on:click={() => selectDirectory("source")}
                                class="btn btn-ghost join-item"
                            >
                                <span class="material-symbols-rounded"
                                    >folder_open</span
                                >
                            </button>
                        </div>
                    </div>

                    <!-- Dataset Analysis -->
                    <div class="form-control">
                        <label class="label">
                            <span class="label-text font-medium"
                                >Dataset Analysis</span
                            >
                        </label>
                        <div class="flex items-center gap-3">
                            <button
                                on:click={analyzeDataset}
                                disabled={!sourceDir || analyzing}
                                class="btn btn-primary btn-sm"
                            >
                                {#if analyzing}
                                    <span
                                        class="loading loading-spinner loading-sm"
                                    ></span>
                                    Analyzing...
                                {:else}
                                    <span
                                        class="material-symbols-rounded icon-sm"
                                        >analytics</span
                                    >
                                    Analyze Dataset
                                {/if}
                            </button>
                            {#if datasetLoaded}
                                <span class="badge badge-success gap-1">
                                    <span
                                        class="material-symbols-rounded icon-sm"
                                        >check_circle</span
                                    >
                                    Dataset analyzed
                                </span>
                            {/if}
                        </div>
                        <label class="label">
                            <span class="label-text-alt opacity-60"
                                >Analyze your dataset to discover available
                                labels and get smart suggestions.</span
                            >
                        </label>
                    </div>

                    <!-- Dataset Summary -->
                    {#if datasetSummary}
                        <div
                            class="stats stats-vertical lg:stats-horizontal shadow w-full bg-primary/10"
                        >
                            <div class="stat">
                                <div class="stat-figure text-primary">
                                    <span
                                        class="material-symbols-rounded text-2xl"
                                        >image</span
                                    >
                                </div>
                                <div class="stat-title">Images</div>
                                <div class="stat-value text-primary text-2xl">
                                    {datasetSummary.total_images}
                                </div>
                            </div>
                            <div class="stat">
                                <div class="stat-figure text-secondary">
                                    <span
                                        class="material-symbols-rounded text-2xl"
                                        >edit_note</span
                                    >
                                </div>
                                <div class="stat-title">Annotations</div>
                                <div class="stat-value text-secondary text-2xl">
                                    {datasetSummary.total_annotations}
                                </div>
                            </div>
                            <div class="stat">
                                <div class="stat-figure text-accent">
                                    <span
                                        class="material-symbols-rounded text-2xl"
                                        >label</span
                                    >
                                </div>
                                <div class="stat-title">Labels</div>
                                <div class="stat-value text-accent text-2xl">
                                    {datasetSummary.unique_labels}
                                </div>
                            </div>
                        </div>
                    {/if}

                    <!-- Output Directory -->
                    <div class="form-control">
                        <label class="label">
                            <span class="label-text font-medium"
                                >Output Directory</span
                            >
                        </label>
                        <div class="join w-full">
                            <input
                                type="text"
                                readonly
                                placeholder="Select output directory..."
                                value={outputDir || ""}
                                class="input input-bordered join-item flex-1"
                            />
                            <button
                                on:click={() => selectDirectory("output")}
                                class="btn btn-ghost join-item"
                            >
                                <span class="material-symbols-rounded"
                                    >folder_open</span
                                >
                            </button>
                        </div>
                    </div>

                    <!-- Dynamic Label Selection -->
                    {#if datasetLoaded && availableLabels.length > 0}
                        <!-- Parent Label -->
                        <div class="form-control">
                            <label class="label">
                                <span class="label-text font-medium"
                                    >Parent Label</span
                                >
                            </label>
                            <select
                                bind:value={selectedParentLabel}
                                class="select select-bordered w-full"
                            >
                                {#each availableLabels as label}
                                    <option value={label}>
                                        {label} ({datasetSummary?.label_counts[
                                            label
                                        ] || 0} annotations)
                                    </option>
                                {/each}
                            </select>
                            <label class="label">
                                <span class="label-text-alt opacity-60"
                                    >The label of the object to crop around.</span
                                >
                            </label>
                        </div>

                        <!-- Child Labels -->
                        <div class="form-control">
                            <label class="label">
                                <span class="label-text font-medium"
                                    >Required Child Labels</span
                                >
                            </label>
                            <div
                                class="max-h-48 overflow-y-auto space-y-2 p-4 bg-base-200 rounded-lg"
                            >
                                {#each getFilteredChildLabels() as label}
                                    <label
                                        class="flex items-center cursor-pointer gap-3"
                                    >
                                        <input
                                            type="checkbox"
                                            bind:group={selectedChildLabels}
                                            value={label}
                                            class="checkbox checkbox-primary checkbox-sm"
                                        />
                                        <span class="label-text">
                                            {label}
                                            <span
                                                class="badge badge-ghost badge-sm ml-2"
                                            >
                                                {datasetSummary?.label_counts[
                                                    label
                                                ] || 0}
                                            </span>
                                        </span>
                                    </label>
                                {/each}
                            </div>
                            <label class="label">
                                <span class="label-text-alt opacity-60">
                                    Only people wearing at least one of the
                                    selected items will be processed.
                                </span>
                            </label>
                            {#if selectedChildLabels.length > 0}
                                <div class="flex flex-wrap gap-1 mt-2">
                                    {#each selectedChildLabels as label}
                                        <span
                                            class="badge badge-primary badge-sm"
                                            >{label}</span
                                        >
                                    {/each}
                                </div>
                            {/if}
                        </div>

                        <!-- Padding Factor -->
                        <div class="form-control">
                            <label class="label">
                                <span class="label-text font-medium"
                                    >Padding Factor</span
                                >
                                <span
                                    class="label-text-alt badge badge-primary"
                                >
                                    {((paddingFactor - 1) * 100).toFixed(0)}% {paddingFactor >
                                    1
                                        ? "larger"
                                        : paddingFactor < 1
                                          ? "smaller"
                                          : "original"}
                                </span>
                            </label>
                            <div class="flex items-center gap-4">
                                <input
                                    type="range"
                                    bind:value={paddingFactor}
                                    min="0.5"
                                    max="2.0"
                                    step="0.1"
                                    class="range range-primary flex-1"
                                />
                                <input
                                    type="number"
                                    bind:value={paddingFactor}
                                    min="0.5"
                                    max="2.0"
                                    step="0.1"
                                    class="input input-bordered input-sm w-20 text-center"
                                />
                            </div>
                            <div
                                class="flex justify-between text-xs opacity-50 mt-1"
                            >
                                <span>0.5x (50% smaller)</span>
                                <span>1.0x (original)</span>
                                <span>2.0x (100% larger)</span>
                            </div>
                        </div>
                    {:else if !datasetLoaded}
                        <div class="alert alert-warning">
                            <span class="material-symbols-rounded">info</span>
                            <span
                                >Please analyze your dataset first to see
                                available labels and configure processing
                                options.</span
                            >
                        </div>
                    {/if}
                </div>

                <!-- Run Button -->
                <div class="card-actions mt-6">
                    <button
                        on:click={runProcessing}
                        disabled={loading ||
                            !sourceDir ||
                            !outputDir ||
                            !datasetLoaded ||
                            !selectedParentLabel ||
                            selectedChildLabels.length === 0}
                        class="btn btn-primary btn-block"
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
                        <div class="alert alert-error mt-2">
                            <span class="material-symbols-rounded">warning</span
                            >
                            <span>{validateSelection()}</span>
                        </div>
                    {/if}
                </div>

                <!-- Status Messages -->
                {#if successMessage}
                    <div class="alert alert-success mt-4">
                        <span class="material-symbols-rounded"
                            >check_circle</span
                        >
                        <div>
                            <h3 class="font-bold">Success!</h3>
                            <p class="text-sm">{successMessage}</p>
                        </div>
                    </div>
                {/if}
                {#if errorMessage}
                    <div class="alert alert-error mt-4">
                        <span class="material-symbols-rounded">error</span>
                        <div>
                            <h3 class="font-bold">Error!</h3>
                            <p class="text-sm">{errorMessage}</p>
                        </div>
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>

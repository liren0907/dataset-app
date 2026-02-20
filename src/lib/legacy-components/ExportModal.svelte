<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import { Button, BrowseInput } from "$lib/components/ui";

    export let showModal: boolean = false;
    export let currentDirectoryPath: string = "";
    export let currentDatasetSummary: any | null = null;

    const dispatch = createEventDispatcher();

    let exportMode: "yolo" | "labelme" = "yolo";
    let outputDir: string = "";
    let trainRatio: number = 0.7;
    let valRatio: number = 0.2;
    let testRatio: number = 0.1;
    let shapeType: "polygon" | "bounding_box" = "polygon";

    let internalExcludedLabels = new Set<string>();

    let localLoading: boolean = false;
    let localError: string = "";

    $: if (showModal) {
        localError = "";
    }

    async function selectOutputDirectory() {
        localError = "";
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select Export Output Directory",
            });

            if (selected && typeof selected === "string") {
                outputDir = selected;
            }
        } catch (err) {
            localError =
                "Failed to select output directory. " +
                (err instanceof Error ? err.message : String(err));
        }
    }

    function toggleLabelExclusion(label: string) {
        if (internalExcludedLabels.has(label)) {
            internalExcludedLabels.delete(label);
        } else {
            internalExcludedLabels.add(label);
        }
        internalExcludedLabels = internalExcludedLabels;
    }

    function handleRunExport() {
        if (!currentDirectoryPath) {
            localError = "Source directory path is missing.";
            return;
        }
        if (!outputDir) {
            localError = "Please select an output directory.";
            return;
        }

        const allLabels = currentDatasetSummary?.label_counts
            ? Object.keys(currentDatasetSummary.label_counts)
            : [];
        const includedLabelsArray = allLabels.filter(
            (label) => !internalExcludedLabels.has(label),
        );

        if (includedLabelsArray.length === 0 && allLabels.length > 0) {
            localError =
                "No labels selected for export. Please include at least one label.";
            return;
        }

        if (exportMode === "yolo") {
            const sum = trainRatio + valRatio + testRatio;
            if (Math.abs(sum - 1.0) > 0.015) {
                localError =
                    "Split ratios for YOLO must sum to 1.0. Current sum: " +
                    sum.toFixed(2);
                return;
            }
            if (
                trainRatio < 0 ||
                valRatio < 0 ||
                testRatio < 0 ||
                trainRatio > 1 ||
                valRatio > 1 ||
                testRatio > 1
            ) {
                localError = "Split ratios must be between 0 and 1.";
                return;
            }
        }
        localError = "";

        dispatch("runExport", {
            sourceDir: currentDirectoryPath,
            outputDir: outputDir,
            mode: exportMode,
            trainRatio: trainRatio,
            valRatio: valRatio,
            testRatio: testRatio,
            shapeType: shapeType,
            includedLabels: includedLabelsArray,
        });
    }

    function closeModalEvent() {
        dispatch("closeModal");
    }

    let availableLabelsForSelection: string[] = [];
    $: availableLabelsForSelection = currentDatasetSummary?.label_counts
        ? Object.keys(currentDatasetSummary.label_counts)
        : [];

    let effectivelyIncludedLabelsCount: number = 0;
    $: {
        const allLabels = currentDatasetSummary?.label_counts
            ? Object.keys(currentDatasetSummary.label_counts)
            : [];
        effectivelyIncludedLabelsCount = allLabels.filter(
            (label) => !internalExcludedLabels.has(label),
        ).length;
    }
</script>

<dialog class="modal" class:modal-open={showModal}>
    <div class="modal-box max-w-2xl">
        <!-- Header -->
        <div class="flex items-center justify-between mb-4">
            <h3 class="font-bold text-lg flex items-center gap-2">
                <span class="material-symbols-rounded text-primary">upload</span
                >
                Export Dataset
            </h3>
            <button
                on:click={closeModalEvent}
                class="btn btn-sm btn-circle btn-ghost"
            >
                <span class="material-symbols-rounded">close</span>
            </button>
        </div>

        <div class="space-y-4">
            {#if localError}
                <div class="alert alert-error">
                    <span class="material-symbols-rounded">error</span>
                    <span>{localError}</span>
                </div>
            {/if}

            <!-- Export Mode -->
            <div class="form-control">
                <label class="label"
                    ><span class="label-text font-medium">Export Mode</span
                    ></label
                >
                <div class="flex gap-4">
                    <label class="label cursor-pointer gap-2">
                        <input
                            type="radio"
                            name="exportMode"
                            value="yolo"
                            bind:group={exportMode}
                            class="radio radio-primary radio-sm"
                        />
                        <span class="label-text">YOLO Format</span>
                    </label>
                    <label class="label cursor-pointer gap-2">
                        <input
                            type="radio"
                            name="exportMode"
                            value="labelme"
                            bind:group={exportMode}
                            class="radio radio-primary radio-sm"
                        />
                        <span class="label-text">LabelMe JSON (Extract)</span>
                    </label>
                </div>
            </div>

            <!-- Source Directory -->
            <div class="form-control">
                <label class="label"
                    ><span class="label-text font-medium">Source Directory</span
                    ></label
                >
                <input
                    type="text"
                    value={currentDirectoryPath}
                    readonly
                    class="input input-bordered bg-base-200"
                />
            </div>

            <!-- Output Directory -->
            <div class="form-control">
                <label class="label"
                    ><span class="label-text font-medium">Output Directory</span
                    ></label
                >
                <BrowseInput
                    value={outputDir}
                    placeholder="Select output directory..."
                    on:browse={selectOutputDirectory}
                />
            </div>

            {#if exportMode === "yolo"}
                <!-- Shape Type -->
                <div class="form-control">
                    <label class="label"
                        ><span class="label-text font-medium">Shape Type</span
                        ></label
                    >
                    <select
                        bind:value={shapeType}
                        class="select select-bordered"
                    >
                        <option value="polygon">Polygon</option>
                        <option value="bounding_box">Bounding Box</option>
                    </select>
                </div>

                <!-- Split Ratios -->
                <div class="grid grid-cols-3 gap-4">
                    <div class="form-control">
                        <label class="label"
                            ><span class="label-text">Train Ratio</span></label
                        >
                        <input
                            type="number"
                            bind:value={trainRatio}
                            min="0"
                            max="1"
                            step="0.01"
                            class="input input-bordered input-sm"
                        />
                    </div>
                    <div class="form-control">
                        <label class="label"
                            ><span class="label-text">Val Ratio</span></label
                        >
                        <input
                            type="number"
                            bind:value={valRatio}
                            min="0"
                            max="1"
                            step="0.01"
                            class="input input-bordered input-sm"
                        />
                    </div>
                    <div class="form-control">
                        <label class="label"
                            ><span class="label-text">Test Ratio</span></label
                        >
                        <input
                            type="number"
                            bind:value={testRatio}
                            min="0"
                            max="1"
                            step="0.01"
                            class="input input-bordered input-sm"
                        />
                    </div>
                </div>
            {/if}

            <!-- Labels Selection -->
            <div class="form-control">
                <label class="label"
                    ><span class="label-text font-medium"
                        >Labels to Include</span
                    ></label
                >
                {#if currentDatasetSummary?.label_counts && availableLabelsForSelection.length > 0}
                    <p class="text-xs opacity-60 mb-2">
                        Click to toggle labels. Included labels are highlighted.
                    </p>
                    <div
                        class="flex flex-wrap gap-2 max-h-40 overflow-y-auto p-3 bg-base-200 rounded-lg"
                    >
                        {#each Object.entries(currentDatasetSummary.label_counts) as [label, count] (label)}
                            <button
                                type="button"
                                class={`badge cursor-pointer transition-all ${
                                    !internalExcludedLabels.has(label)
                                        ? "badge-primary"
                                        : "badge-ghost opacity-50 line-through"
                                }`}
                                on:click={() => toggleLabelExclusion(label)}
                            >
                                {label} ({count})
                            </button>
                        {/each}
                    </div>
                {:else if currentDatasetSummary && availableLabelsForSelection.length === 0}
                    <p class="text-sm opacity-60 italic">
                        No labels found. Load annotations first.
                    </p>
                {:else}
                    <p class="text-sm opacity-60 italic">
                        Dataset summary not available.
                    </p>
                {/if}
            </div>
        </div>

        <!-- Footer -->
        <div class="modal-action">
            <Button
                variant="ghost"
                on:click={closeModalEvent}
                disabled={localLoading}
            >
                Cancel
            </Button>
            <Button
                variant="primary"
                on:click={handleRunExport}
                disabled={localLoading ||
                    !outputDir ||
                    (availableLabelsForSelection.length > 0 &&
                        effectivelyIncludedLabelsCount === 0)}
            >
                {#if localLoading}
                    <span class="loading loading-spinner loading-sm"></span>
                    Processing...
                {:else}
                    <span class="material-symbols-rounded">upload</span>
                    Run Export
                {/if}
            </Button>
        </div>
    </div>
    <form method="dialog" class="modal-backdrop">
        <button on:click={closeModalEvent}>close</button>
    </form>
</dialog>

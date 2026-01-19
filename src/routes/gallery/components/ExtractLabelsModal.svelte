<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import type { DatasetSummary } from "../datasetService";

    // Props
    export let isOpen: boolean = false;
    export let sourceDir: string = "";
    export let datasetSummary: DatasetSummary | null = null;

    const dispatch = createEventDispatcher();

    // Internal state
    let outputDir: string = "";
    let excludedLabels = new Set<string>();
    let localLoading: boolean = false;
    let localError: string = "";

    // Reset error when modal opens
    $: if (isOpen) {
        localError = "";
    }

    // Get available labels
    $: availableLabels = datasetSummary?.label_counts
        ? Object.keys(datasetSummary.label_counts)
        : [];

    // Count included labels
    $: includedLabelsCount = availableLabels.filter(
        (label) => !excludedLabels.has(label),
    ).length;

    async function selectOutputDirectory() {
        localError = "";
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select Output Directory for Extracted Files",
            });

            if (selected && typeof selected === "string") {
                outputDir = selected;
            }
        } catch (err) {
            localError = `Failed to select directory: ${err instanceof Error ? err.message : String(err)}`;
        }
    }

    function toggleLabelExclusion(label: string) {
        if (excludedLabels.has(label)) {
            excludedLabels.delete(label);
        } else {
            excludedLabels.add(label);
        }
        excludedLabels = excludedLabels; // Trigger reactivity
    }

    function handleExtract() {
        if (!sourceDir) {
            localError = "Source directory is missing.";
            return;
        }
        if (!outputDir) {
            localError = "Please select an output directory.";
            return;
        }

        const includedLabels = availableLabels.filter(
            (label) => !excludedLabels.has(label),
        );

        if (includedLabels.length === 0 && availableLabels.length > 0) {
            localError =
                "No labels selected. Please include at least one label.";
            return;
        }

        localError = "";
        dispatch("extract", {
            sourceDir,
            outputDir,
            includedLabels,
        });
    }

    function handleClose() {
        dispatch("close");
    }

    // Allow parent to control loading state
    export function setLoading(value: boolean) {
        localLoading = value;
    }

    export function setError(message: string) {
        localError = message;
    }
</script>

{#if isOpen}
    <dialog
        class="modal modal-open"
        aria-modal="true"
        aria-labelledby="extract-modal-title"
    >
        <div class="modal-box max-w-2xl">
            <!-- Header -->
            <div class="flex justify-between items-center mb-4">
                <h3
                    id="extract-modal-title"
                    class="text-xl font-bold text-base-content"
                >
                    Extract Labels
                </h3>
                <button
                    on:click={handleClose}
                    class="btn btn-sm btn-circle btn-ghost"
                    aria-label="Close modal"
                    disabled={localLoading}>✕</button
                >
            </div>

            <!-- Content -->
            <div class="space-y-6">
                {#if localError}
                    <div class="alert alert-error">
                        <span class="material-symbols-rounded">error</span>
                        <span>{localError}</span>
                    </div>
                {/if}

                <!-- Source Directory -->
                <div class="form-control">
                    <label class="label" for="extractSourceDir">
                        <span class="label-text font-medium"
                            >Source Directory</span
                        >
                    </label>
                    <input
                        type="text"
                        id="extractSourceDir"
                        value={sourceDir}
                        readonly
                        class="input input-bordered bg-base-200 text-base-content/70"
                    />
                </div>

                <!-- Output Directory -->
                <div class="form-control">
                    <label class="label" for="extractOutputDir">
                        <span class="label-text font-medium"
                            >Output Directory</span
                        >
                    </label>
                    <div class="join w-full">
                        <input
                            type="text"
                            id="extractOutputDir"
                            bind:value={outputDir}
                            readonly
                            placeholder="Select output directory..."
                            class="input input-bordered join-item flex-1 bg-base-200 text-base-content/70"
                        />
                        <button
                            on:click={selectOutputDirectory}
                            class="btn btn-neutral join-item"
                            disabled={localLoading}
                        >
                            Browse...
                        </button>
                    </div>
                </div>

                <!-- Label Selection -->
                <div class="form-control">
                    <span class="label-text font-medium block mb-1"
                        >Labels to Extract</span
                    >
                    {#if datasetSummary?.label_counts && availableLabels.length > 0}
                        <p class="text-xs text-base-content/60 mb-2">
                            Click to toggle labels. By default, all are
                            included.
                        </p>
                        <div
                            class="flex flex-wrap gap-2 max-h-40 overflow-y-auto p-3 border border-base-300 rounded-lg bg-base-200"
                        >
                            {#each Object.entries(datasetSummary.label_counts) as [label, count] (label)}
                                <button
                                    type="button"
                                    class={`badge badge-lg cursor-pointer transition-all
                                        ${
                                            !excludedLabels.has(label)
                                                ? "badge-info"
                                                : "badge-ghost opacity-60 line-through"
                                        }`}
                                    on:click={() => toggleLabelExclusion(label)}
                                    disabled={localLoading}
                                >
                                    {label} ({count})
                                </button>
                            {/each}
                        </div>
                        <p class="text-xs text-base-content/50 mt-2">
                            {includedLabelsCount} of {availableLabels.length} labels
                            selected
                        </p>
                    {:else if datasetSummary && availableLabels.length === 0}
                        <p class="text-sm text-base-content/60 italic">
                            No labels found in the dataset summary.
                        </p>
                    {:else}
                        <p class="text-sm text-base-content/60 italic">
                            Dataset summary not available. Load a directory
                            first.
                        </p>
                    {/if}
                </div>
            </div>

            <!-- Footer -->
            <div class="modal-action">
                <button
                    type="button"
                    class="btn btn-ghost"
                    on:click={handleClose}
                    disabled={localLoading}
                >
                    Cancel
                </button>
                <button
                    type="button"
                    class="btn btn-ghost border border-neutral"
                    on:click={handleExtract}
                    disabled={localLoading ||
                        !outputDir ||
                        (availableLabels.length > 0 &&
                            includedLabelsCount === 0)}
                >
                    {#if localLoading}
                        <span class="loading loading-spinner loading-sm"></span>
                        Extracting...
                    {:else}
                        Run Extraction
                    {/if}
                </button>
            </div>
        </div>
        <form method="dialog" class="modal-backdrop">
            <button on:click={handleClose} disabled={localLoading}>close</button
            >
        </form>
    </dialog>
{/if}

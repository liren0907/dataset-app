<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import type { DatasetSummary } from "../datasetService";

    // New UI Components
    import SimpleModal from "$lib/components/ui/SimpleModal.svelte";
    import Alert from "$lib/components/ui/Alert.svelte";
    import Button from "$lib/components/ui/Button.svelte";
    import LabelBadge from "$lib/components/ui/LabelBadge.svelte";
    import BrowseInput from "$lib/components/ui/BrowseInput.svelte";
    import SectionLabel from "$lib/components/ui/SectionLabel.svelte";

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

<SimpleModal
    {isOpen}
    title="Extract Labels"
    maxWidth="max-w-2xl"
    on:close={handleClose}
>
    <div class="space-y-6">
        {#if localError}
            <Alert
                variant="error"
                dismissible
                on:close={() => (localError = "")}
            >
                {localError}
            </Alert>
        {/if}

        <!-- Source Directory -->
        <div>
            <SectionLabel>Source Directory</SectionLabel>
            <div class="px-1 py-2 opacity-70">
                <BrowseInput value={sourceDir} disabled />
            </div>
        </div>

        <!-- Output Directory -->
        <div>
            <SectionLabel>Output Directory</SectionLabel>
            <div class="px-1 py-2">
                <BrowseInput
                    value={outputDir}
                    placeholder="Select destination folder..."
                    disabled={localLoading}
                    on:browse={selectOutputDirectory}
                />
            </div>
        </div>

        <!-- Label Selection -->
        <div>
            <div class="flex justify-between items-end mb-2 px-1">
                <SectionLabel>Labels to Extract</SectionLabel>
                <span class="text-xs text-base-content/50">
                    {includedLabelsCount} of {availableLabels.length} selected
                </span>
            </div>

            {#if datasetSummary?.label_counts && availableLabels.length > 0}
                <div
                    class="flex flex-wrap gap-2 max-h-48 overflow-y-auto p-4 border border-base-200 rounded-lg bg-base-100/50"
                >
                    {#each Object.entries(datasetSummary.label_counts) as [label, count] (label)}
                        <LabelBadge
                            {label}
                            {count}
                            state={excludedLabels.has(label)
                                ? "excluded"
                                : "active"}
                            on:click={() => toggleLabelExclusion(label)}
                        />
                    {/each}
                </div>
            {:else if datasetSummary && availableLabels.length === 0}
                <div
                    class="p-4 border border-dashed border-base-300 rounded-lg text-center text-sm text-base-content/60"
                >
                    No labels found in the dataset.
                </div>
            {:else}
                <div
                    class="p-4 border border-dashed border-base-300 rounded-lg text-center text-sm text-base-content/60"
                >
                    Dataset summary not available.
                </div>
            {/if}
        </div>
    </div>

    <!-- Actions -->
    <div slot="actions" class="flex w-full justify-end gap-2">
        <Button variant="ghost" on:click={handleClose} disabled={localLoading}>
            Cancel
        </Button>
        <Button
            variant="default"
            on:click={handleExtract}
            disabled={localLoading ||
                !outputDir ||
                (availableLabels.length > 0 && includedLabelsCount === 0)}
        >
            {#if localLoading}
                <span class="loading loading-spinner loading-xs mr-2"></span>
                Extracting...
            {:else}
                Run Extraction
            {/if}
        </Button>
    </div>
</SimpleModal>

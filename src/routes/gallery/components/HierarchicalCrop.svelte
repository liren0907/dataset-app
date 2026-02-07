<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { createEventDispatcher } from "svelte";
    import { IconButton, Button, LabelBadge, Alert } from "$lib/components/ui";

    // Props
    export let currentDirectory: string = "";
    export let cropToolOpen: boolean = false;
    export let preSelectedParentLabel: string = "";

    const dispatch = createEventDispatcher();

    // State
    let datasetSummary: any = null;
    let availableLabels: string[] = [];
    let selectedParentLabel: string = "";
    let selectedChildLabels: string[] = [];
    let datasetLoaded: boolean = false;
    let analyzing: boolean = false;
    let paddingFactor: number = 1.2;
    let loading: boolean = false;
    let successMessage: string | null = null;
    let errorMessage: string | null = null;
    let processingMessage: string = "";
    let startTime: number = 0;

    // Auto-analyze when directory is available
    $: if (currentDirectory && cropToolOpen && !datasetLoaded && !analyzing) {
        analyzeDataset();
    }

    // Auto-select parent label when pre-selected
    $: if (
        preSelectedParentLabel &&
        availableLabels.includes(preSelectedParentLabel)
    ) {
        selectedParentLabel = preSelectedParentLabel;
    }

    async function analyzeDataset() {
        if (!currentDirectory) return;

        try {
            analyzing = true;
            errorMessage = null;

            const result = (await invoke("get_labelme_summary", {
                path: currentDirectory,
            })) as string;
            datasetSummary = JSON.parse(result);
            availableLabels = Object.keys(datasetSummary.label_counts || {});

            if (availableLabels.length === 0) {
                errorMessage = "No labels found in the dataset.";
                return;
            }

            // Smart defaults
            selectedParentLabel = suggestParentLabel(availableLabels);
            selectedChildLabels = suggestChildLabels(availableLabels);
            datasetLoaded = true;
        } catch (err) {
            errorMessage = `Failed to analyze: ${err instanceof Error ? err.message : String(err)}`;
            datasetLoaded = false;
        } finally {
            analyzing = false;
        }
    }

    function suggestParentLabel(labels: string[]): string {
        const commonParents = ["person", "people", "human", "worker"];
        for (const parent of commonParents) {
            if (labels.includes(parent)) return parent;
        }
        return labels[0] || "";
    }

    function suggestChildLabels(labels: string[]): string[] {
        const safetyEquipment = [
            "helmet",
            "vest",
            "harness",
            "gloves",
            "boots",
            "safety",
        ];
        return labels.filter((label) =>
            safetyEquipment.some((s) => label.toLowerCase().includes(s)),
        );
    }

    function toggleParent(label: string) {
        selectedParentLabel = label;
        // Reset child selection when parent changes
        selectedChildLabels = selectedChildLabels.filter((l) => l !== label);
    }

    function toggleChild(label: string) {
        if (selectedChildLabels.includes(label)) {
            selectedChildLabels = selectedChildLabels.filter(
                (l) => l !== label,
            );
        } else {
            selectedChildLabels = [...selectedChildLabels, label];
        }
    }

    function selectAllChildren() {
        selectedChildLabels = availableLabels.filter(
            (l) => l !== selectedParentLabel,
        );
    }

    function clearChildren() {
        selectedChildLabels = [];
    }

    function runCrop() {
        if (!currentDirectory || !selectedParentLabel) return;

        // Dispatch event with crop parameters - parent will handle background processing
        dispatch("cropStart", {
            sourceDir: currentDirectory,
            parentLabel: selectedParentLabel,
            childLabels: selectedChildLabels,
            paddingFactor: paddingFactor,
        });
    }

    $: canRun =
        datasetLoaded && selectedParentLabel && selectedChildLabels.length > 0;

    // Calculate estimated time based on parent label count
    $: estimatedTime = (() => {
        if (!datasetLoaded || !selectedParentLabel) return null;
        const count = datasetSummary?.label_counts?.[selectedParentLabel] || 0;
        const seconds = Math.ceil(count * 0.5); // ~0.5 sec per image
        if (seconds < 60) return `~${seconds} seconds`;
        const minutes = Math.ceil(seconds / 60);
        return `~${minutes} minute${minutes > 1 ? "s" : ""}`;
    })();
</script>

<!-- Hierarchical Crop Panel -->
<div class="card bg-base-100 border border-base-200 shadow-lg overflow-hidden">
    <!-- Header -->
    <div
        class="bg-gradient-to-r from-primary/10 to-secondary/10 px-6 py-4 border-b border-base-200"
    >
        <div class="flex items-center gap-3">
            <span class="material-symbols-rounded text-2xl text-primary"
                >account_tree</span
            >
            <div>
                <h3 class="font-bold text-lg text-base-content">
                    Hierarchical Crop
                </h3>
                <p class="text-sm text-base-content/60">
                    Crop by parent label and remap child annotations
                </p>
            </div>
        </div>
    </div>

    <div class="card-body p-6 gap-6">
        <!-- Loading State -->
        {#if analyzing}
            <div class="flex items-center justify-center gap-3 py-8">
                <span class="loading loading-spinner loading-md text-primary"
                ></span>
                <span class="text-base-content/70">Analyzing dataset...</span>
            </div>
        {:else if !datasetLoaded}
            <div class="text-center py-8 text-base-content/50">
                <span class="material-symbols-rounded text-4xl mb-2"
                    >folder_open</span
                >
                <p>
                    No dataset loaded. Select a directory in the gallery first.
                </p>
            </div>
        {:else}
            <!-- Step 1: Select Parent Label -->
            <div class="space-y-3">
                <div class="flex items-center gap-2">
                    <span class="badge badge-primary badge-lg font-bold">1</span
                    >
                    <span class="font-semibold text-base-content"
                        >Select Parent Label</span
                    >
                    <span class="text-xs text-base-content/50"
                        >(crop bounding box)</span
                    >
                </div>
                <div class="flex flex-wrap gap-2">
                    {#each availableLabels as label}
                        <LabelBadge
                            {label}
                            count={datasetSummary?.label_counts[label]}
                            state={selectedParentLabel === label
                                ? "active"
                                : "neutral"}
                            on:click={() => toggleParent(label)}
                        />
                    {/each}
                </div>
            </div>

            <!-- Step 2: Select Child Labels -->
            {#if selectedParentLabel}
                <div class="space-y-3">
                    <div class="flex items-center justify-between">
                        <div class="flex items-center gap-2">
                            <span
                                class="badge badge-secondary badge-lg font-bold"
                                >2</span
                            >
                            <span class="font-semibold text-base-content"
                                >Select Child Labels</span
                            >
                            <span class="text-xs text-base-content/50"
                                >(to keep in cropped images)</span
                            >
                        </div>
                        <div class="flex items-center gap-3">
                            <Button
                                variant="ghost"
                                size="sm"
                                on:click={selectAllChildren}
                            >
                                Select All
                            </Button>
                            <div class="w-px h-4 bg-base-300"></div>
                            <Button
                                variant="ghost"
                                size="sm"
                                on:click={clearChildren}
                                class="text-error/70 hover:text-error hover:bg-error/10"
                            >
                                Clear
                            </Button>
                        </div>
                    </div>
                    <div
                        class="p-4 rounded-xl bg-base-200/50 border border-base-200"
                    >
                        <div class="flex flex-wrap gap-2">
                            {#each availableLabels.filter((l) => l !== selectedParentLabel) as label}
                                <LabelBadge
                                    {label}
                                    count={datasetSummary?.label_counts[label]}
                                    state={selectedChildLabels.includes(label)
                                        ? "active"
                                        : "neutral"}
                                    on:click={() => toggleChild(label)}
                                />
                            {/each}
                        </div>
                        {#if selectedChildLabels.length > 0}
                            <div class="mt-3 text-xs text-base-content/60">
                                Selected: <span
                                    class="font-medium text-secondary"
                                    >{selectedChildLabels.join(", ")}</span
                                >
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}

            <!-- Step 3: Configure Padding -->
            {#if selectedParentLabel && selectedChildLabels.length > 0}
                <div class="space-y-4">
                    <div class="flex items-center gap-2">
                        <span class="badge badge-accent badge-lg font-bold"
                            >3</span
                        >
                        <span class="font-semibold text-base-content"
                            >Configure Padding</span
                        >
                    </div>

                    <!-- Padding Factor -->
                    <div class="form-control">
                        <div class="flex items-center justify-between mb-2">
                            <span class="text-sm font-medium"
                                >Padding Factor</span
                            >
                            <span class="badge badge-neutral font-mono text-xs">
                                {((paddingFactor - 1) * 100).toFixed(0)}% extra
                            </span>
                        </div>
                        <div class="flex items-center gap-4">
                            <input
                                type="range"
                                bind:value={paddingFactor}
                                min="1.0"
                                max="2.0"
                                step="0.1"
                                class="range range-primary range-sm flex-1"
                            />
                            <span class="font-mono text-sm w-12 text-center"
                                >{paddingFactor.toFixed(1)}x</span
                            >
                        </div>
                    </div>
                </div>
            {/if}

            <!-- Action Button -->
            <div class="pt-4 border-t border-base-200 space-y-3">
                <!-- Processing Message -->
                {#if loading && processingMessage}
                    <div
                        class="flex items-center gap-2 p-3 bg-primary/5 rounded-lg"
                    >
                        <span
                            class="loading loading-spinner loading-sm text-primary"
                        ></span>
                        <span class="text-sm text-primary font-medium"
                            >{processingMessage}</span
                        >
                    </div>
                {/if}

                <div class="flex items-center justify-between">
                    <div class="text-sm text-base-content/60">
                        {#if loading}
                            <span class="text-primary">Processing...</span>
                        {:else if datasetLoaded && selectedParentLabel}
                            <div class="flex flex-col gap-0.5">
                                <div>
                                    <span
                                        class="font-semibold text-base-content"
                                        >{datasetSummary?.label_counts?.[
                                            selectedParentLabel
                                        ] || 0}</span
                                    >
                                    <span class="font-medium text-primary"
                                        >{selectedParentLabel}</span
                                    >
                                    instances to crop
                                </div>
                                {#if estimatedTime && canRun}
                                    <div class="text-xs text-base-content/50">
                                        ⏱️ Estimated time: {estimatedTime}
                                    </div>
                                {/if}
                            </div>
                        {/if}
                    </div>
                    <IconButton
                        icon="play_circle"
                        label="Run Crop"
                        tooltip="Start cropping process"
                        variant="soft"
                        size="md"
                        disabled={!canRun}
                        {loading}
                        on:click={runCrop}
                    />
                </div>
            </div>
        {/if}

        <!-- Messages -->
        {#if successMessage}
            <Alert variant="success">{successMessage}</Alert>
        {/if}
        {#if errorMessage}
            <Alert variant="error">{errorMessage}</Alert>
        {/if}
    </div>
</div>

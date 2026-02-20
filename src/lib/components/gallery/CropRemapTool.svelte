<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import {
        performCropAndRemap,
        fetchDatasetSummary,
        type DatasetSummary,
    } from "$lib/services/gallery/datasetService";
    import {
        Button,
        BrowseInput,
        SectionLabel,
        SplitPaneModal,
        Alert,
        LabelBadge,
    } from "$lib/components/ui";

    export let isOpen: boolean = false;

    const dispatch = createEventDispatcher();

    // Internal state
    let sourceDir: string = "";
    let outputDir: string = "";

    // Config State
    let parentLabel: string = "person";
    let paddingFactor: number = 1.25;
    let selectedChildLabels: string[] = [];

    // Analysis State
    let analyzing: boolean = false;
    let datasetSummary: DatasetSummary | null = null;
    let availableLabels: string[] = [];

    // Process State
    let localLoading: boolean = false;
    let localError: string = "";
    let localSuccess: string = "";

    $: if (isOpen) {
        localError = "";
        localSuccess = "";
    }

    // Auto-analyze when source changes
    $: if (sourceDir && isOpen) {
        analyzeDataset();
    }

    async function selectDirectory(type: "source" | "output") {
        localError = "";
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: `Select ${type === "source" ? "Source" : "Output"} Directory`,
            });
            if (selected && typeof selected === "string") {
                if (type === "source") {
                    sourceDir = selected;
                    // Analysis triggers reactively
                } else {
                    outputDir = selected;
                }
            }
        } catch (err) {
            localError = `Failed to select directory.`;
        }
    }

    async function analyzeDataset() {
        if (!sourceDir) return;
        analyzing = true;
        datasetSummary = null;
        availableLabels = [];

        try {
            const summary = await fetchDatasetSummary(sourceDir);
            datasetSummary = summary;
            availableLabels = Object.keys(summary.label_counts || {});

            // Smart Suggestions
            suggestParentLabel(availableLabels);
            suggestChildLabels(availableLabels);
        } catch (err) {
            console.warn("Analysis failed:", err);
            // Don't block UI, just missing suggestions
        } finally {
            analyzing = false;
        }
    }

    function suggestParentLabel(labels: string[]) {
        const commonParents = ["person", "people", "worker", "human"];
        const found = commonParents.find((p) => labels.includes(p));
        if (found) {
            parentLabel = found;
        } else if (labels.length > 0) {
            // Pick most frequent
            if (datasetSummary?.label_counts) {
                const sorted = Object.entries(datasetSummary.label_counts)
                    .sort(([, a], [, b]) => b - a)
                    .map(([l]) => l);
                if (sorted.length > 0) parentLabel = sorted[0];
            }
        }
    }

    function suggestChildLabels(labels: string[]) {
        const safetyKeywords = [
            "helmet",
            "vest",
            "glove",
            "mask",
            "boots",
            "harness",
        ];
        selectedChildLabels = labels.filter((l) =>
            safetyKeywords.some((k) => l.toLowerCase().includes(k)),
        );
    }

    function toggleChildLabel(label: string) {
        if (selectedChildLabels.includes(label)) {
            selectedChildLabels = selectedChildLabels.filter(
                (l) => l !== label,
            );
        } else {
            selectedChildLabels = [...selectedChildLabels, label];
        }
    }

    async function handleRunCrop() {
        if (!sourceDir) {
            localError = "Please select a source directory.";
            return;
        }
        if (!outputDir) {
            localError = "Please select an output directory.";
            return;
        }
        if (!parentLabel.trim()) {
            localError = "Please enter a parent label.";
            return;
        }

        localLoading = true;
        localError = "";
        localSuccess = "";

        try {
            const message = await performCropAndRemap(
                sourceDir,
                outputDir,
                parentLabel,
                selectedChildLabels.join(","),
                paddingFactor,
            );
            localSuccess = message;
            setTimeout(() => {
                dispatch("cropCompleted", { outputDir });
            }, 1500);
        } catch (err) {
            localError = `Processing failed: ${err instanceof Error ? err.message : String(err)}`;
        } finally {
            localLoading = false;
        }
    }

    function handleClose() {
        if (!localLoading) dispatch("close");
    }
</script>

<SplitPaneModal
    show={isOpen}
    title="Crop & Remap"
    subtitle="Isolate objects to new dataset"
    icon="crop"
    on:close={handleClose}
>
    <!-- Sidebar -->
    <div slot="sidebar" class="flex flex-col h-full justify-between">
        <div class="space-y-6">
            <!-- Paths Configuration -->
            <div class="space-y-4">
                <SectionLabel>I/O Configuration</SectionLabel>

                <!-- Source Path -->
                <div>
                    <span
                        class="text-[10px] font-semibold text-base-content/50 mb-1 block pl-1"
                        >Source Dataset</span
                    >
                    <BrowseInput
                        value={sourceDir}
                        placeholder="Select source..."
                        icon="folder_open"
                        on:browse={() => selectDirectory("source")}
                    />
                </div>

                <!-- Output Path -->
                <div>
                    <span
                        class="text-[10px] font-semibold text-base-content/50 mb-1 block pl-1"
                        >Output Destination</span
                    >
                    <BrowseInput
                        value={outputDir}
                        placeholder="Select output..."
                        icon="output"
                        on:browse={() => selectDirectory("output")}
                    />
                </div>
            </div>
        </div>

        <!-- Info Block -->
        <div
            class="p-4 bg-base-100 rounded-xl border border-base-300/50 shadow-sm text-xs text-base-content/70 leading-relaxed"
        >
            <span class="font-bold block mb-1 text-base-content"
                >How it works</span
            >
            This tool creates a new dataset by cropping around the
            <span class="text-secondary font-medium">Parent Label</span>. All
            internal annotations are remapped to the new image coordinates.
        </div>
    </div>

    <!-- Main Content -->
    <div
        slot="content"
        class="flex flex-col items-center justify-center space-y-6 flex-1 w-full max-w-lg mx-auto"
    >
        {#if localError}
            <div class="w-full">
                <Alert
                    variant="error"
                    dismissible
                    on:close={() => (localError = "")}>{localError}</Alert
                >
            </div>
        {/if}

        {#if localSuccess}
            <div class="w-full">
                <Alert variant="success">{localSuccess}</Alert>
            </div>
        {/if}

        <!-- Operation Parameters Card -->
        <div
            class="w-full bg-base-100 border border-base-200 rounded-2xl shadow-sm overflow-hidden"
        >
            <div
                class="bg-base-200/50 px-6 py-3 border-b border-base-200 flex justify-between items-center"
            >
                <SectionLabel>Configuration</SectionLabel>
                {#if analyzing}
                    <span
                        class="loading loading-spinner loading-xs text-primary"
                    ></span>
                {/if}
            </div>

            <div class="p-6 space-y-8">
                <!-- 1. Parent Label Input -->
                <div class="form-control w-full">
                    <label class="label" for="parentLabel">
                        <span class="label-text font-semibold"
                            >Parent Object Class</span
                        >
                        <span class="label-text-alt text-base-content/50"
                            >Primary subject to crop</span
                        >
                    </label>
                    <div class="relative">
                        <input
                            id="parentLabel"
                            type="text"
                            bind:value={parentLabel}
                            placeholder="e.g. person"
                            class="input input-bordered w-full font-mono text-center focus:border-secondary transition-all"
                        />
                        <div
                            class="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none opacity-30"
                        >
                            <span class="material-symbols-rounded"
                                >person_search</span
                            >
                        </div>
                    </div>
                </div>

                <!-- 2. Padding Factor Slider -->
                <div class="form-control w-full">
                    <label class="label cursor-pointer" for="paddingFactor">
                        <span class="label-text font-semibold"
                            >Padding Factor</span
                        >
                        <span class="badge badge-neutral font-mono"
                            >{paddingFactor.toFixed(2)}x</span
                        >
                    </label>
                    <input
                        id="paddingFactor"
                        type="range"
                        min="0.5"
                        max="2.0"
                        step="0.05"
                        bind:value={paddingFactor}
                        class="range range-xs range-secondary"
                    />
                    <div
                        class="w-full flex justify-between text-[10px] px-1 mt-1 text-base-content/40 font-mono"
                    >
                        <span>Tight (0.5x)</span>
                        <span>Loose (2.0x)</span>
                    </div>
                </div>

                <!-- 3. Child Labels Selection -->
                <div class="form-control w-full">
                    <label class="label" for="childLabels">
                        <span class="label-text font-semibold"
                            >Required Child Classes</span
                        >
                        <span class="label-text-alt text-base-content/50">
                            {selectedChildLabels.length} selected
                        </span>
                    </label>

                    {#if availableLabels.length > 0}
                        <div
                            class="flex flex-wrap gap-2 min-h-[50px] max-h-[120px] overflow-y-auto p-2 border border-base-200 rounded-lg bg-base-100"
                        >
                            {#each availableLabels as label}
                                {#if label !== parentLabel}
                                    <LabelBadge
                                        {label}
                                        state={selectedChildLabels.includes(
                                            label,
                                        )
                                            ? "active"
                                            : "neutral"}
                                        on:click={() => toggleChildLabel(label)}
                                    />
                                {/if}
                            {/each}
                        </div>
                        <div class="label">
                            <span class="label-text-alt text-base-content/40"
                                >Only crop if the parent contains these objects</span
                            >
                        </div>
                    {:else if sourceDir && !analyzing}
                        <div
                            class="text-center py-4 border border-dashed border-base-300 rounded-lg text-xs text-base-content/50"
                        >
                            No labels found in source directory.
                        </div>
                    {:else if analyzing}
                        <div
                            class="text-center py-4 rounded-lg text-xs text-base-content/50"
                        >
                            Analyzing dataset labels...
                        </div>
                    {:else}
                        <div
                            class="text-center py-4 border border-dashed border-base-300 rounded-lg text-xs text-base-content/50"
                        >
                            Select a source directory to load labels.
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    </div>

    <!-- Footer -->
    <div slot="footer" class="flex gap-3">
        <Button variant="ghost" on:click={handleClose} disabled={localLoading}
            >Cancel</Button
        >
        <Button
            on:click={handleRunCrop}
            disabled={localLoading ||
                !sourceDir ||
                !outputDir ||
                !parentLabel.trim()}
            minWidth="140px"
        >
            {#if localLoading}
                <span class="loading loading-spinner loading-xs"></span>
            {:else}
                Start Processing
            {/if}
        </Button>
    </div>
</SplitPaneModal>

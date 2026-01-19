<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import {
        Button,
        BrowseInput,
        SelectionCard,
        SectionLabel,
        SplitPaneModal,
        Alert,
        LabelBadge,
    } from "$lib/components/ui";

    export let showModal: boolean = false;
    export let currentDirectoryPath: string = "";
    export let currentDatasetSummary: any | null = null;

    const dispatch = createEventDispatcher();

    // Internal state
    let exportMode: "yolo" | "labelme" = "yolo";
    let outputDir: string = "";
    let trainRatio: number = 0.7;
    let valRatio: number = 0.2;
    let testRatio: number = 0.1;
    let shapeType: "polygon" | "bounding_box" = "polygon";
    let internalExcludedLabels = new Set<string>();
    let localLoading: boolean = false;
    let localError: string = "";

    // Reset error on open
    $: if (showModal) {
        localError = "";
    }

    async function selectOutputDirectory() {
        localError = "";
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select Output Directory",
            });
            if (selected && typeof selected === "string") {
                outputDir = selected;
            }
        } catch (err) {
            localError = "Failed to select output directory.";
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
            localError = "Source directory missing.";
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
            localError = "Please include at least one label.";
            return;
        }

        if (exportMode === "yolo") {
            const sum = trainRatio + valRatio + testRatio;
            if (Math.abs(sum - 1.0) > 0.015) {
                localError = `Split ratios must sum to 1.0 (Current: ${sum.toFixed(2)})`;
                return;
            }
        }
        localError = "";

        // localLoading = true; // Typically we would start loading here, but the parent handles the action via dispatch
        dispatch("runExport", {
            sourceDir: currentDirectoryPath,
            outputDir: outputDir,
            mode: exportMode,
            trainRatio,
            valRatio,
            testRatio,
            shapeType,
            includedLabels: includedLabelsArray,
        });
    }

    let availableLabels: string[] = [];
    $: availableLabels = currentDatasetSummary?.label_counts
        ? Object.keys(currentDatasetSummary.label_counts)
        : [];
</script>

<SplitPaneModal
    show={showModal}
    title="Export Dataset"
    subtitle="Configure format & destination"
    icon="ios_share"
    on:close={() => dispatch("closeModal")}
>
    <!-- Left Sidebar -->
    <div slot="sidebar" class="flex flex-col gap-6 h-full">
        <!-- Export Format Selection -->
        <div class="flex flex-col gap-3 flex-1">
            <SectionLabel>Format</SectionLabel>

            <SelectionCard
                selected={exportMode === "yolo"}
                color="primary"
                on:select={() => (exportMode = "yolo")}
            >
                <span slot="icon" class="font-bold text-xs">YOLO</span>
                <span slot="title">YOLO Format</span>
                <span slot="description">For detection models</span>
            </SelectionCard>

            <SelectionCard
                selected={exportMode === "labelme"}
                color="secondary"
                on:select={() => (exportMode = "labelme")}
            >
                <span slot="icon" class="material-symbols-rounded text-sm"
                    >data_object</span
                >
                <span slot="title">LabelMe JSON</span>
                <span slot="description">Raw annotations</span>
            </SelectionCard>
        </div>

        <!-- Directories -->
        <div class="space-y-3">
            <SectionLabel>Paths</SectionLabel>

            <!-- Source Path (Compact) -->
            <div
                class="tooltip tooltip-right w-full"
                data-tip={currentDirectoryPath}
            >
                <div
                    class="flex items-center gap-2 px-3 py-2 bg-base-200 rounded-lg text-xs opacity-70"
                >
                    <span
                        class="material-symbols-rounded text-base-content/50 text-sm"
                        >folder_open</span
                    >
                    <span class="truncate"
                        >{currentDirectoryPath.split("/").pop()}</span
                    >
                </div>
            </div>

            <!-- Output Path -->
            <BrowseInput
                value={outputDir}
                placeholder="Select output..."
                icon="output"
                on:browse={selectOutputDirectory}
            />
        </div>
    </div>

    <!-- Main Content -->
    <div slot="content" class="space-y-8">
        {#if localError}
            <Alert
                variant="error"
                dismissible
                on:close={() => (localError = "")}
            >
                {localError}
            </Alert>
        {/if}

        <!-- Section: Configuration Details -->
        {#if exportMode === "yolo"}
            <div class="animate-in fade-in zoom-in-95 duration-200">
                <SectionLabel>Configuration</SectionLabel>

                <div
                    class="mt-4 grid grid-cols-2 gap-6 bg-base-200/30 p-5 rounded-xl border border-base-200"
                >
                    <!-- Shape Type -->
                    <div class="col-span-2">
                        <span
                            class="text-xs font-semibold text-base-content/60 mb-2 block"
                            >Annotation Type</span
                        >
                        <div class="join w-full grid grid-cols-2 h-9">
                            <button
                                class={`join-item btn btn-sm border-base-300 font-normal ${shapeType === "polygon" ? "btn-active btn-primary text-white" : "bg-base-100"}`}
                                on:click={() => (shapeType = "polygon")}
                            >
                                Polygon
                            </button>
                            <button
                                class={`join-item btn btn-sm border-base-300 font-normal ${shapeType === "bounding_box" ? "btn-active btn-primary text-white" : "bg-base-100"}`}
                                on:click={() => (shapeType = "bounding_box")}
                            >
                                Bounding Box
                            </button>
                        </div>
                    </div>

                    <!-- Split Ratios -->
                    <div class="col-span-2">
                        <span
                            class="text-xs font-semibold text-base-content/60 mb-2 block"
                            >Dataset Split</span
                        >
                        <div class="flex items-center gap-3">
                            <div
                                class="flex-1 flex flex-col items-center gap-1"
                            >
                                <input
                                    type="number"
                                    bind:value={trainRatio}
                                    class="input input-xs input-bordered w-full text-center font-mono"
                                    step="0.1"
                                />
                                <div
                                    class="w-full h-1 bg-green-500 rounded-full opacity-60"
                                ></div>
                                <span
                                    class="text-[10px] uppercase font-bold text-base-content/40"
                                    >Train</span
                                >
                            </div>
                            <div
                                class="flex-1 flex flex-col items-center gap-1"
                            >
                                <input
                                    type="number"
                                    bind:value={valRatio}
                                    class="input input-xs input-bordered w-full text-center font-mono"
                                    step="0.1"
                                />
                                <div
                                    class="w-full h-1 bg-blue-500 rounded-full opacity-60"
                                ></div>
                                <span
                                    class="text-[10px] uppercase font-bold text-base-content/40"
                                    >Val</span
                                >
                            </div>
                            <div
                                class="flex-1 flex flex-col items-center gap-1"
                            >
                                <input
                                    type="number"
                                    bind:value={testRatio}
                                    class="input input-xs input-bordered w-full text-center font-mono"
                                    step="0.1"
                                />
                                <div
                                    class="w-full h-1 bg-purple-500 rounded-full opacity-60"
                                ></div>
                                <span
                                    class="text-[10px] uppercase font-bold text-base-content/40"
                                    >Test</span
                                >
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        {:else}
            <div
                class="animate-in fade-in zoom-in-95 duration-200 h-32 flex items-center justify-center bg-base-200/30 rounded-xl border border-base-200 border-dashed"
            >
                <div class="text-center opacity-50">
                    <span class="material-symbols-rounded text-3xl"
                        >data_object</span
                    >
                    <p class="text-sm mt-1">Extracts raw LabelMe JSON files</p>
                </div>
            </div>
        {/if}

        <!-- Section: Class Filtering -->
        <div>
            <div class="flex justify-between items-center mb-4">
                <SectionLabel>Classes</SectionLabel>
                <span class="text-xs text-base-content/40">
                    {availableLabels.filter(
                        (l) => !internalExcludedLabels.has(l),
                    ).length} selected
                </span>
            </div>

            {#if availableLabels.length > 0}
                <div class="flex flex-wrap gap-2">
                    {#each availableLabels as label}
                        <LabelBadge
                            {label}
                            count={currentDatasetSummary?.label_counts[label]}
                            state={!internalExcludedLabels.has(label)
                                ? "active"
                                : "excluded"}
                            on:click={() => toggleLabelExclusion(label)}
                        />
                    {/each}
                </div>
            {:else}
                <div
                    class="text-center py-6 text-base-content/40 text-sm italic"
                >
                    No classes found in dataset.
                </div>
            {/if}
        </div>
    </div>

    <!-- Footer -->
    <div slot="footer" class="flex gap-3">
        <Button variant="ghost" on:click={() => dispatch("closeModal")}
            >Cancel</Button
        >
        <Button
            on:click={handleRunExport}
            disabled={localLoading || !outputDir}
            minWidth="140px"
        >
            {#if localLoading}
                <span class="loading loading-spinner loading-xs"></span>
            {:else}
                Export Data
            {/if}
        </Button>
    </div>
</SplitPaneModal>

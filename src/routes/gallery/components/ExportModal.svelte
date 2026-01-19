<script lang="ts">
    import { createEventDispatcher, onMount } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import {
        Button,
        BrowseInput,
        SelectionCard,
        SectionLabel,
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

{#if showModal}
    <dialog class="modal modal-open">
        <!-- Main Fixed-Size Container -->
        <div
            class="modal-box w-11/12 max-w-5xl h-[600px] p-0 flex overflow-hidden rounded-2xl shadow-2xl bg-base-100"
        >
            <!-- Left Sidebar (35%) -->
            <div
                class="w-[35%] bg-base-200/50 border-r border-base-200 flex flex-col p-6 gap-6"
            >
                <!-- Header -->
                <div>
                    <h2
                        class="text-xl font-bold text-base-content flex items-center gap-2"
                    >
                        <span class="material-symbols-rounded text-primary"
                            >ios_share</span
                        >
                        Export Dataset
                    </h2>
                    <p class="text-xs text-base-content/60 mt-1">
                        Configure format & destination
                    </p>
                </div>

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
                        <span
                            slot="icon"
                            class="material-symbols-rounded text-sm"
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

            <!-- Right Content Panel (65%) -->
            <div class="w-[65%] flex flex-col h-full bg-base-100 relative">
                <!-- Close Button -->
                <button
                    on:click={() => dispatch("closeModal")}
                    class="absolute top-4 right-4 z-10 btn btn-sm btn-circle btn-ghost text-base-content/40 hover:text-base-content"
                    >✕</button
                >

                <!-- Scrollable Content -->
                <div class="flex-1 overflow-y-auto p-8 space-y-8">
                    {#if localError}
                        <div
                            class="alert alert-error py-2 px-4 rounded-lg text-sm shadow-sm"
                        >
                            <span class="material-symbols-rounded text-lg"
                                >error</span
                            >
                            <span>{localError}</span>
                        </div>
                    {/if}

                    <!-- Section: Configuration Details -->
                    {#if exportMode === "yolo"}
                        <div class="animate-in fade-in zoom-in-95 duration-200">
                            <h3
                                class="text-sm font-bold text-base-content/70 uppercase tracking-wide mb-4 flex items-center gap-2"
                            >
                                <span class="w-1 h-4 bg-primary rounded-full"
                                ></span>
                                Configuration
                            </h3>

                            <div
                                class="grid grid-cols-2 gap-6 bg-base-200/30 p-5 rounded-xl border border-base-200"
                            >
                                <!-- Shape Type -->
                                <div class="col-span-2">
                                    <span
                                        class="text-xs font-semibold text-base-content/60 mb-2 block"
                                        >Annotation Type</span
                                    >
                                    <div
                                        class="join w-full grid grid-cols-2 h-9"
                                    >
                                        <button
                                            class={`join-item btn btn-sm border-base-300 font-normal ${shapeType === "polygon" ? "btn-active btn-primary text-white" : "bg-base-100"}`}
                                            on:click={() =>
                                                (shapeType = "polygon")}
                                        >
                                            Polygon
                                        </button>
                                        <button
                                            class={`join-item btn btn-sm border-base-300 font-normal ${shapeType === "bounding_box" ? "btn-active btn-primary text-white" : "bg-base-100"}`}
                                            on:click={() =>
                                                (shapeType = "bounding_box")}
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
                                <p class="text-sm mt-1">
                                    Extracts raw LabelMe JSON files
                                </p>
                            </div>
                        </div>
                    {/if}

                    <!-- Section: Class Filtering -->
                    <div>
                        <div class="flex justify-between items-center mb-4">
                            <h3
                                class="text-sm font-bold text-base-content/70 uppercase tracking-wide flex items-center gap-2"
                            >
                                <span class="w-1 h-4 bg-secondary rounded-full"
                                ></span>
                                Classes
                            </h3>
                            <span class="text-xs text-base-content/40">
                                {availableLabels.filter(
                                    (l) => !internalExcludedLabels.has(l),
                                ).length} selected
                            </span>
                        </div>

                        {#if availableLabels.length > 0}
                            <div class="flex flex-wrap gap-2">
                                {#each availableLabels as label}
                                    <button
                                        class={`px-2.5 py-1 rounded-md text-xs font-medium transition-all border select-none
                                        ${
                                            !internalExcludedLabels.has(label)
                                                ? "bg-primary/5 text-primary border-primary/20 hover:bg-primary/10"
                                                : "bg-base-200 text-base-content/30 border-transparent decoration-slice line-through opacity-60"
                                        }`}
                                        on:click={() =>
                                            toggleLabelExclusion(label)}
                                    >
                                        {label}
                                        <span
                                            class="ml-1 opacity-50 text-[10px]"
                                            >#{currentDatasetSummary
                                                ?.label_counts[label]}</span
                                        >
                                    </button>
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

                <!-- Footer Action -->
                <div
                    class="p-6 border-t border-base-100 bg-base-50/50 flex justify-end gap-3"
                >
                    <Button
                        variant="ghost"
                        on:click={() => dispatch("closeModal")}>Cancel</Button
                    >
                    <Button
                        on:click={handleRunExport}
                        disabled={localLoading || !outputDir}
                        minWidth="140px"
                    >
                        {#if localLoading}
                            <span class="loading loading-spinner loading-xs"
                            ></span>
                        {:else}
                            Export Data
                        {/if}
                    </Button>
                </div>
            </div>
        </div>
        <form
            method="dialog"
            class="modal-backdrop bg-base-300/50 backdrop-blur-sm"
        >
            <button on:click={() => dispatch("closeModal")}>close</button>
        </form>
    </dialog>
{/if}

<style>
    /* Custom scrollbar for better aesthetics inside the modal */
    .overflow-y-auto::-webkit-scrollbar {
        width: 6px;
    }
    .overflow-y-auto::-webkit-scrollbar-track {
        background: transparent;
    }
    .overflow-y-auto::-webkit-scrollbar-thumb {
        background-color: rgba(0, 0, 0, 0.1);
        border-radius: 10px;
    }
    :global(.dark) .overflow-y-auto::-webkit-scrollbar-thumb {
        background-color: rgba(255, 255, 255, 0.1);
    }
</style>

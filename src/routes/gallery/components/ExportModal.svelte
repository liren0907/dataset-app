<script lang="ts">
    import { createEventDispatcher, onMount } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";

    export let showModal: boolean = false;
    export let currentDirectoryPath: string = "";
    export let currentDatasetSummary: any | null = null;

    const dispatch = createEventDispatcher();

    // Internal state for the modal
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
            console.error("Error selecting export output directory:", err);
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

{#if showModal}
    <dialog
        class="modal modal-open"
        aria-modal="true"
        aria-labelledby="export-modal-title"
    >
        <div
            class="modal-box max-w-2xl bg-base-100 p-0 overflow-hidden relative"
        >
            <!-- Header Background Accent -->
            <div
                class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-primary to-secondary opacity-50"
            ></div>

            <!-- Header -->
            <div class="px-8 pt-8 pb-4 flex justify-between items-center">
                <div>
                    <h3
                        id="export-modal-title"
                        class="text-2xl font-bold text-base-content flex items-center gap-2"
                    >
                        Export Dataset
                    </h3>
                    <p class="text-sm text-base-content/60 mt-1">
                        Configure export settings and format
                    </p>
                </div>
                <button
                    on:click={closeModalEvent}
                    class="btn btn-sm btn-circle btn-ghost text-base-content/50 hover:text-base-content hover:bg-base-200 transition-colors"
                    aria-label="Close export modal">✕</button
                >
            </div>

            <!-- Content Scroll Area -->
            <div
                class="px-8 py-2 space-y-8 max-h-[65vh] overflow-y-auto custom-scrollbar"
            >
                {#if localError}
                    <div
                        class="alert alert-error rounded-xl shadow-sm border border-error/10"
                    >
                        <span class="material-symbols-rounded">error</span>
                        <span>{localError}</span>
                    </div>
                {/if}

                <!-- Export Mode: Card Selection -->
                <div class="form-control">
                    <label
                        class="label font-bold text-sm text-base-content/40 uppercase tracking-wider mb-2"
                    >
                        Export Format
                    </label>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        <!-- YOLO Card -->
                        <label class="cursor-pointer group relative">
                            <input
                                type="radio"
                                name="exportModeModal"
                                value="yolo"
                                bind:group={exportMode}
                                class="peer sr-only"
                            />
                            <div
                                class="p-4 rounded-xl border-2 border-base-200 bg-base-100 hover:border-primary/50 peer-checked:border-primary peer-checked:bg-primary/5 transition-all duration-200 h-full flex flex-col gap-2"
                            >
                                <div class="flex justify-between items-start">
                                    <div
                                        class="w-10 h-10 rounded-lg bg-primary/10 text-primary flex items-center justify-center"
                                    >
                                        <span class="font-bold text-sm"
                                            >YOLO</span
                                        >
                                    </div>
                                    <div
                                        class="w-5 h-5 rounded-full border-2 border-base-300 peer-checked:border-primary peer-checked:bg-primary flex items-center justify-center transition-all"
                                    >
                                        <div
                                            class="w-2 h-2 rounded-full bg-white opacity-0 peer-checked:opacity-100 transition-opacity"
                                        ></div>
                                    </div>
                                </div>
                                <div>
                                    <span
                                        class="font-bold text-base-content block"
                                        >YOLO Format</span
                                    >
                                    <span class="text-xs text-base-content/60"
                                        >For object detection & segmentation
                                        models</span
                                    >
                                </div>
                            </div>
                        </label>

                        <!-- LabelMe Card -->
                        <label class="cursor-pointer group relative">
                            <input
                                type="radio"
                                name="exportModeModal"
                                value="labelme"
                                bind:group={exportMode}
                                class="peer sr-only"
                            />
                            <div
                                class="p-4 rounded-xl border-2 border-base-200 bg-base-100 hover:border-primary/50 peer-checked:border-primary peer-checked:bg-primary/5 transition-all duration-200 h-full flex flex-col gap-2"
                            >
                                <div class="flex justify-between items-start">
                                    <div
                                        class="w-10 h-10 rounded-lg bg-secondary/10 text-secondary flex items-center justify-center"
                                    >
                                        <span
                                            class="material-symbols-rounded text-xl"
                                            >data_object</span
                                        >
                                    </div>
                                    <div
                                        class="w-5 h-5 rounded-full border-2 border-base-300 peer-checked:border-primary peer-checked:bg-primary flex items-center justify-center transition-all"
                                    >
                                        <div
                                            class="w-2 h-2 rounded-full bg-white opacity-0 peer-checked:opacity-100 transition-opacity"
                                        ></div>
                                    </div>
                                </div>
                                <div>
                                    <span
                                        class="font-bold text-base-content block"
                                        >LabelMe JSON</span
                                    >
                                    <span class="text-xs text-base-content/60"
                                        >Raw annotation extraction</span
                                    >
                                </div>
                            </div>
                        </label>
                    </div>
                </div>

                <!-- Directories Section -->
                <div class="space-y-4">
                    <label
                        class="label font-bold text-sm text-base-content/40 uppercase tracking-wider"
                    >
                        Paths Configuration
                    </label>

                    <!-- Source Directory (Read-only aesthetic) -->
                    <div class="relative group">
                        <div
                            class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none"
                        >
                            <span
                                class="material-symbols-rounded text-base-content/30 text-lg"
                                >folder_open</span
                            >
                        </div>
                        <input
                            type="text"
                            value={currentDirectoryPath}
                            readonly
                            class="input input-bordered w-full pl-10 bg-base-200/50 text-base-content/60 cursor-not-allowed rounded-lg focus:outline-none border-transparent"
                        />
                        <div
                            class="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none"
                        >
                            <span
                                class="text-xs text-base-content/40 font-medium bg-base-200 px-2 py-0.5 rounded"
                                >Source</span
                            >
                        </div>
                    </div>

                    <!-- Output Directory (Seamless Input Group) -->
                    <!-- Flex container acting as a single input box -->
                    <div
                        class="flex items-center w-full px-1 py-1 rounded-xl border border-base-300 bg-base-100 hover:border-base-content/30 focus-within:ring-2 focus-within:ring-primary/20 focus-within:border-primary transition-all shadow-sm"
                    >
                        <div class="pl-3 text-primary">
                            <span class="material-symbols-rounded">output</span>
                        </div>
                        <input
                            type="text"
                            bind:value={outputDir}
                            readonly
                            placeholder="Select output destination..."
                            class="input input-ghost w-full focus:outline-none border-none bg-transparent h-10 text-sm"
                        />
                        <button
                            on:click={selectOutputDirectory}
                            class="btn btn-sm btn-ghost bg-base-200/80 hover:bg-base-300 text-base-content/70 mr-1 rounded-lg font-medium"
                        >
                            Browse
                        </button>
                    </div>
                </div>

                {#if exportMode === "yolo"}
                    <div
                        class="p-5 bg-base-200/30 rounded-2xl border border-base-200"
                    >
                        <div class="flex items-center gap-2 mb-4">
                            <span class="material-symbols-rounded text-primary"
                                >settings_suggest</span
                            >
                            <span
                                class="font-bold text-sm uppercase tracking-wide opacity-80"
                                >YOLO Configuration</span
                            >
                        </div>

                        <!-- Shape Type -->
                        <div class="form-control mb-4">
                            <label
                                class="label text-xs font-bold text-base-content/50 uppercase"
                                >Annotation Type</label
                            >
                            <select
                                id="shapeTypeSelectModal"
                                bind:value={shapeType}
                                class="select select-bordered w-full bg-base-100 rounded-lg"
                            >
                                <option value="polygon"
                                    >Polygon (Segmentation)</option
                                >
                                <option value="bounding_box"
                                    >Bounding Box (Detection)</option
                                >
                            </select>
                        </div>

                        <!-- Split Ratios -->
                        <div>
                            <label
                                class="label text-xs font-bold text-base-content/50 uppercase mb-1"
                                >Dataset Split</label
                            >
                            <div class="flex gap-2">
                                <div
                                    class="flex-1 p-3 bg-base-100 rounded-lg border border-base-200 flex flex-col gap-1 items-center relative overflow-hidden group"
                                >
                                    <div
                                        class="absolute bottom-0 left-0 h-1 bg-green-500 w-full opacity-50"
                                    ></div>
                                    <span
                                        class="text-xs text-base-content/50 font-bold"
                                        >TRAIN</span
                                    >
                                    <input
                                        type="number"
                                        step="0.01"
                                        min="0"
                                        max="1"
                                        bind:value={trainRatio}
                                        class="w-full text-center font-mono font-bold bg-transparent focus:outline-none"
                                    />
                                </div>
                                <div
                                    class="flex-1 p-3 bg-base-100 rounded-lg border border-base-200 flex flex-col gap-1 items-center relative overflow-hidden group"
                                >
                                    <div
                                        class="absolute bottom-0 left-0 h-1 bg-blue-500 w-full opacity-50"
                                    ></div>
                                    <span
                                        class="text-xs text-base-content/50 font-bold"
                                        >VAL</span
                                    >
                                    <input
                                        type="number"
                                        step="0.01"
                                        min="0"
                                        max="1"
                                        bind:value={valRatio}
                                        class="w-full text-center font-mono font-bold bg-transparent focus:outline-none"
                                    />
                                </div>
                                <div
                                    class="flex-1 p-3 bg-base-100 rounded-lg border border-base-200 flex flex-col gap-1 items-center relative overflow-hidden group"
                                >
                                    <div
                                        class="absolute bottom-0 left-0 h-1 bg-purple-500 w-full opacity-50"
                                    ></div>
                                    <span
                                        class="text-xs text-base-content/50 font-bold"
                                        >TEST</span
                                    >
                                    <input
                                        type="number"
                                        step="0.01"
                                        min="0"
                                        max="1"
                                        bind:value={testRatio}
                                        class="w-full text-center font-mono font-bold bg-transparent focus:outline-none"
                                    />
                                </div>
                            </div>
                        </div>
                    </div>
                {/if}

                <!-- Label Selection -->
                <div class="form-control pb-4">
                    <div class="flex justify-between items-end mb-2">
                        <label
                            class="label font-bold text-sm text-base-content/40 uppercase tracking-wider p-0"
                        >
                            Class Filtering
                        </label>
                        {#if currentDatasetSummary?.label_counts}
                            <span class="text-xs text-base-content/40">
                                {effectivelyIncludedLabelsCount} / {availableLabelsForSelection.length}
                                classes included
                            </span>
                        {/if}
                    </div>

                    {#if currentDatasetSummary?.label_counts && availableLabelsForSelection.length > 0}
                        <div
                            class="flex flex-wrap gap-2 max-h-40 overflow-y-auto p-1"
                        >
                            {#each Object.entries(currentDatasetSummary.label_counts) as [label, count] (label)}
                                <button
                                    type="button"
                                    class={`px-3 py-1.5 rounded-full text-xs font-bold transition-all border
                                        ${
                                            !internalExcludedLabels.has(label)
                                                ? "bg-primary/10 text-primary border-primary/20 hover:bg-primary/20 hover:border-primary/40"
                                                : "bg-base-200 text-base-content/30 border-transparent decoration-slice line-through hover:bg-base-300"
                                        }`}
                                    on:click={() => toggleLabelExclusion(label)}
                                >
                                    {label}
                                    <span class="opacity-60 ml-1 font-normal"
                                        >#{count}</span
                                    >
                                </button>
                            {/each}
                        </div>
                    {:else if currentDatasetSummary && availableLabelsForSelection.length === 0}
                        <div
                            class="p-8 text-center border-2 border-dashed border-base-300 rounded-xl bg-base-100/50 text-base-content/50"
                        >
                            <span
                                class="material-symbols-rounded text-3xl mb-2 block opacity-50"
                                >data_array</span
                            >
                            <span>No classes found in dataset</span>
                        </div>
                    {:else}
                        <div
                            class="p-8 flex justify-center items-center border border-base-200 rounded-xl bg-base-100/50"
                        >
                            <span class="loading loading-spinner text-primary"
                            ></span>
                        </div>
                    {/if}
                </div>
            </div>

            <!-- Footer -->
            <div
                class="px-8 py-5 bg-base-100 border-t border-base-100 flex justify-end gap-3 sticky bottom-0 z-10 w-full"
            >
                <button
                    type="button"
                    class="btn btn-ghost hover:bg-base-200 font-medium text-base-content/70"
                    on:click={closeModalEvent}
                >
                    Cancel
                </button>
                <button
                    type="button"
                    class="btn btn-primary min-w-[140px] shadow-lg shadow-primary/20 border-none hover:bg-primary-focus hover:scale-[1.02] active:scale-[0.98] transition-all"
                    on:click={handleRunExport}
                    disabled={localLoading ||
                        !outputDir ||
                        (availableLabelsForSelection.length > 0 &&
                            effectivelyIncludedLabelsCount === 0)}
                >
                    {#if localLoading}
                        <span
                            class="loading loading-spinner loading-sm text-primary-content"
                        ></span>
                    {:else}
                        <span>Export</span>
                        <span class="material-symbols-rounded text-lg"
                            >arrow_forward</span
                        >
                    {/if}
                </button>
            </div>
        </div>
        <form
            method="dialog"
            class="modal-backdrop bg-base-300/50 backdrop-blur-sm"
        >
            <button on:click={closeModalEvent}>close</button>
        </form>
    </dialog>
{/if}

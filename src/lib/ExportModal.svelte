<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import { open } from "@tauri-apps/plugin-dialog";

    export let showModal: boolean = false;
    export let currentDirectoryPath: string = "";
    export let currentDatasetSummary: any | null = null;

    const dispatch = createEventDispatcher();

    // Internal state for the modal
    let exportMode: 'yolo' | 'labelme' = 'yolo';
    let outputDir: string = "";
    let trainRatio: number = 0.7;
    let valRatio: number = 0.2;
    let testRatio: number = 0.1;
    let shapeType: "polygon" | "bounding_box" = "polygon";
    
    let internalExcludedLabels = new Set<string>();
    
    let localLoading: boolean = false; // For UI elements within the modal, like directory selection
    let localError: string = "";
    // No localSuccess needed here as parent page shows global success/error for the actual export.

    // Reactive statement to reset some fields when modal becomes visible
    $: if (showModal) {
        localError = ""; // Clear local errors on open
        // Consider if outputDir or internalExcludedLabels should be reset here or managed by parent.
        // For now, they persist if modal is closed and reopened, unless parent changes props.
    }

    async function selectOutputDirectory() {
        localError = "";
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select Export Output Directory",
            });

            if (selected && typeof selected === 'string') {
                outputDir = selected;
            }
        } catch (err) {
            console.error("Error selecting export output directory:", err);
            localError = "Failed to select output directory. " + (err instanceof Error ? err.message : String(err));
        }
    }

    function toggleLabelExclusion(label: string) {
        if (internalExcludedLabels.has(label)) {
            internalExcludedLabels.delete(label);
        } else {
            internalExcludedLabels.add(label);
        }
        internalExcludedLabels = internalExcludedLabels; // Trigger Svelte reactivity
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

        const allLabels = currentDatasetSummary?.label_counts ? Object.keys(currentDatasetSummary.label_counts) : [];
        const includedLabelsArray = allLabels.filter(label => !internalExcludedLabels.has(label));

        if (includedLabelsArray.length === 0 && allLabels.length > 0) {
            localError = "No labels selected for export. Please include at least one label.";
            return;
        }
        
        if (exportMode === 'yolo') {
            const sum = trainRatio + valRatio + testRatio;
            if (Math.abs(sum - 1.0) > 0.015) { 
                localError = "Split ratios for YOLO must sum to 1.0. Current sum: " + sum.toFixed(2);
                return;
            }
             if (trainRatio < 0 || valRatio < 0 || testRatio < 0 || trainRatio > 1 || valRatio > 1 || testRatio > 1 ) {
                localError = "Split ratios must be between 0 and 1.";
                return;
            }
        }
        localError = ""; // Clear local errors before dispatching

        dispatch('runExport', {
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

    function closeModalEvent() { // Renamed to avoid conflict with potential prop named 'closeModal'
        dispatch('closeModal');
    }

    let availableLabelsForSelection: string[] = [];
    $: availableLabelsForSelection = currentDatasetSummary?.label_counts ? Object.keys(currentDatasetSummary.label_counts) : [];
    
    let effectivelyIncludedLabelsCount: number = 0;
    $: {
        const allLabels = currentDatasetSummary?.label_counts ? Object.keys(currentDatasetSummary.label_counts) : [];
        effectivelyIncludedLabelsCount = allLabels.filter(label => !internalExcludedLabels.has(label)).length;
    }

</script>

{#if showModal}
    <div
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
        on:click|self={closeModalEvent} 
        role="dialog"
        aria-modal="true"
        aria-labelledby="export-modal-title"
    >
        <div class="max-w-2xl w-full bg-white rounded-lg shadow-xl overflow-hidden flex flex-col max-h-[90vh]">
            <div class="px-6 py-4 border-b flex justify-between items-center">
                <h3 id="export-modal-title" class="text-xl font-bold text-gray-800">Export Dataset</h3>
                <button on:click={closeModalEvent} class="text-gray-400 hover:text-gray-600 p-1 -mr-1" aria-label="Close export modal">
                     <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>

            <div class="px-6 py-4 overflow-y-auto space-y-6">
                {#if localError}
                    <div class="bg-red-50 text-red-700 p-3 rounded-md text-sm">
                        {localError}
                    </div>
                {/if}

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">Export Mode</label>
                    <div class="flex items-center space-x-4">
                        <div>
                            <input type="radio" id="exportModeYoloModal" name="exportModeModal" value="yolo" bind:group={exportMode} class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 cursor-pointer">
                            <label for="exportModeYoloModal" class="ml-2 text-sm text-gray-700 cursor-pointer">YOLO Format</label>
                        </div>
                        <div>
                            <input type="radio" id="exportModeLabelMeModal" name="exportModeModal" value="labelme" bind:group={exportMode} class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 cursor-pointer">
                            <label for="exportModeLabelMeModal" class="ml-2 text-sm text-gray-700 cursor-pointer">LabelMe JSON (Extract)</label>
                        </div>
                    </div>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="sourceDirInputModal">Source Directory</label>
                    <input
                        type="text"
                        id="sourceDirInputModal" 
                        value={currentDirectoryPath}
                        readonly
                        class="w-full px-3 py-2 border border-gray-300 rounded-md bg-gray-100 text-gray-600 text-sm"
                    />
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1" for="outputDirInputModal">Output Directory</label>
                    <div class="flex gap-2">
                        <input
                            type="text"
                            id="outputDirInputModal"
                            bind:value={outputDir}
                            readonly
                            placeholder="Select output directory..."
                            class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-100 text-gray-600 text-sm"
                        />
                        <button
                            on:click={selectOutputDirectory} 
                            class="px-4 py-2 bg-gray-200 hover:bg-gray-300 text-gray-800 rounded-md border border-gray-400 text-sm shadow-sm"
                        >
                            Browse...
                        </button>
                    </div>
                </div>

                {#if exportMode === 'yolo'}
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1" for="shapeTypeSelectModal">Shape Type (for YOLO conversion)</label>
                        <select
                            id="shapeTypeSelectModal"
                            bind:value={shapeType}
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500 text-sm"
                        >
                            <option value="polygon">Polygon</option>
                            <option value="bounding_box">Bounding Box</option>
                        </select>
                    </div>

                    <div class="grid grid-cols-3 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1" for="trainRatioInputModal">Train Ratio</label>
                            <input
                                type="number"
                                id="trainRatioInputModal"
                                bind:value={trainRatio}
                                min="0"
                                max="1"
                                step="0.01"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500 text-sm"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1" for="valRatioInputModal">Validation Ratio</label>
                            <input
                                type="number"
                                id="valRatioInputModal"
                                bind:value={valRatio}
                                min="0"
                                max="1"
                                step="0.01"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500 text-sm"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1" for="testRatioInputModal">Test Ratio</label>
                            <input
                                type="number"
                                id="testRatioInputModal"
                                bind:value={testRatio}
                                min="0"
                                max="1"
                                step="0.01"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500 text-sm"
                            />
                        </div>
                    </div>
                {/if}

                <div>
                    <label class="block text-gray-700 mb-2 font-medium text-sm">Labels to Include in Export</label>
                    {#if currentDatasetSummary?.label_counts && availableLabelsForSelection.length > 0}
                        <p class="text-xs text-gray-500 mb-2">Select labels to include. By default, all are included.</p>
                        <div class="flex flex-wrap gap-2 max-h-40 overflow-y-auto p-2 border rounded-md bg-gray-50">
                            {#each Object.entries(currentDatasetSummary.label_counts) as [label, count] (label)}
                                <button
                                    type="button"
                                    class={`px-2.5 py-1 rounded-full text-xs font-medium border transition-colors duration-150 shadow-sm
                                        ${!internalExcludedLabels.has(label)
                                            ? 'bg-sky-100 text-sky-800 border-sky-300 hover:bg-sky-200'
                                            : 'bg-gray-100 text-gray-500 border-gray-300 hover:bg-gray-200 line-through opacity-80'}`}
                                    on:click={() => toggleLabelExclusion(label)}
                                >
                                    {label} ({count})
                                </button>
                            {/each}
                        </div>
                    {:else if currentDatasetSummary && availableLabelsForSelection.length === 0}
                         <p class="text-sm text-gray-500 italic">No labels found in the dataset summary to select. Annotations might need to be loaded or processed.</p>
                    {:else}
                         <p class="text-sm text-gray-500 italic">Dataset summary not yet available. Load a directory and annotations first.</p>
                    {/if}
                </div>
            </div>

            <div class="px-6 py-4 border-t bg-gray-50 flex justify-end space-x-3 mt-auto">
                <button
                    type="button"
                    class="px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-100 text-gray-700 text-sm shadow-sm"
                    on:click={closeModalEvent}
                    disabled={localLoading} 
                >
                    Cancel
                </button>
                <button
                    type="button"
                    class="bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-md disabled:opacity-60 disabled:cursor-not-allowed flex items-center text-sm shadow-sm"
                    on:click={handleRunExport} 
                    disabled={localLoading || !outputDir || (availableLabelsForSelection.length > 0 && effectivelyIncludedLabelsCount === 0) }
                >
                    {#if localLoading}
                        <div class="mr-2 animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"></div>
                        Processing...
                    {:else}
                        Run Export
                    {/if}
                </button>
            </div>
        </div>
    </div>
{/if} 
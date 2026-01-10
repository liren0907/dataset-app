<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let isOpen: boolean;
    export let settings: {
        outputDir: string;
        trainRatio: number;
        valRatio: number;
        testRatio: number;
        shapeType: string;
    };
    export let loading: boolean;
    export let error: string;
    export let success: string;
    export let datasetSummary: any;
    export let excludedLabels: Set<string>;
    export let sourceDir: string;

    const dispatch = createEventDispatcher();
</script>

{#if isOpen}
    <div
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
    >
        <div
            class="max-w-2xl w-full bg-white rounded-lg shadow-xl overflow-hidden flex flex-col max-h-[90vh]"
        >
            <!-- Header -->
            <div class="px-6 py-4 border-b">
                <h3 class="text-xl font-bold">Export to YOLO Format</h3>
            </div>

            <!-- Form -->
            <div class="px-6 py-4 overflow-y-auto">
                {#if success}
                    <div class="bg-green-50 text-green-700 p-3 rounded-md mb-4">
                        {success}
                    </div>
                {/if}
                {#if error}
                    <div class="bg-red-50 text-red-700 p-3 rounded-md mb-4">
                        {error}
                    </div>
                {/if}

                <!-- Source Directory -->
                <div class="mb-4">
                    <label
                        class="block text-sm font-medium text-gray-700 mb-1"
                        for="sourceDirInput">Source Directory</label
                    >
                    <input
                        type="text"
                        id="sourceDirInput"
                        value={sourceDir}
                        readonly
                        class="w-full px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-500"
                    />
                </div>

                <!-- Output Directory -->
                <div class="mb-4">
                    <label
                        class="block text-sm font-medium text-gray-700 mb-1"
                        for="outputDirInput">Output Directory</label
                    >
                    <div class="flex gap-2">
                        <input
                            type="text"
                            id="outputDirInput"
                            bind:value={settings.outputDir}
                            readonly
                            class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-500"
                        />
                        <button
                            on:click={() => dispatch("selectDir")}
                            class="px-3 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md border border-gray-300"
                        >
                            Browse...
                        </button>
                    </div>
                </div>

                <!-- Shape Type -->
                <div class="mb-4">
                    <label
                        class="block text-sm font-medium text-gray-700 mb-1"
                        for="shapeTypeSelect">Shape Type</label
                    >
                    <select
                        id="shapeTypeSelect"
                        bind:value={settings.shapeType}
                        class="w-full px-3 py-2 border border-gray-300 rounded-md"
                    >
                        <option value="polygon">Polygon</option>
                        <option value="bounding_box">Bounding Box</option>
                    </select>
                </div>

                <!-- Data Split Ratios -->
                <div class="mb-4">
                    <label
                        class="block text-sm font-medium text-gray-700 mb-1"
                        for="trainRatioInput">Train Ratio</label
                    >
                    <input
                        type="number"
                        id="trainRatioInput"
                        bind:value={settings.trainRatio}
                        min="0"
                        max="1"
                        step="0.1"
                        class="w-full px-3 py-2 border border-gray-300 rounded-md"
                    />
                </div>
                <div class="mb-4">
                    <label
                        class="block text-sm font-medium text-gray-700 mb-1"
                        for="valRatioInput">Validation Ratio</label
                    >
                    <input
                        type="number"
                        id="valRatioInput"
                        bind:value={settings.valRatio}
                        min="0"
                        max="1"
                        step="0.1"
                        class="w-full px-3 py-2 border border-gray-300 rounded-md"
                    />
                </div>
                <div class="mb-4">
                    <label
                        class="block text-sm font-medium text-gray-700 mb-1"
                        for="testRatioInput">Test Ratio</label
                    >
                    <input
                        type="number"
                        id="testRatioInput"
                        bind:value={settings.testRatio}
                        min="0"
                        max="1"
                        step="0.1"
                        class="w-full px-3 py-2 border border-gray-300 rounded-md"
                    />
                </div>

                <!-- Label Selection Section -->
                <div class="mb-4">
                    <span class="block text-gray-700 mb-2 font-medium"
                        >Labels to Export</span
                    >
                    {#if datasetSummary?.label_counts && Object.keys(datasetSummary.label_counts).length > 0}
                        <p class="text-sm text-gray-500 mb-3">
                            Click on a label to exclude it from the export. By
                            default, all labels are included.
                        </p>
                        <div class="flex flex-wrap gap-2">
                            {#each Object.entries(datasetSummary.label_counts) as [label, count]}
                                <button
                                    type="button"
                                    class={`px-3 py-1 rounded-full text-sm border transition-colors duration-150
                                        ${
                                            !excludedLabels.has(label)
                                                ? "bg-blue-100 text-blue-800 border-blue-300 hover:bg-blue-200"
                                                : "bg-gray-100 text-gray-600 border-gray-300 hover:bg-gray-200 line-through"
                                        }
                                    `}
                                    on:click={() =>
                                        dispatch("toggleExclusion", label)}
                                >
                                    {label} ({count})
                                </button>
                            {/each}
                        </div>
                    {:else if datasetSummary}
                        <p class="text-sm text-gray-500">
                            No labels found in the dataset summary.
                        </p>
                    {:else}
                        <p class="text-sm text-gray-500">Loading labels...</p>
                    {/if}
                </div>
            </div>

            <!-- Actions -->
            <div
                class="px-6 py-4 border-t bg-gray-50 flex justify-end space-x-2 mt-auto"
            >
                <button
                    class="px-4 py-2 border rounded-md hover:bg-gray-100"
                    on:click={() => dispatch("close")}
                    disabled={loading}
                >
                    Cancel
                </button>
                <button
                    class="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-md disabled:opacity-50 flex items-center"
                    on:click={() => dispatch("export")}
                    disabled={loading ||
                        !settings.outputDir ||
                        !datasetSummary?.label_counts}
                >
                    {#if loading}
                        <div
                            class="mr-2 animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"
                        ></div>
                        Exporting...
                    {:else}
                        Export
                    {/if}
                </button>
            </div>
        </div>
    </div>
{/if}

<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let isOpen: boolean;
    export let settings: { outputDir: string };
    export let loading: boolean;
    export let error: string;
    export let success: string;
    export let datasetSummary: any;
    export let selectedLabels: Set<string>;
    export let sourceDir: string;

    const dispatch = createEventDispatcher();
</script>

{#if isOpen}
    <div
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
    >
        <div
            class="max-w-xl w-full bg-white rounded-lg shadow-xl overflow-hidden flex flex-col max-h-[90vh]"
        >
            <!-- Header -->
            <div class="px-6 py-4 border-b">
                <h3 class="text-xl font-bold">Extract Specific Labels</h3>
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
                        for="extractSourceDir">Source Directory</label
                    >
                    <input
                        type="text"
                        id="extractSourceDir"
                        value={sourceDir}
                        readonly
                        class="w-full px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-500"
                    />
                </div>

                <!-- Output Directory -->
                <div class="mb-4">
                    <label
                        class="block text-sm font-medium text-gray-700 mb-1"
                        for="extractOutputDir">Output Directory</label
                    >
                    <div class="flex gap-2">
                        <input
                            type="text"
                            id="extractOutputDir"
                            bind:value={settings.outputDir}
                            readonly
                            placeholder="Select directory for extracted files..."
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

                <!-- Labels Input -->
                <div>
                    <span class="block text-sm font-medium text-gray-700 mb-2"
                        >Labels to Extract (check to include)</span
                    >
                    <div
                        class="max-h-60 overflow-y-auto border border-gray-200 rounded-md p-2 space-y-1"
                    >
                        {#if datasetSummary?.label_counts}
                            {#each Object.entries(datasetSummary.label_counts) as [label, count]}
                                <label
                                    class="flex items-center space-x-2 cursor-pointer text-sm p-1 hover:bg-gray-50 rounded"
                                >
                                    <input
                                        type="checkbox"
                                        checked={selectedLabels.has(label)}
                                        on:change={() =>
                                            dispatch("toggleLabel", label)}
                                        class="rounded border-gray-300 text-indigo-600 shadow-sm focus:border-indigo-300 focus:ring focus:ring-offset-0 focus:ring-indigo-200 focus:ring-opacity-50"
                                    />
                                    <span>{label} ({count})</span>
                                </label>
                            {/each}
                        {:else}
                            <p class="text-sm text-gray-500 italic p-2">
                                No labels found in dataset summary.
                            </p>
                        {/if}
                    </div>
                    <p class="text-xs text-gray-500 mt-1">
                        Only annotations with the selected labels will be kept.
                    </p>
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
                    class="bg-teal-600 hover:bg-teal-700 text-white px-4 py-2 rounded-md disabled:opacity-50 flex items-center"
                    on:click={() => dispatch("extract")}
                    disabled={loading ||
                        !settings.outputDir ||
                        selectedLabels.size === 0}
                >
                    {#if loading}
                        <div
                            class="mr-2 animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"
                        ></div>
                        Extracting...
                    {:else}
                        Run Extraction
                    {/if}
                </button>
            </div>
        </div>
    </div>
{/if}

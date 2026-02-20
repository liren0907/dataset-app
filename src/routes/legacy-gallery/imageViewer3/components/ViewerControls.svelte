<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let directoryPath: string;
    export let loading: boolean;
    export let imagesLength: number;
    export let annotating: boolean;
    export let annotationType: string;
    export let viewMode: string;

    const dispatch = createEventDispatcher();
</script>

<div class="flex flex-wrap items-center gap-4 mb-6">
    <!-- Controls that appear *after* directory selection -->
    {#if directoryPath}
        <!-- Button to select/change directory -->
        <button
            on:click={() => dispatch("selectDirectory")}
            class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            disabled={loading}
        >
            {loading ? "Loading..." : "Change Directory"}
        </button>

        {#if imagesLength > 0}
            <!-- Annotation controls -->
            <div class="flex items-center gap-2">
                <select
                    bind:value={annotationType}
                    class="bg-white border border-gray-300 rounded-md shadow-sm py-2 px-3 text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                >
                    <option value="bounding_box">Bounding Boxes</option>
                    <option value="polygon">Polygons</option>
                </select>

                <button
                    on:click={() => dispatch("annotate")}
                    class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
                    disabled={annotating}
                >
                    {annotating ? "Annotating..." : "Load Annotations"}
                </button>

                <!-- YOLO Export Button -->
                <button
                    on:click={() => dispatch("openExport")}
                    class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-yellow-600 hover:bg-yellow-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-yellow-500"
                >
                    Export to YOLO
                </button>

                <!-- Extract Labels Button -->
                <button
                    on:click={() => dispatch("openExtract")}
                    class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-teal-600 hover:bg-teal-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-teal-500"
                    disabled={!directoryPath || imagesLength === 0}
                >
                    Extract Labels
                </button>
            </div>
        {/if}

        <!-- Directory Path Display -->
        <div class="text-sm text-gray-600 flex-1 truncate">
            <span class="font-medium">Directory:</span>
            {directoryPath}
        </div>

        {#if imagesLength > 0}
            <!-- View Mode Controls -->
            <div class="flex items-center space-x-4 ml-auto">
                <span class="text-sm text-gray-600">View:</span>
                <button
                    class={`px-3 py-1 rounded-md text-sm ${viewMode === "grid" ? "bg-indigo-100 text-indigo-700" : "text-gray-700 hover:bg-gray-100"}`}
                    on:click={() => dispatch("changeViewMode", "grid")}
                >
                    Grid
                </button>
                <button
                    class={`px-3 py-1 rounded-md text-sm ${viewMode === "column" ? "bg-indigo-100 text-indigo-700" : "text-gray-700 hover:bg-gray-100"}`}
                    on:click={() => dispatch("changeViewMode", "column")}
                >
                    Column
                </button>
            </div>
        {/if}
    {/if}
</div>

<script lang="ts">
    let {
        directoryPath,
        loading,
        imagesLength,
        annotating,
        annotationType = $bindable(),
        viewMode,
        onselectdirectory,
        onannotate,
        onchangeviewmode,
        onopenexport,
        onopenextract,
    }: {
        directoryPath: string;
        loading: boolean;
        imagesLength: number;
        annotating: boolean;
        annotationType: string;
        viewMode: string;
        onselectdirectory?: () => void;
        onannotate?: () => void;
        onchangeviewmode?: (mode: string) => void;
        onopenexport?: () => void;
        onopenextract?: () => void;
    } = $props();
</script>

<div class="flex flex-wrap items-center gap-4 mb-6">
    {#if directoryPath}
        <button
            onclick={() => onselectdirectory?.()}
            class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            disabled={loading}
        >
            {loading ? "Loading..." : "Change Directory"}
        </button>

        {#if imagesLength > 0}
            <div class="flex items-center gap-2">
                <select
                    bind:value={annotationType}
                    class="bg-white border border-gray-300 rounded-md shadow-sm py-2 px-3 text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                >
                    <option value="bounding_box">Bounding Boxes</option>
                    <option value="polygon">Polygons</option>
                </select>

                <button
                    onclick={() => onannotate?.()}
                    class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
                    disabled={annotating}
                >
                    {annotating ? "Annotating..." : "Load Annotations"}
                </button>

                <button
                    onclick={() => onopenexport?.()}
                    class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-yellow-600 hover:bg-yellow-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-yellow-500"
                >
                    Export to YOLO
                </button>

                <button
                    onclick={() => onopenextract?.()}
                    class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-teal-600 hover:bg-teal-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-teal-500"
                    disabled={!directoryPath || imagesLength === 0}
                >
                    Extract Labels
                </button>
            </div>
        {/if}

        <div class="text-sm text-gray-600 flex-1 truncate">
            <span class="font-medium">Directory:</span>
            {directoryPath}
        </div>

        {#if imagesLength > 0}
            <div class="flex items-center space-x-4 ml-auto">
                <span class="text-sm text-gray-600">View:</span>
                <button
                    class={`px-3 py-1 rounded-md text-sm ${viewMode === "grid" ? "bg-indigo-100 text-indigo-700" : "text-gray-700 hover:bg-gray-100"}`}
                    onclick={() => onchangeviewmode?.("grid")}
                >
                    Grid
                </button>
                <button
                    class={`px-3 py-1 rounded-md text-sm ${viewMode === "column" ? "bg-indigo-100 text-indigo-700" : "text-gray-700 hover:bg-gray-100"}`}
                    onclick={() => onchangeviewmode?.("column")}
                >
                    Column
                </button>
            </div>
        {/if}
    {/if}
</div>

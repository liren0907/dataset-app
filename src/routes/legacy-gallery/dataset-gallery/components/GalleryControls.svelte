<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { ProcessedImage } from "$lib/services/datasetService";

    export let directoryPath: string;
    export let images: ProcessedImage[];
    export let loading: boolean;
    export let annotating: boolean;
    export let annotationType: string;
    export let cropToolOpen: boolean;
    export let viewMode: string;

    const dispatch = createEventDispatcher();
</script>

<div class="flex flex-wrap items-center gap-3 mb-6">
    <!-- Change Directory Button -->
    <button
        on:click={() => dispatch("selectDirectory")}
        class="btn btn-ghost btn-sm border border-base-content/20"
        disabled={loading}
    >
        <span class="material-symbols-rounded icon-sm">folder_open</span>
        {loading ? "Loading..." : "Change Directory"}
    </button>

    <!-- Directory Path -->
    <div class="badge badge-lg badge-ghost flex-1 truncate max-w-md">
        <span class="material-symbols-rounded icon-sm mr-1">folder</span>
        {directoryPath}
    </div>

    {#if images.length > 0}
        <!-- Annotation Controls -->
        <div class="join">
            <select
                bind:value={annotationType}
                class="select select-sm select-bordered join-item"
            >
                <option value="bounding_box">Bounding Boxes</option>
                <option value="polygon">Polygons</option>
            </select>

            <button
                on:click={() => dispatch("annotate")}
                class="btn btn-success btn-sm join-item"
                disabled={annotating}
            >
                <span class="material-symbols-rounded icon-sm">
                    {annotating ? "hourglass_empty" : "auto_fix_high"}
                </span>
                {annotating ? "Loading..." : "Load Annotations"}
            </button>
        </div>

        <!-- Export Button -->
        <button
            on:click={() => dispatch("export")}
            class="btn btn-info btn-sm"
            disabled={!directoryPath || images.length === 0}
        >
            <span class="material-symbols-rounded icon-sm">upload</span>
            Export Dataset
        </button>

        <!-- Crop & Remap Toggle -->
        <button
            on:click={() => dispatch("toggleCrop")}
            class="btn btn-secondary btn-sm"
        >
            <span class="material-symbols-rounded icon-sm">crop</span>
            {cropToolOpen ? "Hide" : "Show"} Crop & Remap
        </button>

        <!-- View Mode Toggle -->
        <div class="join ml-auto">
            <button
                class="btn btn-sm join-item {viewMode === 'grid'
                    ? 'btn-active'
                    : ''}"
                on:click={() => dispatch("changeViewMode", "grid")}
            >
                <span class="material-symbols-rounded icon-sm">grid_view</span>
            </button>
            <button
                class="btn btn-sm join-item {viewMode === 'column'
                    ? 'btn-active'
                    : ''}"
                on:click={() => dispatch("changeViewMode", "column")}
            >
                <span class="material-symbols-rounded icon-sm">view_list</span>
            </button>
        </div>
    {/if}
</div>

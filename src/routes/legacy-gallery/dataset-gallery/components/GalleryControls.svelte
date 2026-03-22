<script lang="ts">
    import type { ProcessedImage } from "$lib/services/datasetService";

    let {
        directoryPath,
        images,
        loading,
        annotating,
        annotationType = $bindable(),
        cropToolOpen,
        viewMode,
        onselectdirectory,
        onannotate,
        onexport,
        ontogglecrop,
        onchangeviewmode,
    }: {
        directoryPath: string;
        images: ProcessedImage[];
        loading: boolean;
        annotating: boolean;
        annotationType: string;
        cropToolOpen: boolean;
        viewMode: string;
        onselectdirectory?: () => void;
        onannotate?: () => void;
        onexport?: () => void;
        ontogglecrop?: () => void;
        onchangeviewmode?: (mode: string) => void;
    } = $props();
</script>

<div class="flex flex-wrap items-center gap-3 mb-6">
    <!-- Change Directory Button -->
    <button
        onclick={() => onselectdirectory?.()}
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
                onclick={() => onannotate?.()}
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
            onclick={() => onexport?.()}
            class="btn btn-info btn-sm"
            disabled={!directoryPath || images.length === 0}
        >
            <span class="material-symbols-rounded icon-sm">upload</span>
            Export Dataset
        </button>

        <!-- Crop & Remap Toggle -->
        <button
            onclick={() => ontogglecrop?.()}
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
                onclick={() => onchangeviewmode?.("grid")}
            >
                <span class="material-symbols-rounded icon-sm">grid_view</span>
            </button>
            <button
                class="btn btn-sm join-item {viewMode === 'column'
                    ? 'btn-active'
                    : ''}"
                onclick={() => onchangeviewmode?.("column")}
            >
                <span class="material-symbols-rounded icon-sm">view_list</span>
            </button>
        </div>
    {/if}
</div>

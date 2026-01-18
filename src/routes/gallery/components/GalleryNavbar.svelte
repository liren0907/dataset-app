<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { DatasetSummary, ProcessedImage } from "../datasetService";

    export let isMockMode: boolean;
    export let loading: boolean;
    export let directoryPath: string;
    export let images: ProcessedImage[];
    export let annotationType: string;
    export let autoAnnotationEnabled: boolean;
    export let annotating: boolean;
    export let autoAnnotating: boolean;
    export let showCropTool: boolean;
    export let datasetSummary: DatasetSummary | null;
    export let viewMode: string;
    export let editMode: "modal" | "sidebar";

    const dispatch = createEventDispatcher();

    function splitPath(path: string) {
        return path.split("/").slice(-2);
    }
</script>

<div class="flex flex-col gap-4 mb-6">
    <!-- Navbar Toolbar (Modern Minimalist) -->
    <div
        class="navbar bg-base-100 min-h-0 h-14 border border-base-200 shadow-sm rounded-lg px-3 gap-2"
    >
        <!-- Left: Directory & Breadcrumbs -->
        <div class="flex items-center gap-2 flex-1 min-w-0">
            <!-- Mock Mode Toggle -->
            <div
                class="tooltip tooltip-bottom"
                data-tip={isMockMode
                    ? "Switch to Real Data"
                    : "Switch to Mock Data"}
            >
                <button
                    class={`btn btn-sm btn-circle ${isMockMode ? "btn-secondary text-secondary-content" : "btn-ghost text-base-content/40"}`}
                    on:click={() => dispatch("toggleMockMode")}
                    aria-label="Toggle Mock Mode"
                >
                    <span class="material-symbols-rounded text-lg">science</span
                    >
                </button>
            </div>

            <div class="divider divider-horizontal mx-0 h-6"></div>

            <button
                on:click={() => dispatch("selectDirectory")}
                class="btn btn-ghost btn-sm gap-2 text-base-content/70 hover:text-base-content hover:bg-base-200"
                disabled={loading}
                title="Open Project Directory"
            >
                {#if loading}
                    <span class="loading loading-spinner loading-xs"></span>
                {:else}
                    <span class="material-symbols-rounded text-lg"
                        >folder_open</span
                    >
                {/if}
            </button>

            <div class="divider divider-horizontal mx-0 h-6"></div>

            {#if directoryPath}
                <div class="breadcrumbs text-sm ml-1 hidden sm:block">
                    <ul>
                        <li>
                            <span class="text-base-content/50">Project</span>
                        </li>
                        {#each splitPath(directoryPath) as part, i}
                            <li>
                                <span
                                    class={`font-medium ${i === 1 ? "text-base-content" : "text-base-content/70"}`}
                                >
                                    {part}
                                </span>
                            </li>
                        {/each}
                    </ul>
                </div>
                <!-- Mobile fallback for path -->
                <div
                    class="text-sm font-medium text-base-content truncated sm:hidden"
                >
                    {directoryPath.split("/").pop()}
                </div>
            {:else}
                <span class="text-sm text-base-content/50 italic ml-1"
                    >No directory selected</span
                >
            {/if}
        </div>

        <!-- Right: Tools & Actions -->
        <div class="flex items-center gap-1 sm:gap-2 flex-none">
            <!-- Annotation Type Toggle -->
            <div class="join hidden lg:flex">
                <button
                    class={`btn btn-sm join-item gap-2 border-0 px-4 ${annotationType === "bounding_box" ? "bg-base-200 text-base-content shadow-inner" : "btn-ghost text-base-content/70"}`}
                    on:click={() =>
                        dispatch("setAnnotationType", "bounding_box")}
                    disabled={!directoryPath || images.length === 0}
                    title="Bounding Boxes"
                >
                    <span class="material-symbols-rounded text-lg"
                        >crop_square</span
                    >
                    <span class="hidden xl:inline font-normal">Boxes</span>
                </button>
                <button
                    class={`btn btn-sm join-item gap-2 border-0 px-4 ${annotationType === "polygon" ? "bg-base-200 text-base-content shadow-inner" : "btn-ghost text-base-content/70"}`}
                    on:click={() => dispatch("setAnnotationType", "polygon")}
                    disabled={!directoryPath || images.length === 0}
                    title="Polygons"
                >
                    <span class="material-symbols-rounded text-lg">hexagon</span
                    >
                    <span class="hidden xl:inline font-normal">Polygon</span>
                </button>
            </div>

            <div
                class="divider divider-horizontal mx-0 h-6 hidden lg:flex"
            ></div>

            <!-- Auto-load Toggle -->
            <div
                class="tooltip tooltip-bottom"
                data-tip={autoAnnotationEnabled
                    ? "Auto-load active"
                    : "Auto-load inactive"}
            >
                <button
                    class={`btn btn-sm btn-square join-item ${autoAnnotationEnabled ? "btn-active text-primary" : "btn-ghost text-base-content/50"}`}
                    on:click={() => dispatch("toggleAutoAnnotation")}
                    disabled={!directoryPath}
                    aria-label="Toggle Auto-load Annotations"
                >
                    <span class="material-symbols-rounded text-xl"
                        >autorenew</span
                    >
                </button>
            </div>

            <div
                class="divider divider-horizontal mx-0 h-6 hidden lg:flex"
            ></div>

            <!-- Actions -->
            <button
                on:click={() => dispatch("annotateImages")}
                class="btn btn-ghost btn-sm btn-square text-base-content/70 hover:text-primary hover:bg-primary/10"
                disabled={!directoryPath ||
                    images.length === 0 ||
                    annotating ||
                    autoAnnotating}
                title="Load Annotations"
            >
                {#if annotating}
                    <span class="loading loading-spinner loading-xs"></span>
                {:else}
                    <span class="material-symbols-rounded text-xl">label</span>
                {/if}
            </button>

            <button
                on:click={() => dispatch("openExportModal")}
                class="btn btn-ghost btn-sm btn-square text-base-content/70 hover:text-primary hover:bg-primary/10"
                disabled={!directoryPath || images.length === 0}
                title="Export Dataset"
            >
                <span class="material-symbols-rounded text-xl">ios_share</span>
            </button>

            <!-- Crop & Remap Button -->
            <button
                on:click={() => dispatch("toggleCropTool")}
                class="btn btn-ghost btn-sm btn-square text-base-content/70 hover:text-secondary hover:bg-secondary/10"
                class:bg-secondary={showCropTool}
                title="Crop & Remap Tool"
            >
                <span class="material-symbols-rounded text-xl">crop</span>
            </button>

            <!-- Extract Labels Button -->
            <button
                on:click={() => dispatch("openExtractModal")}
                class="btn btn-ghost btn-sm btn-square text-base-content/70 hover:text-accent hover:bg-accent/10"
                disabled={!directoryPath || !datasetSummary}
                title="Extract Labels"
            >
                <span class="material-symbols-rounded text-xl">label</span>
            </button>

            <div class="divider divider-horizontal mx-0 h-6"></div>

            <!-- View Mode Toggle -->
            <div class="join">
                <button
                    class={`btn btn-sm join-item btn-square border-0 ${viewMode === "grid" ? "bg-base-200 text-base-content shadow-inner" : "btn-ghost text-base-content/60"}`}
                    on:click={() => dispatch("setViewMode", "grid")}
                    title="Grid View"
                >
                    <span class="material-symbols-rounded text-lg"
                        >grid_view</span
                    >
                </button>
                <button
                    class={`btn btn-sm join-item btn-square border-0 ${viewMode === "column" ? "bg-base-200 text-base-content shadow-inner" : "btn-ghost text-base-content/60"}`}
                    on:click={() => dispatch("setViewMode", "column")}
                    title="List View"
                >
                    <span class="material-symbols-rounded text-lg"
                        >view_list</span
                    >
                </button>
            </div>
        </div>
    </div>

    <!-- Secondary Toolbar (Edit Mode) -->
    <div class="flex justify-end px-1">
        <!-- Edit Mode Toggle -->
        <div class="join">
            <button
                class={`btn btn-sm join-item gap-2 border-0 px-6 ${editMode === "modal" ? "bg-base-200 text-base-content shadow-inner" : "btn-ghost text-base-content/60"}`}
                on:click={() => dispatch("setEditMode", "modal")}
                title="Pop-out Editor Mode"
            >
                <span class="material-symbols-rounded text-lg">open_in_new</span
                >
                <span class="hidden 2xl:inline">Pop-out</span>
            </button>
            <button
                class={`btn btn-sm join-item gap-2 border-0 px-6 ${editMode === "sidebar" ? "bg-base-200 text-base-content shadow-inner" : "btn-ghost text-base-content/60"}`}
                on:click={() => dispatch("setEditMode", "sidebar")}
                title="Sidebar Editor Mode"
            >
                <span class="material-symbols-rounded text-lg"
                    >view_sidebar</span
                >
                <span class="hidden 2xl:inline">Sidebar</span>
            </button>
        </div>
    </div>
</div>

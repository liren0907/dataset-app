<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type {
        DatasetSummary,
        ProcessedImage,
    } from "../services/datasetService";
    import IconButton from "$lib/components/ui/IconButton.svelte";
    import ToggleButtonGroup from "$lib/components/ui/ToggleButtonGroup.svelte";
    import IconSegmentedControl from "$lib/components/ui/IconSegmentedControl.svelte";

    export let isMockMode: boolean;
    export let loading: boolean;
    export let directoryPath: string;
    export let images: ProcessedImage[];
    export let annotationType: "bounding_box" | "polygon" = "bounding_box";
    export let autoAnnotationEnabled: boolean;
    export let annotating: boolean;

    export let showCropTool: boolean;
    export let showAdvancedCropTool: boolean;
    export let viewMode: "grid" | "column" = "grid";
    export let editMode: "modal" | "sidebar" = "sidebar";

    const dispatch = createEventDispatcher();

    function splitPath(path: string) {
        if (!path) return [];
        return path.split("/").slice(-2);
    }

    // --- Options Configuration ---

    // 1. Annotation Types (Box vs Polygon)
    const annotationOptions = [
        {
            value: "bounding_box",
            icon: "crop_square",
            tooltip: "Bounding Boxes",
        },
        {
            value: "polygon",
            icon: "hexagon",
            tooltip: "Polygons",
        },
    ];

    // 2. View Modes (Grid vs List)
    const viewModeOptions = [
        { value: "grid", icon: "grid_view", tooltip: "Grid View" },
        { value: "column", icon: "view_list", tooltip: "List View" },
    ];

    // 3. Edit Modes (Modal vs Sidebar)
    const editModeOptions = [
        {
            value: "modal",
            icon: "open_in_new",
            label: "Pop-out",
            tooltip: "Pop-out Editor Mode",
        },
        {
            value: "sidebar",
            icon: "view_sidebar",
            label: "Sidebar",
            tooltip: "Sidebar Editor Mode",
        },
    ];
</script>

<div class="flex flex-col gap-2 mb-6">
    <!-- Row 1: Data Source Selection -->
    <div
        class="navbar bg-base-100 min-h-0 h-12 border border-base-200 shadow-sm rounded-lg px-3 gap-2"
    >
        <!-- Mock Mode Toggle -->
        <IconButton
            icon="science"
            variant={isMockMode ? "soft" : "ghost"}
            active={isMockMode}
            tooltip={isMockMode ? "Switch to Real Data" : "Switch to Mock Data"}
            on:click={() => dispatch("toggleMockMode")}
        />

        <div class="divider divider-horizontal mx-0 h-6"></div>

        <!-- Open Directory Button -->
        <IconButton
            icon="folder_open"
            tooltip="Open Project Directory"
            disabled={loading}
            {loading}
            variant="ghost"
            on:click={() => dispatch("selectDirectory")}
        />

        <div class="divider divider-horizontal mx-0 h-6"></div>

        <!-- Breadcrumbs -->
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

    <!-- Row 2: Tools & Actions -->
    <div
        class="navbar bg-base-100 min-h-0 h-12 border border-base-200 shadow-sm rounded-lg px-3 gap-2"
    >
        <!-- Left: Annotation Tools -->
        <div class="flex items-center gap-1 sm:gap-2">
            <!-- Annotation Type Toggle -->
            {#if directoryPath && images.length > 0}
                <IconSegmentedControl
                    options={annotationOptions}
                    value={annotationType}
                    on:change={(e) => dispatch("setAnnotationType", e.detail)}
                />
                <div class="divider divider-horizontal mx-0 h-6"></div>
            {/if}

            <!-- Auto-load Toggle -->
            <IconButton
                icon="autorenew"
                label="Auto"
                active={autoAnnotationEnabled}
                variant={autoAnnotationEnabled ? "soft" : "ghost"}
                tooltip={autoAnnotationEnabled
                    ? "Auto-load Active"
                    : "Auto-load Inactive"}
                disabled={!directoryPath}
                on:click={() => dispatch("toggleAutoAnnotation")}
            />

            <!-- Load Annotations -->
            <IconButton
                icon="label"
                label="Load"
                tooltip="Load Annotations"
                disabled={!directoryPath || images.length === 0 || annotating}
                loading={annotating}
                on:click={() => dispatch("annotateImages")}
            />
        </div>

        <div class="divider divider-horizontal mx-0 h-6"></div>

        <!-- Center: Export & Crop Tools -->
        <div class="flex items-center gap-1 sm:gap-2">
            <!-- Export -->
            <IconButton
                icon="ios_share"
                label="Export"
                tooltip="Export Dataset"
                disabled={!directoryPath || images.length === 0}
                on:click={() => dispatch("openExportModal")}
            />

            <!-- Crop Tool -->
            <IconButton
                icon="crop"
                label="Crop"
                tooltip="Crop & Remap Tool"
                active={showCropTool}
                variant={showCropTool ? "soft" : "ghost"}
                on:click={() => dispatch("toggleCropTool")}
            />

            <!-- Advanced Crop Tool -->
            <IconButton
                icon="auto_awesome"
                label="Preview"
                tooltip="Advanced Crop with Preview"
                active={showAdvancedCropTool}
                variant={showAdvancedCropTool ? "soft" : "ghost"}
                on:click={() => dispatch("toggleAdvancedCropTool")}
            />
        </div>

        <!-- Right: View Controls -->
        <div class="flex-1"></div>
        <div class="flex items-center gap-2">
            <!-- View Mode Toggle -->
            <IconSegmentedControl
                options={viewModeOptions}
                value={viewMode}
                on:change={(e) => dispatch("setViewMode", e.detail)}
            />

            <div class="divider divider-horizontal mx-0 h-6"></div>

            <!-- Edit Mode Toggle -->
            <ToggleButtonGroup
                options={editModeOptions}
                value={editMode}
                on:change={(e) => dispatch("setEditMode", e.detail)}
            />
        </div>
    </div>
</div>

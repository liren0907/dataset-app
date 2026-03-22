<script lang="ts">
    import type {
        BBox,
        Polygon,
        Polyline,
        Keypoint,
    } from "$lib/logic/FabricManager";
    import SectionLabel from "$lib/components/ui/SectionLabel.svelte";
    import ClassManager from "$lib/components/fabric/ClassManager.svelte";
    import {
        labelTaxonomy,
        classColorMap,
    } from "$lib/stores/labelTaxonomyStore";

    let {
        bBoxes = [],
        polygons = [],
        polylines = [],
        keypoints = [],
        selectedObjects = [],
        onUpdateBBoxLabel = () => {},
        onUpdatePolygonLabel = () => {},
        onUpdatePolylineLabel = () => {},
        onUpdateKeypointLabel = () => {},
        onDeleteBBox = () => {},
        onDeletePolygon = () => {},
        onDeletePolyline = () => {},
        onDeleteKeypoint = () => {},
        onToggleLock = () => {},
        onToggleVisibility = () => {},
    }: {
        bBoxes?: BBox[];
        polygons?: Polygon[];
        polylines?: Polyline[];
        keypoints?: Keypoint[];
        selectedObjects?: any[];
        onUpdateBBoxLabel?: (id: number, label: string) => void;
        onUpdatePolygonLabel?: (id: number, label: string) => void;
        onUpdatePolylineLabel?: (id: number, label: string) => void;
        onUpdateKeypointLabel?: (id: number, label: string) => void;
        onDeleteBBox?: (id: number) => void;
        onDeletePolygon?: (id: number) => void;
        onDeletePolyline?: (id: number) => void;
        onDeleteKeypoint?: (id: number) => void;
        onToggleLock?: (id: number, type: string) => void;
        onToggleVisibility?: (id: number, type: string) => void;
    } = $props();

    function getColor(label: string): string {
        return $classColorMap.get(label) || "#6b7280";
    }

    function isSelected(id: number, type: string): boolean {
        return selectedObjects.some(
            (obj) => obj.id === id && obj.type === type,
        );
    }

    let totalAnnotations = $derived(
        bBoxes.length + polygons.length + polylines.length + keypoints.length,
    );
</script>

<ClassManager />

<div class="flex-1 overflow-y-auto space-y-4 p-4">
    {#if totalAnnotations === 0}
        <p class="text-xs text-base-content/40 text-center py-4">
            No annotations yet
        </p>
    {/if}

    <!-- Bounding Boxes Section -->
    {#if bBoxes.length > 0}
        <div class="space-y-2">
            <SectionLabel>Bounding Boxes ({bBoxes.length})</SectionLabel>
            {#each bBoxes as item (item.id)}
                <div
                    class="card bg-base-200 p-3 text-xs space-y-1 border-l-4 group transition-all duration-200"
                    class:opacity-50={item.hidden}
                    class:ring-2={isSelected(item.id, "bbox")}
                    class:ring-primary={isSelected(item.id, "bbox")}
                    style="border-left-color: {getColor(item.label)};"
                >
                    <div
                        class="font-semibold text-base-content flex items-center justify-between gap-1"
                    >
                        <div class="flex items-center gap-1">
                            <span>Box {item.id}</span>
                            <button
                                class="btn btn-ghost btn-xs p-0 min-h-0 h-4 w-4 opacity-40 hover:opacity-100"
                                class:text-primary={item.locked}
                                class:opacity-100={item.locked}
                                onclick={() => onToggleLock(item.id, "bbox")}
                                title={item.locked ? "Unlock" : "Lock"}
                            >
                                <span
                                    class="material-symbols-outlined text-[14px]"
                                >
                                    {item.locked ? "lock" : "lock_open"}
                                </span>
                            </button>
                            <button
                                class="btn btn-ghost btn-xs p-0 min-h-0 h-4 w-4 opacity-40 hover:opacity-100"
                                class:opacity-100={item.hidden}
                                onclick={() =>
                                    onToggleVisibility(item.id, "bbox")}
                                title={item.hidden ? "Show" : "Hide"}
                            >
                                <span
                                    class="material-symbols-outlined text-[14px]"
                                >
                                    {item.hidden
                                        ? "visibility_off"
                                        : "visibility"}
                                </span>
                            </button>
                        </div>

                        <div class="flex items-center gap-1">
                            <select
                                class="select select-xs bg-base-300 text-[10px] w-20 h-5 min-h-0 px-1"
                                value={item.label}
                                onchange={(e) =>
                                    onUpdateBBoxLabel(
                                        item.id,
                                        e.currentTarget.value,
                                    )}
                            >
                                <option value="">—</option>
                                {#each $labelTaxonomy as cls (cls.name)}
                                    <option value={cls.name}
                                        >{cls.shortcut
                                            ? `${cls.shortcut}: `
                                            : ""}{cls.name}</option
                                    >
                                {/each}
                            </select>
                            <button
                                class="btn btn-ghost btn-xs opacity-0 group-hover:opacity-100 text-error p-0 min-h-0 h-5 w-5 transition-opacity"
                                title="Delete"
                                onclick={() => onDeleteBBox(item.id)}
                            >
                                <span class="material-symbols-outlined text-sm"
                                    >delete</span
                                >
                            </button>
                        </div>
                    </div>

                    <div class="text-base-content/60 flex justify-between">
                        <span
                            >({Math.round(item.x1)}, {Math.round(
                                item.y1,
                            )})</span
                        >
                        <span
                            >({Math.round(item.x2)}, {Math.round(
                                item.y2,
                            )})</span
                        >
                    </div>
                </div>
            {/each}
        </div>
    {/if}

    <!-- Polygons Section -->
    {#if polygons.length > 0}
        <div class="space-y-2">
            <SectionLabel>Polygons ({polygons.length})</SectionLabel>
            {#each polygons as item (item.id)}
                <div
                    class="card bg-base-200 p-3 text-xs space-y-1 border-l-4 group transition-all duration-200"
                    class:opacity-50={item.hidden}
                    class:ring-2={isSelected(item.id, "polygon")}
                    class:ring-primary={isSelected(item.id, "polygon")}
                    style="border-left-color: {getColor(item.label)};"
                >
                    <div
                        class="font-semibold text-base-content flex items-center justify-between gap-1"
                    >
                        <div class="flex items-center gap-1">
                            <span>Polygon {item.id}</span>
                            <button
                                class="btn btn-ghost btn-xs p-0 min-h-0 h-4 w-4 opacity-40 hover:opacity-100"
                                class:text-primary={item.locked}
                                class:opacity-100={item.locked}
                                onclick={() =>
                                    onToggleLock(item.id, "polygon")}
                                title={item.locked ? "Unlock" : "Lock"}
                            >
                                <span
                                    class="material-symbols-outlined text-[14px]"
                                >
                                    {item.locked ? "lock" : "lock_open"}
                                </span>
                            </button>
                            <button
                                class="btn btn-ghost btn-xs p-0 min-h-0 h-4 w-4 opacity-40 hover:opacity-100"
                                class:opacity-100={item.hidden}
                                onclick={() =>
                                    onToggleVisibility(item.id, "polygon")}
                                title={item.hidden ? "Show" : "Hide"}
                            >
                                <span
                                    class="material-symbols-outlined text-[14px]"
                                >
                                    {item.hidden
                                        ? "visibility_off"
                                        : "visibility"}
                                </span>
                            </button>
                        </div>

                        <div class="flex items-center gap-1">
                            <select
                                class="select select-xs bg-base-300 text-[10px] w-20 h-5 min-h-0 px-1"
                                value={item.label}
                                onchange={(e) =>
                                    onUpdatePolygonLabel(
                                        item.id,
                                        e.currentTarget.value,
                                    )}
                            >
                                <option value="">—</option>
                                {#each $labelTaxonomy as cls (cls.name)}
                                    <option value={cls.name}
                                        >{cls.shortcut
                                            ? `${cls.shortcut}: `
                                            : ""}{cls.name}</option
                                    >
                                {/each}
                            </select>
                            <button
                                class="btn btn-ghost btn-xs opacity-0 group-hover:opacity-100 text-error p-0 min-h-0 h-5 w-5 transition-opacity"
                                title="Delete"
                                onclick={() => onDeletePolygon(item.id)}
                            >
                                <span class="material-symbols-outlined text-sm"
                                    >delete</span
                                >
                            </button>
                        </div>
                    </div>
                    <div class="text-[9px] text-base-content/40 truncate">
                        {item.points.length} points
                    </div>
                </div>
            {/each}
        </div>
    {/if}

    <!-- Polylines Section -->
    {#if polylines.length > 0}
        <div class="space-y-2">
            <SectionLabel>Polylines ({polylines.length})</SectionLabel>
            {#each polylines as item (item.id)}
                <div
                    class="card bg-base-200 p-3 text-xs space-y-1 border-l-4 group transition-all duration-200"
                    class:opacity-50={item.hidden}
                    class:ring-2={isSelected(item.id, "polyline")}
                    class:ring-primary={isSelected(item.id, "polyline")}
                    style="border-left-color: {getColor(item.label)};"
                >
                    <div
                        class="font-semibold text-base-content flex items-center justify-between gap-1"
                    >
                        <div class="flex items-center gap-1">
                            <span>Polyline {item.id}</span>
                            <button
                                class="btn btn-ghost btn-xs p-0 min-h-0 h-4 w-4 opacity-40 hover:opacity-100"
                                class:text-primary={item.locked}
                                class:opacity-100={item.locked}
                                onclick={() =>
                                    onToggleLock(item.id, "polyline")}
                                title={item.locked ? "Unlock" : "Lock"}
                            >
                                <span
                                    class="material-symbols-outlined text-[14px]"
                                >
                                    {item.locked ? "lock" : "lock_open"}
                                </span>
                            </button>
                            <button
                                class="btn btn-ghost btn-xs p-0 min-h-0 h-4 w-4 opacity-40 hover:opacity-100"
                                class:opacity-100={item.hidden}
                                onclick={() =>
                                    onToggleVisibility(item.id, "polyline")}
                                title={item.hidden ? "Show" : "Hide"}
                            >
                                <span
                                    class="material-symbols-outlined text-[14px]"
                                >
                                    {item.hidden
                                        ? "visibility_off"
                                        : "visibility"}
                                </span>
                            </button>
                        </div>

                        <div class="flex items-center gap-1">
                            <select
                                class="select select-xs bg-base-300 text-[10px] w-20 h-5 min-h-0 px-1"
                                value={item.label}
                                onchange={(e) =>
                                    onUpdatePolylineLabel(
                                        item.id,
                                        e.currentTarget.value,
                                    )}
                            >
                                <option value="">—</option>
                                {#each $labelTaxonomy as cls (cls.name)}
                                    <option value={cls.name}
                                        >{cls.shortcut
                                            ? `${cls.shortcut}: `
                                            : ""}{cls.name}</option
                                    >
                                {/each}
                            </select>
                            <button
                                class="btn btn-ghost btn-xs opacity-0 group-hover:opacity-100 text-error p-0 min-h-0 h-5 w-5 transition-opacity"
                                title="Delete"
                                onclick={() => onDeletePolyline(item.id)}
                            >
                                <span class="material-symbols-outlined text-sm"
                                    >delete</span
                                >
                            </button>
                        </div>
                    </div>
                    <div class="text-[9px] text-base-content/40 truncate">
                        {item.points.length} points
                    </div>
                </div>
            {/each}
        </div>
    {/if}

    <!-- Keypoints Section -->
    {#if keypoints.length > 0}
        <div class="space-y-2">
            <SectionLabel>Keypoints ({keypoints.length})</SectionLabel>
            {#each keypoints as item (item.id)}
                <div
                    class="card bg-base-200 p-3 text-xs space-y-1 border-l-4 group transition-all duration-200"
                    class:opacity-50={item.hidden}
                    class:ring-2={isSelected(item.id, "keypoint")}
                    class:ring-primary={isSelected(item.id, "keypoint")}
                    style="border-left-color: {getColor(item.label)};"
                >
                    <div
                        class="font-semibold text-base-content flex items-center justify-between gap-1"
                    >
                        <div class="flex items-center gap-1">
                            <span>Point {item.id}</span>
                            <button
                                class="btn btn-ghost btn-xs p-0 min-h-0 h-4 w-4 opacity-40 hover:opacity-100"
                                class:text-primary={item.locked}
                                class:opacity-100={item.locked}
                                onclick={() =>
                                    onToggleLock(item.id, "keypoint")}
                                title={item.locked ? "Unlock" : "Lock"}
                            >
                                <span
                                    class="material-symbols-outlined text-[14px]"
                                >
                                    {item.locked ? "lock" : "lock_open"}
                                </span>
                            </button>
                            <button
                                class="btn btn-ghost btn-xs p-0 min-h-0 h-4 w-4 opacity-40 hover:opacity-100"
                                class:opacity-100={item.hidden}
                                onclick={() =>
                                    onToggleVisibility(item.id, "keypoint")}
                                title={item.hidden ? "Show" : "Hide"}
                            >
                                <span
                                    class="material-symbols-outlined text-[14px]"
                                >
                                    {item.hidden
                                        ? "visibility_off"
                                        : "visibility"}
                                </span>
                            </button>
                        </div>

                        <div class="flex items-center gap-1">
                            <select
                                class="select select-xs bg-base-300 text-[10px] w-20 h-5 min-h-0 px-1"
                                value={item.label}
                                onchange={(e) =>
                                    onUpdateKeypointLabel(
                                        item.id,
                                        e.currentTarget.value,
                                    )}
                            >
                                <option value="">—</option>
                                {#each $labelTaxonomy as cls (cls.name)}
                                    <option value={cls.name}
                                        >{cls.shortcut
                                            ? `${cls.shortcut}: `
                                            : ""}{cls.name}</option
                                    >
                                {/each}
                            </select>
                            <button
                                class="btn btn-ghost btn-xs opacity-0 group-hover:opacity-100 text-error p-0 min-h-0 h-5 w-5 transition-opacity"
                                title="Delete"
                                onclick={() => onDeleteKeypoint(item.id)}
                            >
                                <span class="material-symbols-outlined text-sm"
                                    >delete</span
                                >
                            </button>
                        </div>
                    </div>
                    <div class="text-base-content/60">
                        ({Math.round(item.x)}, {Math.round(item.y)})
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>

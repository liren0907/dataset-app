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

    export let bBoxes: BBox[] = [];
    export let polygons: Polygon[] = [];
    export let polylines: Polyline[] = [];
    export let keypoints: Keypoint[] = [];
    export let selectedObjects: any[] = [];

    export let onUpdateBBoxLabel: (
        id: number,
        label: string,
    ) => void = () => {};
    export let onUpdatePolygonLabel: (
        id: number,
        label: string,
    ) => void = () => {};
    export let onUpdatePolylineLabel: (
        id: number,
        label: string,
    ) => void = () => {};
    export let onUpdateKeypointLabel: (
        id: number,
        label: string,
    ) => void = () => {};

    export let onDeleteBBox: (id: number) => void = () => {};
    export let onDeletePolygon: (id: number) => void = () => {};
    export let onDeletePolyline: (id: number) => void = () => {};
    export let onDeleteKeypoint: (id: number) => void = () => {};

    export let onToggleLock: (id: number, type: string) => void = () => {};
    export let onToggleVisibility: (
        id: number,
        type: string,
    ) => void = () => {};

    function getColor(label: string): string {
        return $classColorMap.get(label) || "#6b7280";
    }

    function isSelected(id: number, type: string): boolean {
        return selectedObjects.some(
            (obj) => obj.id === id && obj.type === type,
        );
    }

    $: totalAnnotations =
        bBoxes.length + polygons.length + polylines.length + keypoints.length;
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
                                on:click={() => onToggleLock(item.id, "bbox")}
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
                                on:click={() =>
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
                                on:change={(e) =>
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
                                on:click={() => onDeleteBBox(item.id)}
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
                                on:click={() =>
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
                                on:click={() =>
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
                                on:change={(e) =>
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
                                on:click={() => onDeletePolygon(item.id)}
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
                                on:click={() =>
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
                                on:click={() =>
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
                                on:change={(e) =>
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
                                on:click={() => onDeletePolyline(item.id)}
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
                                on:click={() =>
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
                                on:click={() =>
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
                                on:change={(e) =>
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
                                on:click={() => onDeleteKeypoint(item.id)}
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

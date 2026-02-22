<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import {
        labelTaxonomy,
        classColorMap,
    } from "$lib/stores/labelTaxonomyStore";

    export let selectedObject: any = null; // Single selection
    export let selectedObjects: any[] = []; // Multiple selection
    export let onUpdateProperty: (
        id: number,
        type: string,
        prop: string,
        value: any,
    ) => void = () => {};
    export let onBatchUpdateLabel: (label: string) => void = () => {};

    function handleChange(prop: string, value: any) {
        if (!selectedObject) return;
        onUpdateProperty(selectedObject.id, selectedObject.type, prop, value);
    }

    $: type = selectedObject?.type || "";
    $: isMultiSelect = selectedObjects.length > 1;
</script>

<div class="bg-base-200 p-3 border-t border-base-300">
    <div class="flex items-center justify-between mb-2">
        <h3
            class="text-xs font-bold uppercase tracking-wider text-base-content/50"
        >
            {isMultiSelect ? "Batch Actions" : "Properties"}
        </h3>
        {#if !isMultiSelect && type}
            <span class="badge badge-outline badge-xs opacity-70">{type}</span>
        {:else if isMultiSelect}
            <span class="badge badge-primary badge-xs"
                >{selectedObjects.length} selected</span
            >
        {/if}
    </div>

    {#if isMultiSelect}
        <!-- Batch UI -->
        <div class="space-y-3 py-2">
            <div class="form-control">
                <label class="label py-0 mb-1" for="batch-label">
                    <span
                        class="label-text text-[10px] text-base-content/60 font-bold uppercase"
                        >Re-label All</span
                    >
                </label>
                <div class="flex gap-2">
                    <select
                        id="batch-label"
                        class="select select-bordered select-xs flex-1 text-[10px]"
                        on:change={(e) =>
                            onBatchUpdateLabel(e.currentTarget.value)}
                    >
                        <option value="">Choose Label...</option>
                        {#each $labelTaxonomy as cls (cls.name)}
                            <option value={cls.name}>{cls.name}</option>
                        {/each}
                    </select>
                </div>
            </div>
            <p class="text-[9px] text-base-content/40 italic">
                Changes will apply to all selected annotations on the canvas.
            </p>
        </div>
    {:else if selectedObject}
        <!-- Single Object UI -->
        <div class="grid grid-cols-2 gap-2 text-[10px]">
            <div class="col-span-2 flex items-center gap-2 mb-1">
                <span class="text-base-content/60">ID: {selectedObject.id}</span
                >
                <span class="text-base-content/60">|</span>
                <span class="font-medium"
                    >{selectedObject.label || "No Label"}</span
                >
            </div>

            {#if type === "bbox"}
                <div>
                    <label
                        class="block text-base-content/40 mb-0.5"
                        for="bbox-x1">X1</label
                    >
                    <input
                        id="bbox-x1"
                        type="number"
                        class="input input-bordered input-xs w-full"
                        value={Math.round(selectedObject.x1)}
                        on:change={(e) =>
                            handleChange(
                                "x1",
                                parseFloat(e.currentTarget.value),
                            )}
                    />
                </div>
                <div>
                    <label
                        class="block text-base-content/40 mb-0.5"
                        for="bbox-y1">Y1</label
                    >
                    <input
                        id="bbox-y1"
                        type="number"
                        class="input input-bordered input-xs w-full"
                        value={Math.round(selectedObject.y1)}
                        on:change={(e) =>
                            handleChange(
                                "y1",
                                parseFloat(e.currentTarget.value),
                            )}
                    />
                </div>
                <div>
                    <label
                        class="block text-base-content/40 mb-0.5"
                        for="bbox-x2">X2</label
                    >
                    <input
                        id="bbox-x2"
                        type="number"
                        class="input input-bordered input-xs w-full"
                        value={Math.round(selectedObject.x2)}
                        on:change={(e) =>
                            handleChange(
                                "x2",
                                parseFloat(e.currentTarget.value),
                            )}
                    />
                </div>
                <div>
                    <label
                        class="block text-base-content/40 mb-0.5"
                        for="bbox-y2">Y2</label
                    >
                    <input
                        id="bbox-y2"
                        type="number"
                        class="input input-bordered input-xs w-full"
                        value={Math.round(selectedObject.y2)}
                        on:change={(e) =>
                            handleChange(
                                "y2",
                                parseFloat(e.currentTarget.value),
                            )}
                    />
                </div>
            {:else if type === "keypoint"}
                <div>
                    <label class="block text-base-content/40 mb-0.5" for="kp-x"
                        >X</label
                    >
                    <input
                        id="kp-x"
                        type="number"
                        class="input input-bordered input-xs w-full"
                        value={Math.round(selectedObject.x)}
                        on:change={(e) =>
                            handleChange(
                                "x",
                                parseFloat(e.currentTarget.value),
                            )}
                    />
                </div>
                <div>
                    <label class="block text-base-content/40 mb-0.5" for="kp-y"
                        >Y</label
                    >
                    <input
                        id="kp-y"
                        type="number"
                        class="input input-bordered input-xs w-full"
                        value={Math.round(selectedObject.y)}
                        on:change={(e) =>
                            handleChange(
                                "y",
                                parseFloat(e.currentTarget.value),
                            )}
                    />
                </div>
            {:else}
                <div class="col-span-2 text-base-content/40 italic">
                    Coordinates editing for {type} coming soon.
                </div>
            {/if}
        </div>
    {:else}
        <div class="text-[10px] text-base-content/30 italic text-center py-4">
            Select an object to edit properties
        </div>
    {/if}
</div>

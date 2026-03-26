<script lang="ts">
    import {
        labelClasses,
    } from "$lib/stores/labelTaxonomyStore.svelte";
    import { TextInput, Badge } from "$lib/components/ui";

    let {
        selectedObject = null,
        selectedObjects = [],
        onUpdateProperty = () => {},
        onBatchUpdateLabel = () => {},
    }: {
        selectedObject?: any;
        selectedObjects?: any[];
        onUpdateProperty?: (id: number, type: string, prop: string, value: any) => void;
        onBatchUpdateLabel?: (label: string) => void;
    } = $props();

    function handleChange(prop: string, value: any) {
        if (!selectedObject) return;
        onUpdateProperty(selectedObject.id, selectedObject.type, prop, value);
    }

    function getInputValue(e: Event): number {
        return parseFloat((e.target as any).value);
    }

    let type = $derived(selectedObject?.type || "");
    let isMultiSelect = $derived(selectedObjects.length > 1);
</script>

<div class="bg-base-200 p-3 border-t border-base-300">
    <div class="flex items-center justify-between mb-2">
        <h3
            class="text-xs font-bold uppercase tracking-wider text-base-content/50"
        >
            {isMultiSelect ? "Batch Actions" : "Properties"}
        </h3>
        {#if !isMultiSelect && type}
            <Badge variant="outline" size="xs" class="opacity-70">{type}</Badge>
        {:else if isMultiSelect}
            <Badge variant="primary" size="xs">{selectedObjects.length} selected</Badge>
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
                        onchange={(e) =>
                            onBatchUpdateLabel(e.currentTarget.value)}
                    >
                        <option value="">Choose Label...</option>
                        {#each labelClasses as cls (cls.name)}
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
                        >X1</label
                    >
                    <TextInput
                        type="number"
                        size="xs"
                        class="w-full"
                        value={Math.round(selectedObject.x1)}
                        onchange={(e) =>
                            handleChange(
                                "x1",
                                getInputValue(e),
                            )}
                    />
                </div>
                <div>
                    <label
                        class="block text-base-content/40 mb-0.5"
                        >Y1</label
                    >
                    <TextInput
                        type="number"
                        size="xs"
                        class="w-full"
                        value={Math.round(selectedObject.y1)}
                        onchange={(e) =>
                            handleChange(
                                "y1",
                                getInputValue(e),
                            )}
                    />
                </div>
                <div>
                    <label
                        class="block text-base-content/40 mb-0.5"
                        >X2</label
                    >
                    <TextInput
                        type="number"
                        size="xs"
                        class="w-full"
                        value={Math.round(selectedObject.x2)}
                        onchange={(e) =>
                            handleChange(
                                "x2",
                                getInputValue(e),
                            )}
                    />
                </div>
                <div>
                    <label
                        class="block text-base-content/40 mb-0.5"
                        >Y2</label
                    >
                    <TextInput
                        type="number"
                        size="xs"
                        class="w-full"
                        value={Math.round(selectedObject.y2)}
                        onchange={(e) =>
                            handleChange(
                                "y2",
                                getInputValue(e),
                            )}
                    />
                </div>
            {:else if type === "keypoint"}
                <div>
                    <label class="block text-base-content/40 mb-0.5"
                        >X</label
                    >
                    <TextInput
                        type="number"
                        size="xs"
                        class="w-full"
                        value={Math.round(selectedObject.x)}
                        onchange={(e) =>
                            handleChange(
                                "x",
                                getInputValue(e),
                            )}
                    />
                </div>
                <div>
                    <label class="block text-base-content/40 mb-0.5"
                        >Y</label
                    >
                    <TextInput
                        type="number"
                        size="xs"
                        class="w-full"
                        value={Math.round(selectedObject.y)}
                        onchange={(e) =>
                            handleChange(
                                "y",
                                getInputValue(e),
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

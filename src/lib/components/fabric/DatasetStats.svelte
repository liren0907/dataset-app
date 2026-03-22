<script lang="ts">
    import {
        labelTaxonomy,
        classColorMap,
    } from "$lib/stores/labelTaxonomyStore";
    import type {
        BBox,
        Polygon,
        Polyline,
        Keypoint,
        ImageEntry,
    } from "$lib/logic/FabricManager";

    let {
        images = [],
        allBBoxes = [],
        allPolygons = [],
        allPolylines = [],
        allKeypoints = [],
    }: {
        images?: ImageEntry[];
        allBBoxes?: BBox[];
        allPolygons?: Polygon[];
        allPolylines?: Polyline[];
        allKeypoints?: Keypoint[];
    } = $props();

    // Aggregate counts by class
    let stats = $derived.by(() => {
        const counts = new Map<string, number>();

        const process = (items: any[]) => {
            items.forEach((item) => {
                if (item.label) {
                    counts.set(item.label, (counts.get(item.label) || 0) + 1);
                }
            });
        };

        process(allBBoxes);
        process(allPolygons);
        process(allPolylines);
        process(allKeypoints);

        return Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
    });

    let totalAnns = $derived(
        allBBoxes.length +
        allPolygons.length +
        allPolylines.length +
        allKeypoints.length,
    );
    let annotatedImages = $derived(images.filter((img) => img.hasJson).length);
</script>

<div class="card bg-base-100 shadow-xl border border-base-200">
    <div class="card-body p-6">
        <h2 class="card-title text-sm font-bold flex items-center gap-2 mb-4">
            <span class="material-symbols-rounded text-primary">analytics</span>
            Dataset Statistics
        </h2>

        <div class="grid grid-cols-2 gap-4 mb-6">
            <div class="bg-base-200 p-3 rounded-lg text-center">
                <div
                    class="text-[10px] uppercase text-base-content/50 font-bold mb-1"
                >
                    Images
                </div>
                <div class="text-xl font-black">
                    {annotatedImages} / {images.length}
                </div>
                <div class="text-[9px] text-base-content/40">
                    progress: {Math.round(
                        (annotatedImages / (images.length || 1)) * 100,
                    )}%
                </div>
            </div>
            <div class="bg-base-200 p-3 rounded-lg text-center">
                <div
                    class="text-[10px] uppercase text-base-content/50 font-bold mb-1"
                >
                    Annotations
                </div>
                <div class="text-xl font-black">{totalAnns}</div>
                <div class="text-[9px] text-base-content/40">
                    avg per img: {(totalAnns / (annotatedImages || 1)).toFixed(
                        1,
                    )}
                </div>
            </div>
        </div>

        <div class="space-y-3">
            <h3 class="text-xs font-bold text-base-content/70">
                Class Distribution
            </h3>
            {#if stats.length === 0}
                <p class="text-[10px] text-base-content/30 italic py-2">
                    No labeled objects detected yet
                </p>
            {:else}
                <div class="space-y-2 max-h-48 overflow-y-auto pr-2">
                    {#each stats as [label, count]}
                        <div class="space-y-1">
                            <div class="flex justify-between text-[10px] mb-1">
                                <span class="font-medium">{label}</span>
                                <span class="font-bold">{count}</span>
                            </div>
                            <div
                                class="w-full bg-base-300 rounded-full h-1.5 overflow-hidden"
                            >
                                <div
                                    class="h-full rounded-full transition-all duration-500 ease-out"
                                    style="width: {Math.min(
                                        100,
                                        (count / totalAnns) * 100,
                                    )}%; background-color: {$classColorMap.get(
                                        label,
                                    ) || '#6b7280'};"
                                ></div>
                            </div>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>

        <div class="divider my-2"></div>

        <div
            class="grid grid-cols-4 gap-1 text-[9px] text-center text-base-content/50"
        >
            <div>Boxes: {allBBoxes.length}</div>
            <div>Poly: {allPolygons.length}</div>
            <div>Lines: {allPolylines.length}</div>
            <div>Points: {allKeypoints.length}</div>
        </div>
    </div>
</div>

<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { DatasetSummary } from "$lib/services/gallery/datasetService";
    
    export let datasetSummary: DatasetSummary | null = null;

    const dispatch = createEventDispatcher<{
        initiateCrop: { label: string };
    }>();

    function formatPercentage(value: number, total: number): string {
        if (!total || total === 0) return "0.0%";
        return ((value / total) * 100).toFixed(1) + "%";
    }

    function formatAverage(value: number, count: number): string {
        if (!count || count === 0) return "0.0";
        return (value / count).toFixed(1);
    }

    function handleLabelClick(label: string) {
        dispatch("initiateCrop", { label });
    }

    // Provide default values for empty state
    $: summary = datasetSummary || {
        total_images: 0,
        images_with_annotations: 0,
        total_annotations: 0,
        unique_labels: 0,
        label_counts: {},
        annotation_types: [],
    };

    $: sortedLabelCounts = Object.entries(summary.label_counts || {}).sort(
        ([, countA], [, countB]) => (countB as number) - (countA as number),
    );
</script>

<!-- Dense Info Strip Layout -->
<div
    class="flex flex-col md:flex-row items-stretch border border-base-200 rounded-lg bg-base-100 shadow-sm overflow-hidden min-h-[72px]"
>
    <!-- Section 1: File Statistics -->
    <div
        class="flex flex-col justify-center px-5 py-2 border-b md:border-b-0 md:border-r border-base-200 bg-base-50/50 min-w-[150px]"
    >
        <span
            class="text-[10px] font-bold text-base-content/40 uppercase tracking-wider"
            >File Stats</span
        >
        <div class="flex items-baseline gap-2 mt-0.5">
            <span class="text-2xl font-bold text-base-content"
                >{summary.total_images}</span
            >
            <span class="text-xs font-medium text-base-content/60">Images</span>
        </div>
        <div class="text-xs text-base-content/50 truncate">
            {formatPercentage(
                summary.images_with_annotations,
                summary.total_images,
            )} Annotated
        </div>
    </div>

    <!-- Section 2: Annotation Statistics -->
    <div
        class="flex flex-col justify-center px-5 py-2 border-b md:border-b-0 md:border-r border-base-200 bg-base-50/50 min-w-[180px]"
    >
        <span
            class="text-[10px] font-bold text-base-content/40 uppercase tracking-wider"
            >Annotations</span
        >
        <div class="flex items-baseline gap-2 mt-0.5">
            <span class="text-2xl font-bold text-base-content"
                >{summary.total_annotations}</span
            >
            <span class="text-xs font-medium text-base-content/60">Total</span>
        </div>
        <div class="text-xs text-base-content/50 truncate">
            Avg {formatAverage(
                summary.total_annotations,
                summary.images_with_annotations,
            )} / image
        </div>
    </div>

    <!-- Section 3: Label Distribution -->
    <div
        class="flex-1 flex flex-col justify-center px-5 py-2 overflow-hidden relative group"
    >
        <div class="flex items-center justify-between mb-1">
            <span
                class="text-[10px] font-bold text-base-content/40 uppercase tracking-wider"
                >Label Distribution ({summary.unique_labels})</span
            >
        </div>

        {#if summary.label_counts && Object.keys(summary.label_counts).length > 0}
            <div
                class="flex items-center gap-2 overflow-x-auto no-scrollbar mask-linear-fade pb-1"
            >
                {#each sortedLabelCounts as [label, count]}
                    <button
                        on:click={() => handleLabelClick(label)}
                        class="badge badge-neutral gap-1.5 py-3 px-3 border-base-300 bg-base-200 text-base-content whitespace-nowrap flex-shrink-0 hover:bg-primary hover:text-primary-content hover:border-primary transition-all duration-200 cursor-pointer group"
                        title="Click to crop around '{label}'"
                    >
                        <span class="font-medium text-xs">{label}</span>
                        <span
                            class="badge badge-sm badge-ghost p-0 h-4 min-w-[16px] text-[10px] bg-base-content/10 text-base-content/70 group-hover:bg-primary-content/20 group-hover:text-primary-content"
                            >{count}</span
                        >
                        <span class="material-symbols-rounded text-xs opacity-0 group-hover:opacity-100 transition-opacity">crop</span>
                    </button>
                {/each}
            </div>
            <!-- Scroll Details Hint -->
            <div
                class="absolute right-0 top-0 bottom-0 w-8 bg-gradient-to-l from-base-100 to-transparent pointer-events-none md:block hidden"
            ></div>
        {:else}
            <div
                class="text-sm text-base-content/40 italic flex items-center gap-2"
            >
                <span class="material-symbols-rounded text-lg">info</span>
                No labels found in dataset
            </div>
        {/if}
    </div>
</div>

<style>
    /* Hide scrollbar for Chrome, Safari and Opera */
    .no-scrollbar::-webkit-scrollbar {
        display: none;
    }
    /* Hide scrollbar for IE, Edge and Firefox */
    .no-scrollbar {
        -ms-overflow-style: none; /* IE and Edge */
        scrollbar-width: none; /* Firefox */
    }
</style>

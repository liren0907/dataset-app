<script lang="ts">
    export let datasetSummary: any = null;

    function formatPercentage(value: number, total: number): string {
        if (!total || total === 0) return "0.0%";
        return ((value / total) * 100).toFixed(1) + "%";
    }

    function formatAverage(value: number, count: number): string {
        if (!count || count === 0) return "0.0";
        return (value / count).toFixed(1);
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
</script>

<div class="card bg-base-100 shadow-sm border border-base-300">
    <div class="card-body p-4">
        <h2 class="card-title text-lg text-base-content">Dataset Summary</h2>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-2">
            <!-- File Statistics -->
            <div class="bg-info/10 p-3 rounded-lg border border-info/20">
                <h3 class="text-sm font-medium text-info">File Statistics</h3>
                <p class="text-sm mt-1 text-base-content/80">
                    Total Images: <span class="font-medium"
                        >{summary.total_images}</span
                    >
                </p>
                <p class="text-sm mt-1 text-base-content/80">
                    Images with Annotations: <span class="font-medium"
                        >{summary.images_with_annotations}</span
                    >
                </p>
                <p class="text-sm mt-1 text-base-content/80">
                    Annotation Coverage: <span class="font-medium"
                        >{formatPercentage(
                            summary.images_with_annotations,
                            summary.total_images,
                        )}</span
                    >
                </p>
            </div>

            <!-- Annotation Statistics -->
            <div class="bg-success/10 p-3 rounded-lg border border-success/20">
                <h3 class="text-sm font-medium text-success">
                    Annotation Statistics
                </h3>
                <p class="text-sm mt-1 text-base-content/80">
                    Total Annotations: <span class="font-medium"
                        >{summary.total_annotations}</span
                    >
                </p>
                <p class="text-sm mt-1 text-base-content/80">
                    Average per Annotated Image: <span class="font-medium"
                        >{formatAverage(
                            summary.total_annotations,
                            summary.images_with_annotations,
                        )}</span
                    >
                </p>
                {#if summary.annotation_types && summary.annotation_types.length > 0}
                    <p class="text-sm mt-1 text-base-content/80">
                        Annotation Types: <span class="font-medium"
                            >{summary.annotation_types.join(", ")}</span
                        >
                    </p>
                {:else}
                    <p class="text-sm mt-1 text-base-content/50 italic">
                        No annotation types yet
                    </p>
                {/if}
            </div>

            <!-- Label Statistics -->
            <div
                class="bg-secondary/10 p-3 rounded-lg border border-secondary/20"
            >
                <h3 class="text-sm font-medium text-secondary">
                    Label Statistics
                </h3>
                <p class="text-sm mt-1 text-base-content/80">
                    Unique Labels: <span class="font-medium"
                        >{summary.unique_labels}</span
                    >
                </p>
                {#if summary.label_counts && Object.keys(summary.label_counts).length > 0}
                    <p class="text-sm mt-1 text-base-content/80">
                        Label Counts:
                    </p>
                    <ul
                        class="text-xs mt-1 space-y-1 max-h-32 overflow-y-auto pr-2"
                    >
                        {#each Object.entries(summary.label_counts) as [label, count] (label)}
                            <li
                                class="flex justify-between text-base-content/70"
                            >
                                <span class="font-medium truncate pr-2"
                                    >{label}</span
                                >
                                <span class="badge badge-sm badge-ghost"
                                    >{count}</span
                                >
                            </li>
                        {/each}
                    </ul>
                {:else}
                    <p class="text-xs text-base-content/50 mt-1 italic">
                        No labels found.
                    </p>
                {/if}
            </div>
        </div>
    </div>
</div>

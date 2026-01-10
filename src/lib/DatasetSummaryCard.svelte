<script lang="ts">
    export let datasetSummary: any;

    function formatPercentage(value, total) {
        if (!total || total === 0) return "0.0%";
        return ((value / total) * 100).toFixed(1) + "%";
    }

    function formatAverage(value, count) {
        if (!count || count === 0) return "0.0";
        return (value / count).toFixed(1);
    }
</script>

{#if datasetSummary}
    <div class="card bg-base-100 shadow-xl mb-6">
        <div class="card-body">
            <h2 class="card-title">
                <span class="material-symbols-rounded text-primary"
                    >analytics</span
                >
                Dataset Summary
            </h2>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
                <!-- File Statistics -->
                <div class="stats stats-vertical bg-info/10 shadow">
                    <div class="stat">
                        <div class="stat-figure text-info">
                            <span class="material-symbols-rounded text-2xl"
                                >image</span
                            >
                        </div>
                        <div class="stat-title">Total Images</div>
                        <div class="stat-value text-info text-2xl">
                            {datasetSummary.total_images}
                        </div>
                    </div>
                    <div class="stat">
                        <div class="stat-title">With Annotations</div>
                        <div class="stat-value text-lg">
                            {datasetSummary.images_with_annotations}
                        </div>
                        <div class="stat-desc">
                            {formatPercentage(
                                datasetSummary.images_with_annotations,
                                datasetSummary.total_images,
                            )} coverage
                        </div>
                    </div>
                </div>

                <!-- Annotation Statistics -->
                <div class="stats stats-vertical bg-success/10 shadow">
                    <div class="stat">
                        <div class="stat-figure text-success">
                            <span class="material-symbols-rounded text-2xl"
                                >edit_note</span
                            >
                        </div>
                        <div class="stat-title">Total Annotations</div>
                        <div class="stat-value text-success text-2xl">
                            {datasetSummary.total_annotations}
                        </div>
                    </div>
                    <div class="stat">
                        <div class="stat-title">Avg per Image</div>
                        <div class="stat-value text-lg">
                            {formatAverage(
                                datasetSummary.total_annotations,
                                datasetSummary.images_with_annotations,
                            )}
                        </div>
                        {#if datasetSummary.annotation_types?.length > 0}
                            <div class="stat-desc">
                                Types: {datasetSummary.annotation_types.join(
                                    ", ",
                                )}
                            </div>
                        {/if}
                    </div>
                </div>

                <!-- Label Statistics -->
                <div class="stats stats-vertical bg-secondary/10 shadow">
                    <div class="stat">
                        <div class="stat-figure text-secondary">
                            <span class="material-symbols-rounded text-2xl"
                                >label</span
                            >
                        </div>
                        <div class="stat-title">Unique Labels</div>
                        <div class="stat-value text-secondary text-2xl">
                            {datasetSummary.unique_labels}
                        </div>
                    </div>
                    {#if datasetSummary.label_counts && Object.keys(datasetSummary.label_counts).length > 0}
                        <div class="stat">
                            <div class="stat-title">Label Distribution</div>
                            <ul
                                class="text-xs mt-2 space-y-1 max-h-24 overflow-y-auto"
                            >
                                {#each Object.entries(datasetSummary.label_counts) as [label, count] (label)}
                                    <li class="flex justify-between">
                                        <span
                                            class="badge badge-sm badge-ghost truncate max-w-[120px]"
                                            >{label}</span
                                        >
                                        <span class="font-mono">{count}</span>
                                    </li>
                                {/each}
                            </ul>
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    </div>
{/if}

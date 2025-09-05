<script lang="ts">
    export let datasetSummary: any; // Or a more specific type if you have one defined

    // Helper function, can be kept here or passed as prop if used elsewhere
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
    <div class="bg-white rounded-lg shadow-md p-4 mb-6">
        <h2 class="text-xl font-semibold text-gray-800 mb-3">
            Dataset Summary
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div class="bg-blue-50 p-3 rounded-md">
                <h3 class="text-sm font-medium text-blue-700">
                    File Statistics
                </h3>
                <p class="text-sm mt-1">
                    Total Images: <span class="font-medium"
                        >{datasetSummary.total_images}</span
                    >
                </p>
                <p class="text-sm mt-1">
                    Images with Annotations: <span
                        class="font-medium"
                        >{datasetSummary.images_with_annotations}</span
                    >
                </p>
                <p class="text-sm mt-1">
                    Annotation Coverage: <span class="font-medium"
                        >{formatPercentage(datasetSummary.images_with_annotations, datasetSummary.total_images)}</span
                    >
                </p>
            </div>

            <div class="bg-green-50 p-3 rounded-md">
                <h3 class="text-sm font-medium text-green-700">
                    Annotation Statistics
                </h3>
                <p class="text-sm mt-1">
                    Total Annotations: <span class="font-medium"
                        >{datasetSummary.total_annotations}</span
                    >
                </p>
                <p class="text-sm mt-1">
                    Average per Annotated Image: <span class="font-medium"
                        >{formatAverage(datasetSummary.total_annotations, datasetSummary.images_with_annotations)}</span
                    >
                </p>
                {#if datasetSummary.annotation_types && datasetSummary.annotation_types.length > 0}
                <p class="text-sm mt-1">
                    Annotation Types: <span class="font-medium"
                        >{datasetSummary.annotation_types.join(
                            ", ",
                        )}</span
                    >
                </p>
                {/if}
            </div>

            <div class="bg-purple-50 p-3 rounded-md">
                <h3 class="text-sm font-medium text-purple-700">
                    Label Statistics
                </h3>
                <p class="text-sm mt-1">
                    Unique Labels: <span class="font-medium"
                        >{datasetSummary.unique_labels}</span
                    >
                </p>
                {#if datasetSummary.label_counts && Object.keys(datasetSummary.label_counts).length > 0}
                    <p class="text-sm mt-1">Label Counts:</p>
                    <ul class="text-xs mt-1 space-y-1 max-h-32 overflow-y-auto pr-2"> 
                        {#each Object.entries(datasetSummary.label_counts) as [label, count] (label)}
                            <li class="flex justify-between">
                                <span class="font-medium truncate pr-2">{label}</span>
                                <span>{count}</span>
                            </li>
                        {/each}
                    </ul>
                {:else}
                    <p class="text-xs text-gray-500 mt-1 italic">No labels found.</p>
                {/if}
            </div>
        </div>
    </div>
{/if} 
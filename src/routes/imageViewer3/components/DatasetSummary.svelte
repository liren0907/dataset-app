<script lang="ts">
    export let summary: any;
</script>

{#if summary}
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
                        >{summary.total_images}</span
                    >
                </p>
                <p class="text-sm mt-1">
                    Images with Annotations: <span class="font-medium"
                        >{summary.images_with_annotations}</span
                    >
                </p>
                <p class="text-sm mt-1">
                    Annotation Coverage: <span class="font-medium"
                        >{(
                            (summary.images_with_annotations /
                                summary.total_images) *
                            100
                        ).toFixed(1)}%</span
                    >
                </p>
            </div>

            <div class="bg-green-50 p-3 rounded-md">
                <h3 class="text-sm font-medium text-green-700">
                    Annotation Statistics
                </h3>
                <p class="text-sm mt-1">
                    Total Annotations: <span class="font-medium"
                        >{summary.total_annotations}</span
                    >
                </p>
                <p class="text-sm mt-1">
                    Average per Image: <span class="font-medium"
                        >{(
                            summary.total_annotations /
                            summary.images_with_annotations
                        ).toFixed(1)}</span
                    >
                </p>
                <p class="text-sm mt-1">
                    Annotation Types: <span class="font-medium"
                        >{summary.annotation_types.join(", ")}</span
                    >
                </p>
            </div>

            <div class="bg-purple-50 p-3 rounded-md">
                <h3 class="text-sm font-medium text-purple-700">
                    Label Statistics
                </h3>
                <p class="text-sm mt-1">
                    Unique Labels: <span class="font-medium"
                        >{summary.unique_labels}</span
                    >
                </p>
                <p class="text-sm mt-1">Label Counts:</p>
                <!-- Make the list scrollable if it exceeds a certain height -->
                <ul
                    class="text-xs mt-1 space-y-1 max-h-32 overflow-y-auto pr-2"
                >
                    {#if summary.label_counts}
                        {#each Object.entries(summary.label_counts) as [label, count]}
                            <li class="flex justify-between">
                                <span class="font-medium truncate pr-2"
                                    >{label}</span
                                >
                                <span>{count}</span>
                            </li>
                        {/each}
                    {/if}
                </ul>
            </div>
        </div>
    </div>
{/if}

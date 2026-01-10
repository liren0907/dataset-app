<script lang="ts">
    import type { DatasetSummary } from "../services/dataService";
    export let summary: DatasetSummary;
</script>

{#if summary}
    <div class="bg-blue-50 p-3 rounded-md border border-blue-200">
        <h4 class="font-medium text-blue-900 mb-1">Dataset Summary</h4>
        <div class="text-sm text-blue-700 space-y-1">
            <p>📁 {summary.total_images} images</p>
            <p>🏷️ {summary.total_annotations} annotations</p>
            <p>🎯 {summary.unique_labels} unique labels</p>
        </div>
    </div>
{/if}

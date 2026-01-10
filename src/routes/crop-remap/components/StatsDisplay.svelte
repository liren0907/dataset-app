<script lang="ts">
    export let datasetSummary: any = null;
</script>

{#if datasetSummary}
    <div class="stats stats-vertical w-full">
        <div class="stat">
            <div class="stat-figure text-primary">
                <span class="material-symbols-rounded text-2xl">image</span>
            </div>
            <div class="stat-title">Images</div>
            <div class="stat-value text-primary text-2xl">
                {datasetSummary.total_images}
            </div>
        </div>
        <div class="stat">
            <div class="stat-figure text-secondary">
                <span class="material-symbols-rounded text-2xl">edit_note</span>
            </div>
            <div class="stat-title">Annotations</div>
            <div class="stat-value text-secondary text-2xl">
                {datasetSummary.total_annotations}
            </div>
        </div>
        <div class="stat">
            <div class="stat-figure text-accent">
                <span class="material-symbols-rounded text-2xl">label</span>
            </div>
            <div class="stat-title">Labels</div>
            <div class="stat-value text-accent text-2xl">
                {datasetSummary.unique_labels}
            </div>
        </div>
    </div>
{/if}

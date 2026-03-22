<script lang="ts">
    let { analyzing = false, datasetLoaded = false, disabled = false, onanalyze }: {
        analyzing: boolean;
        datasetLoaded: boolean;
        disabled: boolean;
        onanalyze?: () => void;
    } = $props();
</script>

<div class="form-control">
    <div class="label">
        <span class="label-text font-medium">Dataset Analysis</span>
    </div>
    <div class="flex items-center gap-3">
        <button
            onclick={() => onanalyze?.()}
            {disabled}
            class="btn btn-primary btn-sm"
        >
            {#if analyzing}
                <span class="loading loading-spinner loading-sm"></span>
                Analyzing...
            {:else}
                <span class="material-symbols-rounded icon-sm">analytics</span>
                Analyze Dataset
            {/if}
        </button>
        {#if datasetLoaded}
            <span class="badge badge-success gap-1">
                <span class="material-symbols-rounded icon-sm"
                    >check_circle</span
                >
                Dataset analyzed
            </span>
        {/if}
    </div>
    <div class="label">
        <span class="label-text-alt opacity-60"
            >Analyze your dataset to discover available labels and get smart
            suggestions.</span
        >
    </div>
</div>

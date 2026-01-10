<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let analyzing: boolean = false;
    export let datasetLoaded: boolean = false;
    export let disabled: boolean = false;

    const dispatch = createEventDispatcher();
</script>

<div class="form-control">
    <label class="label">
        <span class="label-text font-medium">Dataset Analysis</span>
    </label>
    <div class="flex items-center gap-3">
        <button
            on:click={() => dispatch("analyze")}
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
    <label class="label">
        <span class="label-text-alt opacity-60"
            >Analyze your dataset to discover available labels and get smart
            suggestions.</span
        >
    </label>
</div>

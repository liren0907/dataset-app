<script lang="ts">
    export let datasetLoaded: boolean = false;
    export let availableLabels: string[] = [];
    export let selectedParentLabel: string = "person";
    export let selectedChildLabels: string[] = [];
    export let paddingFactor: number = 1.2;
    export let datasetSummary: any = null;

    // Derived state for filtered child labels (excluding parent)
    $: filteredChildLabels = availableLabels.filter(
        (label) => label !== selectedParentLabel,
    );

    function getLabelCount(label: string): number {
        return datasetSummary?.label_counts?.[label] || 0;
    }
</script>

{#if datasetLoaded && availableLabels.length > 0}
    <!-- Parent Label -->
    <div class="form-control">
        <label class="label">
            <span class="label-text font-medium">Parent Label</span>
        </label>
        <select
            bind:value={selectedParentLabel}
            class="select select-bordered w-full"
        >
            {#each availableLabels as label}
                <option value={label}>
                    {label} ({getLabelCount(label)} annotations)
                </option>
            {/each}
        </select>
        <label class="label">
            <span class="label-text-alt opacity-60"
                >The label of the object to crop around.</span
            >
        </label>
    </div>

    <!-- Child Labels -->
    <div class="form-control">
        <label class="label">
            <span class="label-text font-medium">Required Child Labels</span>
        </label>
        <div
            class="max-h-48 overflow-y-auto space-y-2 p-4 bg-base-200 rounded-lg"
        >
            {#each filteredChildLabels as label}
                <label class="flex items-center cursor-pointer gap-3">
                    <input
                        type="checkbox"
                        bind:group={selectedChildLabels}
                        value={label}
                        class="checkbox checkbox-primary checkbox-sm"
                    />
                    <span class="label-text">
                        {label}
                        <span class="badge badge-ghost badge-sm ml-2">
                            {getLabelCount(label)}
                        </span>
                    </span>
                </label>
            {/each}
        </div>
        <label class="label">
            <span class="label-text-alt opacity-60">
                Only people wearing at least one of the selected items will be
                processed.
            </span>
        </label>
        {#if selectedChildLabels.length > 0}
            <div class="flex flex-wrap gap-1 mt-2">
                {#each selectedChildLabels as label}
                    <span class="badge badge-primary badge-sm">{label}</span>
                {/each}
            </div>
        {/if}
    </div>

    <!-- Padding Factor -->
    <div class="form-control">
        <label class="label">
            <span class="label-text font-medium">Padding Factor</span>
            <span class="label-text-alt badge badge-primary">
                {((paddingFactor - 1) * 100).toFixed(0)}% {paddingFactor > 1
                    ? "larger"
                    : paddingFactor < 1
                      ? "smaller"
                      : "original"}
            </span>
        </label>
        <div class="flex items-center gap-4">
            <input
                type="range"
                bind:value={paddingFactor}
                min="0.5"
                max="2.0"
                step="0.1"
                class="range range-primary flex-1"
            />
            <input
                type="number"
                bind:value={paddingFactor}
                min="0.5"
                max="2.0"
                step="0.1"
                class="input input-bordered input-sm w-20 text-center"
            />
        </div>
        <div class="flex justify-between text-xs opacity-50 mt-1">
            <span>0.5x (50% smaller)</span>
            <span>1.0x (original)</span>
            <span>2.0x (100% larger)</span>
        </div>
    </div>
{:else if !datasetLoaded}
    <div class="alert alert-warning">
        <span class="material-symbols-rounded">info</span>
        <span>
            Please analyze your dataset first to see available labels and
            configure processing options.
        </span>
    </div>
{/if}

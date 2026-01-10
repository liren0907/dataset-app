<script lang="ts">
    import type { DatasetSummary } from "../services/dataService";

    export let summary: DatasetSummary;
    export let availableLabels: string[] = [];
    export let selectedParentLabel: string;
    export let selectedChildLabels: string[] = [];
    export let paddingFactor: number = 1.2;

    function getFilteredChildLabels(): string[] {
        return availableLabels.filter((label) => label !== selectedParentLabel);
    }
</script>

{#if summary && availableLabels.length > 0}
    <div>
        <label class="block text-sm font-medium text-gray-700 mb-1"
            >Parent Label</label
        >
        <select
            bind:value={selectedParentLabel}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500 text-sm"
        >
            {#each availableLabels as label}
                <option value={label}>
                    {label} ({summary.label_counts[label] || 0} annotations)
                </option>
            {/each}
        </select>
        <p class="text-xs text-gray-500 mt-1">
            The label of the object to crop around.
        </p>
    </div>

    <!-- Dynamic Child Labels Selection -->
    <div>
        <label class="block text-sm font-medium text-gray-700 mb-2"
            >Required Child Labels</label
        >
        <div
            class="max-h-48 overflow-y-auto space-y-2 p-3 bg-gray-50 rounded-md border"
        >
            {#each getFilteredChildLabels() as label}
                <label class="flex items-center">
                    <input
                        type="checkbox"
                        bind:group={selectedChildLabels}
                        value={label}
                        class="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                    />
                    <span class="ml-2 text-sm text-gray-700">
                        {label} ({summary.label_counts[label] || 0} annotations)
                    </span>
                </label>
            {/each}
        </div>
        <p class="text-xs text-gray-500 mt-1">
            Only people wearing at least one of the selected items will be
            processed.
            {#if selectedChildLabels.length > 0}
                <br /><strong>Selected:</strong>
                {selectedChildLabels.join(", ")}
            {:else}
                <br /><strong>Selected:</strong> None
            {/if}
        </p>
    </div>

    <!-- Padding Factor -->
    <div>
        <label
            for="paddingFactor"
            class="block text-sm font-medium text-gray-700 mb-1"
        >
            Padding Factor
            <span class="text-indigo-600 font-medium"
                >({((paddingFactor - 1) * 100).toFixed(0)}% {paddingFactor > 1
                    ? "larger"
                    : paddingFactor < 1
                      ? "smaller"
                      : "original"})</span
            >
        </label>
        <div class="flex items-center gap-3">
            <input
                type="range"
                id="paddingFactor"
                bind:value={paddingFactor}
                min="0.5"
                max="2.0"
                step="0.1"
                class="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
            />
            <input
                type="number"
                bind:value={paddingFactor}
                min="0.5"
                max="2.0"
                step="0.1"
                class="w-20 px-2 py-1 border border-gray-300 rounded text-sm text-center"
            />
        </div>
        <div class="flex justify-between text-xs text-gray-500 mt-1">
            <span>0.5x (50% smaller)</span>
            <span>1.0x (original size)</span>
            <span>2.0x (100% larger)</span>
        </div>
        <p class="text-xs text-gray-500 mt-1">
            Controls how much larger the crop area should be compared to the
            parent bounding box. Larger values provide more context but may
            include unwanted background.
        </p>
    </div>
{:else}
    <div class="bg-yellow-50 p-3 rounded-md border border-yellow-200">
        <p class="text-sm text-yellow-800">
            📋 Please analyze your dataset first to see available labels and
            configure processing options.
        </p>
    </div>
{/if}

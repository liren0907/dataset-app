<script lang="ts">
    import {
        labelTaxonomy,
        type LabelClass,
    } from "$lib/stores/labelTaxonomyStore";
    import SectionLabel from "$lib/components/ui/SectionLabel.svelte";

    let newClassName = "";

    function handleAddClass() {
        const trimmed = newClassName.trim();
        if (trimmed) {
            labelTaxonomy.addClass(trimmed);
            newClassName = "";
        }
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === "Enter") handleAddClass();
    }
</script>

<div class="space-y-2 p-4 border-b border-base-200">
    <SectionLabel>Label Classes</SectionLabel>

    <div class="flex flex-wrap gap-1">
        {#each $labelTaxonomy as cls (cls.name)}
            <div
                class="badge badge-sm gap-1 cursor-default group"
                style="background-color: {cls.color}20; border-color: {cls.color}; color: {cls.color};"
            >
                {#if cls.shortcut}
                    <kbd class="kbd kbd-xs opacity-60">{cls.shortcut}</kbd>
                {/if}
                <span class="text-xs font-medium">{cls.name}</span>
                <button
                    class="opacity-0 group-hover:opacity-100 ml-0.5 hover:text-error transition-opacity"
                    on:click={() => labelTaxonomy.removeClass(cls.name)}
                    title="Remove class">×</button
                >
            </div>
        {/each}
    </div>

    <div class="flex gap-1">
        <input
            type="text"
            class="input input-xs input-bordered flex-1 bg-base-200 text-xs"
            placeholder="Add class..."
            bind:value={newClassName}
            on:keydown={handleKeyDown}
        />
        <button
            class="btn btn-xs btn-primary"
            on:click={handleAddClass}
            disabled={!newClassName.trim()}>+</button
        >
    </div>
</div>

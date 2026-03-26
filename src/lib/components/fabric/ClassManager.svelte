<script lang="ts">
    import {
        labelClasses,
        addClass,
        removeClass,
    } from "$lib/stores/labelTaxonomyStore.svelte";
    import SectionLabel from "$lib/components/ui/SectionLabel.svelte";
    import { TextInput } from "$lib/components/ui";

    let newClassName = $state("");

    function handleAddClass() {
        const trimmed = newClassName.trim();
        if (trimmed) {
            addClass(trimmed);
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
        {#each labelClasses as cls (cls.name)}
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
                    onclick={() => removeClass(cls.name)}
                    title="Remove class">×</button
                >
            </div>
        {/each}
    </div>

    <div class="flex gap-1">
        <TextInput
            size="xs"
            class="flex-1 bg-base-200 text-xs"
            placeholder="Add class..."
            bind:value={newClassName}
            onkeydown={handleKeyDown}
        />
        <button
            class="btn btn-xs btn-primary"
            onclick={handleAddClass}
            disabled={!newClassName.trim()}>+</button
        >
    </div>
</div>

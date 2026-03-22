<script lang="ts">
    import type { Snippet } from "svelte";

    /**
     * SelectionCard Component - /gallery Style
     * Selectable card button with icon, title, and description.
     * Used for option selection (e.g., export format).
     */

    let {
        selected = false,
        color = "primary",
        disabled = false,
        onselect,
        icon,
        title,
        description,
    }: {
        selected?: boolean;
        color?: "primary" | "secondary";
        disabled?: boolean;
        onselect?: () => void;
        icon?: Snippet;
        title?: Snippet;
        description?: Snippet;
    } = $props();

    function handleClick() {
        if (!disabled) {
            onselect?.();
        }
    }

    let selectedClass = $derived(
        selected
            ? `bg-base-100 border-${color} shadow-sm`
            : "border-base-300 hover:bg-base-200/50",
    );

    let iconBgClass = $derived(
        selected
            ? `bg-${color}/10 text-${color}`
            : "bg-base-200 text-base-content/40 group-hover:bg-base-300",
    );
</script>

<button
    class="flex items-center gap-3 p-3 rounded-xl border transition-all text-left group w-full {selectedClass}"
    class:opacity-50={disabled}
    onclick={handleClick}
    {disabled}
>
    <!-- Icon Container -->
    <div
        class="w-8 h-8 rounded-lg flex items-center justify-center transition-colors {iconBgClass}"
    >
        {#if icon}
            {@render icon()}
        {/if}
    </div>

    <!-- Text Content -->
    <div class="flex-1">
        <span class="block font-medium text-sm">
            {#if title}
                {@render title()}
            {/if}
        </span>
        <span class="block text-[10px] text-base-content/50">
            {#if description}
                {@render description()}
            {/if}
        </span>
    </div>
</button>

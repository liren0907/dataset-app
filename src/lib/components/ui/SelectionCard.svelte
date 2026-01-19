<script lang="ts">
    import { createEventDispatcher } from "svelte";

    /**
     * SelectionCard Component - /gallery Style
     * Selectable card button with icon, title, and description.
     * Used for option selection (e.g., export format).
     */

    export let selected: boolean = false;
    export let color: "primary" | "secondary" = "primary";
    export let disabled: boolean = false;

    const dispatch = createEventDispatcher();

    function handleClick() {
        if (!disabled) {
            dispatch("select");
        }
    }

    $: selectedClass = selected
        ? `bg-base-100 border-${color} shadow-sm`
        : "border-base-300 hover:bg-base-200/50";

    $: iconBgClass = selected
        ? `bg-${color}/10 text-${color}`
        : "bg-base-200 text-base-content/40 group-hover:bg-base-300";
</script>

<button
    class="flex items-center gap-3 p-3 rounded-xl border transition-all text-left group w-full {selectedClass}"
    class:opacity-50={disabled}
    on:click={handleClick}
    {disabled}
>
    <!-- Icon Container -->
    <div
        class="w-8 h-8 rounded-lg flex items-center justify-center transition-colors {iconBgClass}"
    >
        <slot name="icon" />
    </div>

    <!-- Text Content -->
    <div class="flex-1">
        <span class="block font-medium text-sm">
            <slot name="title" />
        </span>
        <span class="block text-[10px] text-base-content/50">
            <slot name="description" />
        </span>
    </div>
</button>

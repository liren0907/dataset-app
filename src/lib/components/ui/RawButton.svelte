<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let icon: string; // Material Symbol name
    export let label: string | undefined = undefined; // Optional text label
    export let tooltip: string | undefined = undefined;
    export let active: boolean = false;
    export let loading: boolean = false;
    export let size: "sm" | "md" | "lg" = "sm";
    export let variant: "ghost" | "soft" = "ghost";
    export let disabled: boolean = false;
    let className: string = "";
    export { className as class };

    const dispatch = createEventDispatcher();

    function handleClick(event: MouseEvent) {
        if (!disabled && !loading) {
            dispatch("click", event);
        }
    }

    $: sizeClass =
        size === "sm"
            ? "btn-sm text-lg"
            : size === "md"
              ? "btn-md text-xl"
              : "btn-lg text-2xl";

    $: variantClass =
        variant === "ghost"
            ? "btn-ghost text-base-content/70 hover:bg-base-200 border border-base-300"
            : "bg-base-200 text-base-content hover:bg-base-300 border border-base-300";

    $: activeClass = active
        ? "bg-primary text-primary-content hover:bg-primary hover:text-primary-content shadow-inner"
        : "";

    // Use btn-square only when no label
    $: shapeClass = label ? "gap-2 px-3" : "btn-square";
</script>

<div class="tooltip tooltip-bottom" data-tip={tooltip}>
    <button
        type="button"
        class="btn {sizeClass} {variantClass} {activeClass} {shapeClass} {className}"
        {disabled}
        class:loading
        on:click={handleClick}
    >
        {#if !loading}
            <span class="material-symbols-rounded">{icon}</span>
            {#if label}
                <span class="text-sm font-medium">{label}</span>
            {/if}
        {/if}
    </button>
</div>

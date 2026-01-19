<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let icon: string; // Material Symbol name
    export let tooltip: string | undefined = undefined;
    export let active: boolean = false;
    export let loading: boolean = false;
    export let size: "sm" | "md" | "lg" = "sm";
    export let variant: "ghost" | "soft" = "ghost";
    export let disabled: boolean = false;

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
            ? "btn-ghost text-base-content/70 hover:bg-base-200"
            : "bg-base-200 text-base-content hover:bg-base-300 border-none";

    $: activeClass = active
        ? "bg-primary text-primary-content hover:bg-primary hover:text-primary-content shadow-inner"
        : "";
</script>

<div class="tooltip tooltip-bottom" data-tip={tooltip}>
    <button
        type="button"
        class="btn btn-square {sizeClass} {variantClass} {activeClass}"
        {disabled}
        class:loading
        on:click={handleClick}
    >
        {#if !loading}
            <span class="material-symbols-rounded">{icon}</span>
        {/if}
    </button>
</div>

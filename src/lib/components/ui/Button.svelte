<script lang="ts">
    import { createEventDispatcher } from "svelte";

    /**
     * Button Component - /gallery Style
     * Uses the Flat Gray aesthetic from /gallery ExportModal.
     */

    export let variant: "default" | "ghost" = "default";
    export let size: "sm" | "md" = "sm";
    export let disabled: boolean = false;
    export let type: "button" | "submit" | "reset" = "button";
    export let minWidth: string | null = null;
    let className: string = "";
    export { className as class };

    const dispatch = createEventDispatcher();

    function handleClick(event: MouseEvent) {
        if (!disabled) {
            dispatch("click", event);
        }
    }

    $: baseClass = "btn border-none font-normal transition-all";

    $: sizeClass = size === "sm" ? "btn-sm" : "";

    $: variantClass =
        variant === "ghost"
            ? "btn-ghost bg-base-200 hover:bg-base-300 text-base-content/70"
            : "bg-base-200 hover:bg-base-300 text-base-content";

    $: style = minWidth ? `min-width: ${minWidth};` : "";
</script>

<button
    {type}
    class="{baseClass} {sizeClass} {variantClass} {className}"
    {style}
    {disabled}
    on:click={handleClick}
>
    <slot />
</button>

<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let options: Array<{
        value: any;
        icon: string;
        tooltip?: string;
    }> = [];
    export let value: any;
    export let size: "sm" | "md" = "sm";

    const dispatch = createEventDispatcher();

    function select(optionValue: any) {
        if (value !== optionValue) {
            value = optionValue;
            dispatch("change", value);
        }
    }

    $: sizeClass = size === "sm" ? "btn-sm" : "btn-md";
</script>

<div class="join border border-base-300 bg-base-100 rounded-lg p-1 space-x-1">
    {#each options as option}
        <div class="tooltip" data-tip={option.tooltip}>
            <button
                class="btn {sizeClass} btn-square join-item border-none hover:bg-base-200 transition-all duration-200"
                class:bg-base-200={value === option.value}
                class:shadow-inner={value === option.value}
                class:text-primary={value === option.value}
                class:text-base-content-secondary={value !== option.value}
                on:click={() => select(option.value)}
                aria-label={option.tooltip}
            >
                <span class="material-symbols-rounded text-lg">
                    {option.icon}
                </span>
            </button>
        </div>
    {/each}
</div>

<style>
    /* Override join defaults to allow gap */
    .join .join-item {
        border-radius: 0.5rem; /* Rounded inner buttons */
    }
    .join {
        display: flex; /* Flexbox for better control than daisyui join sometimes */
    }
</style>

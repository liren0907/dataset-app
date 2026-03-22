<script lang="ts">
    let {
        options = [],
        value = $bindable(),
        size = "sm",
        onchange,
    }: {
        options?: Array<{
            value: any;
            label?: string;
            icon?: string;
            tooltip?: string;
        }>;
        value?: any;
        size?: "sm" | "md";
        onchange?: (value: any) => void;
    } = $props();

    function select(optionValue: any) {
        if (value !== optionValue) {
            value = optionValue;
            onchange?.(value);
        }
    }

    let sizeClass = $derived(size === "sm" ? "btn-sm" : "btn-md");
</script>

<div class="join border border-base-300 bg-base-100 rounded-lg p-1 space-x-1">
    {#each options as option}
        <div class="tooltip" data-tip={option.tooltip}>
            <button
                class="btn {sizeClass} join-item border-none hover:bg-base-200 transition-all duration-200 gap-2 px-4"
                class:bg-base-200={value === option.value}
                class:shadow-inner={value === option.value}
                class:text-base-content={value === option.value}
                class:text-base-content-secondary={value !== option.value}
                onclick={() => select(option.value)}
                aria-label={option.label || option.tooltip}
            >
                {#if option.icon}
                    <span class="material-symbols-rounded text-lg"
                        >{option.icon}</span
                    >
                {/if}
                {#if option.label}
                    <span>{option.label}</span>
                {/if}
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

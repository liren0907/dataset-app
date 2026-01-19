<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { fly } from "svelte/transition";

    export let variant: "info" | "success" | "warning" | "error" = "info";
    export let title: string | undefined = undefined;
    export let dismissible: boolean = false;
    export let icon: string | undefined = undefined;

    const dispatch = createEventDispatcher();

    // Mapping variants to icons if not provided
    const defaultIcons = {
        info: "info",
        success: "check_circle",
        warning: "warning",
        error: "error",
    };

    $: resolvedIcon = icon || defaultIcons[variant];

    $: alertClass = {
        info: "alert-info",
        success: "alert-success",
        warning: "alert-warning",
        error: "alert-error",
    }[variant];
</script>

<div
    role="alert"
    class="alert {alertClass} shadow-sm"
    transition:fly|local={{ y: -10, duration: 200 }}
>
    <span class="material-symbols-rounded">{resolvedIcon}</span>

    <div class="flex-1">
        {#if title}
            <h3 class="font-bold">{title}</h3>
        {/if}
        <div class="text-sm">
            <slot />
        </div>
    </div>

    {#if dismissible}
        <button
            class="btn btn-sm btn-circle btn-ghost"
            on:click={() => dispatch("close")}
        >
            <span class="material-symbols-rounded">close</span>
        </button>
    {/if}
</div>

<script lang="ts">
    import { toastStore } from "$lib/stores/toastStore";
    import { fly } from "svelte/transition";

    const variantStyles = {
        success: "alert-success",
        error: "alert-error",
        info: "alert-info",
        warning: "alert-warning",
    };

    const variantIcons = {
        success: "check_circle",
        error: "error",
        info: "info",
        warning: "warning",
    };
</script>

{#if $toastStore.length > 0}
    <div class="toast toast-top toast-end z-50">
        {#each $toastStore as toast (toast.id)}
            <div
                class="alert {variantStyles[toast.variant]} shadow-lg"
                transition:fly={{ x: 100, duration: 300 }}
            >
                <span class="material-symbols-rounded"
                    >{variantIcons[toast.variant]}</span
                >
                <span>{toast.message}</span>
                <button
                    class="btn btn-ghost btn-xs"
                    on:click={() => toastStore.dismiss(toast.id)}
                >
                    <span class="material-symbols-rounded text-sm">close</span>
                </button>
            </div>
        {/each}
    </div>
{/if}

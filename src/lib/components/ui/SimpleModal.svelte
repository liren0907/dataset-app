<script lang="ts">
    import { createEventDispatcher, onMount } from "svelte";
    import { fade, scale } from "svelte/transition";
    import IconButton from "./IconButton.svelte";

    export let isOpen: boolean = false;
    export let title: string;
    export let maxWidth: string = "max-w-md";

    const dispatch = createEventDispatcher();

    // Close on escape
    function handleKeydown(event: KeyboardEvent) {
        if (event.key === "Escape" && isOpen) {
            close();
        }
    }

    function close() {
        dispatch("close");
    }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
    <div
        class="modal modal-open z-[60]"
        role="dialog"
        aria-labelledby="modal-title"
    >
        <!-- Backdrop -->
        <button
            type="button"
            class="modal-backdrop cursor-default"
            on:click={close}
            on:keydown={(e) => e.key === "Enter" && close()}
            aria-label="Close modal"
        ></button>

        <!-- Modal Box -->
        <div
            class="modal-box {maxWidth} p-0 bg-base-100 border border-base-300 shadow-2xl overflow-hidden flex flex-col max-h-[90vh]"
            transition:scale={{ start: 0.95, duration: 200 }}
        >
            <!-- Header -->
            <div
                class="px-6 py-4 flex items-center justify-between border-b border-base-200 bg-base-100 sticky top-0 z-10"
            >
                <h3 id="modal-title" class="font-bold text-lg">{title}</h3>
                <IconButton icon="close" size="sm" on:click={close} />
            </div>

            <!-- Content (Scrollable) -->
            <div class="p-6 overflow-y-auto flex-1">
                <slot />
            </div>

            <!-- Actions (Footer) -->
            {#if $$slots.actions}
                <div
                    class="px-6 py-4 bg-base-200/50 border-t border-base-200 flex justify-end items-center gap-3"
                >
                    <slot name="actions" />
                </div>
            {/if}
        </div>
    </div>
{/if}

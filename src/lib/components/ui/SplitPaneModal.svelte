<script lang="ts">
    import { createEventDispatcher } from "svelte";

    /**
     * SplitPaneModal Component - /gallery Style
     * 35%/65% left-right split layout from /gallery ExportModal.
     */

    export let show: boolean = false;
    export let title: string = "Modal Title";
    export let subtitle: string = "";
    export let icon: string = "settings";

    const dispatch = createEventDispatcher();

    function handleClose() {
        dispatch("close");
    }

    function handleBackdropClick() {
        handleClose();
    }
</script>

{#if show}
    <dialog class="modal modal-open">
        <!-- Main Fixed-Size Container -->
        <div
            class="modal-box w-11/12 max-w-5xl h-[600px] p-0 flex overflow-hidden rounded-2xl shadow-2xl bg-base-100"
        >
            <!-- Left Sidebar (35%) -->
            <div
                class="w-[35%] bg-base-200/50 border-r border-base-200 flex flex-col p-6 gap-6"
            >
                <!-- Header -->
                <div>
                    <h2
                        class="text-xl font-bold text-base-content flex items-center gap-2"
                    >
                        <span class="material-symbols-rounded text-primary"
                            >{icon}</span
                        >
                        {title}
                    </h2>
                    {#if subtitle}
                        <p class="text-xs text-base-content/60 mt-1">
                            {subtitle}
                        </p>
                    {/if}
                </div>

                <!-- Sidebar Content -->
                <div class="flex flex-col gap-3 flex-1">
                    <slot name="sidebar" />
                </div>

                <!-- Sidebar Footer (optional) -->
                <slot name="sidebar-footer" />
            </div>

            <!-- Right Content Panel (65%) -->
            <div class="w-[65%] flex flex-col h-full bg-base-100 relative">
                <!-- Close Button -->
                <button
                    on:click={handleClose}
                    class="absolute top-4 right-4 z-10 btn btn-sm btn-circle btn-ghost text-base-content/40 hover:text-base-content"
                    >✕</button
                >

                <!-- Scrollable Content -->
                <div class="flex-1 overflow-y-auto p-8 space-y-8">
                    <slot name="content" />
                </div>

                <!-- Footer Action -->
                <div
                    class="p-6 border-t border-base-100 bg-base-50/50 flex justify-end gap-3"
                >
                    <slot name="footer">
                        <button
                            class="btn btn-sm btn-ghost bg-base-200 hover:bg-base-300 text-base-content/70 border-none font-normal"
                            on:click={handleClose}>Close</button
                        >
                    </slot>
                </div>
            </div>
        </div>
        <form
            method="dialog"
            class="modal-backdrop bg-base-300/50 backdrop-blur-sm"
        >
            <button on:click={handleBackdropClick}>close</button>
        </form>
    </dialog>
{/if}

<style>
    /* Custom scrollbar for better aesthetics inside the modal */
    .overflow-y-auto::-webkit-scrollbar {
        width: 6px;
    }
    .overflow-y-auto::-webkit-scrollbar-track {
        background: transparent;
    }
    .overflow-y-auto::-webkit-scrollbar-thumb {
        background-color: rgba(0, 0, 0, 0.1);
        border-radius: 10px;
    }
    :global(.dark) .overflow-y-auto::-webkit-scrollbar-thumb {
        background-color: rgba(255, 255, 255, 0.1);
    }
</style>

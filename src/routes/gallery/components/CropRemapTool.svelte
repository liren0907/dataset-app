<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import { performCropAndRemap } from "../datasetService";

    // Props
    export let isOpen: boolean = false;

    const dispatch = createEventDispatcher();

    // Internal state
    let sourceDir: string = "";
    let outputDir: string = "";
    let parentLabel: string = "person";
    let localLoading: boolean = false;
    let localError: string = "";
    let localSuccess: string = "";

    // Reset messages when modal opens
    $: if (isOpen) {
        localError = "";
        localSuccess = "";
    }

    async function selectDirectory(type: "source" | "output") {
        localError = "";
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: `Select ${type === "source" ? "Source" : "Output"} Directory`,
            });

            if (selected && typeof selected === "string") {
                if (type === "source") {
                    sourceDir = selected;
                } else {
                    outputDir = selected;
                }
            }
        } catch (err) {
            localError = `Failed to select directory: ${err instanceof Error ? err.message : String(err)}`;
        }
    }

    async function handleRunCrop() {
        if (!sourceDir) {
            localError = "Please select a source directory.";
            return;
        }
        if (!outputDir) {
            localError = "Please select an output directory.";
            return;
        }
        if (!parentLabel.trim()) {
            localError = "Please enter a parent label.";
            return;
        }

        localLoading = true;
        localError = "";
        localSuccess = "";

        try {
            const message = await performCropAndRemap(
                sourceDir,
                outputDir,
                parentLabel,
            );
            localSuccess = message;

            // Dispatch completion event after short delay
            setTimeout(() => {
                dispatch("cropCompleted", { outputDir });
            }, 1500);
        } catch (err) {
            localError = `Processing failed: ${err instanceof Error ? err.message : String(err)}`;
        } finally {
            localLoading = false;
        }
    }

    function handleClose() {
        if (!localLoading) {
            dispatch("close");
        }
    }
</script>

{#if isOpen}
    <dialog
        class="modal modal-open"
        aria-modal="true"
        aria-labelledby="crop-modal-title"
    >
        <div
            class="modal-box max-w-2xl bg-base-100 p-0 overflow-hidden relative"
        >
            <!-- Header Accent -->
            <div
                class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-secondary to-accent opacity-50"
            ></div>

            <!-- Header -->
            <div class="px-8 pt-8 pb-4 flex justify-between items-center">
                <div>
                    <h3
                        id="crop-modal-title"
                        class="text-2xl font-bold text-base-content flex items-center gap-2"
                    >
                        Crop & Remap
                    </h3>
                    <p class="text-sm text-base-content/60 mt-1">
                        Simple crop based on parent class
                    </p>
                </div>
                <button
                    on:click={handleClose}
                    class="btn btn-sm btn-circle btn-ghost text-base-content/50 hover:text-base-content hover:bg-base-200 transition-colors"
                    aria-label="Close modal"
                    disabled={localLoading}>✕</button
                >
            </div>

            <!-- Content -->
            <div class="px-8 py-2 space-y-6">
                <!-- Status Messages -->
                {#if localError}
                    <div
                        class="alert alert-error rounded-xl shadow-sm border border-error/10"
                    >
                        <span class="material-symbols-rounded">error</span>
                        <span>{localError}</span>
                    </div>
                {/if}

                {#if localSuccess}
                    <div
                        class="alert alert-success rounded-xl shadow-sm border border-success/10 bg-success/10 text-success-content"
                    >
                        <span class="material-symbols-rounded"
                            >check_circle</span
                        >
                        <span>{localSuccess}</span>
                    </div>
                {/if}

                <!-- Info Card -->
                <div
                    class="p-4 bg-info/10 border border-info/20 rounded-xl text-sm text-base-content/80 flex gap-3 items-start"
                >
                    <div class="p-1 bg-info/20 rounded-full text-info shrink-0">
                        <span class="material-symbols-rounded text-lg block"
                            >info</span
                        >
                    </div>
                    <div class="leading-relaxed opacity-90">
                        This tool crops images around a specified <strong
                            >Parent Label</strong
                        > (e.g., "person") and remaps all child label coordinates
                        to the new cropped region.
                    </div>
                </div>

                <!-- Source Directory -->
                <div class="space-y-2">
                    <label
                        class="label font-bold text-xs text-base-content/40 uppercase tracking-wider pl-1 pb-1"
                    >
                        Input Source
                    </label>
                    <div
                        class="flex items-center w-full px-1 py-1 rounded-xl border border-base-300 bg-base-100 hover:border-base-content/30 focus-within:ring-2 focus-within:ring-secondary/20 focus-within:border-secondary transition-all shadow-sm"
                    >
                        <div class="pl-3 text-secondary/60">
                            <span class="material-symbols-rounded"
                                >folder_open</span
                            >
                        </div>
                        <input
                            type="text"
                            bind:value={sourceDir}
                            readonly
                            placeholder="Select source directory..."
                            class="input input-ghost w-full focus:outline-none border-none bg-transparent h-10 text-sm"
                        />
                        <button
                            on:click={() => selectDirectory("source")}
                            class="btn btn-sm btn-ghost bg-base-200/80 hover:bg-base-300 text-base-content/70 mr-1 rounded-lg font-medium"
                            disabled={localLoading}
                        >
                            Browse
                        </button>
                    </div>
                </div>

                <!-- Output Directory -->
                <div class="space-y-2">
                    <label
                        class="label font-bold text-xs text-base-content/40 uppercase tracking-wider pl-1 pb-1"
                    >
                        Destination
                    </label>
                    <div
                        class="flex items-center w-full px-1 py-1 rounded-xl border border-base-300 bg-base-100 hover:border-base-content/30 focus-within:ring-2 focus-within:ring-secondary/20 focus-within:border-secondary transition-all shadow-sm"
                    >
                        <div class="pl-3 text-secondary/60">
                            <span class="material-symbols-rounded">output</span>
                        </div>
                        <input
                            type="text"
                            bind:value={outputDir}
                            readonly
                            placeholder="Select output directory..."
                            class="input input-ghost w-full focus:outline-none border-none bg-transparent h-10 text-sm"
                        />
                        <button
                            on:click={() => selectDirectory("output")}
                            class="btn btn-sm btn-ghost bg-base-200/80 hover:bg-base-300 text-base-content/70 mr-1 rounded-lg font-medium"
                            disabled={localLoading}
                        >
                            Browse
                        </button>
                    </div>
                    <p class="text-xs text-base-content/40 px-2">
                        Processed images will be automatically loaded into the
                        gallery.
                    </p>
                </div>

                <!-- Parent Label -->
                <div class="space-y-2 pb-2">
                    <label
                        class="label font-bold text-xs text-base-content/40 uppercase tracking-wider pl-1 pb-1"
                        for="cropParentLabel"
                    >
                        Target Class (Parent)
                    </label>
                    <div
                        class="flex items-center w-full px-1 py-1 rounded-xl border border-base-300 bg-base-100 hover:border-base-content/30 focus-within:ring-2 focus-within:ring-secondary/20 focus-within:border-secondary transition-all shadow-sm"
                    >
                        <div class="pl-3 text-base-content/40">
                            <span class="material-symbols-rounded">label</span>
                        </div>
                        <input
                            type="text"
                            id="cropParentLabel"
                            bind:value={parentLabel}
                            placeholder="e.g., person, car..."
                            class="input input-ghost w-full focus:outline-none border-none bg-transparent h-10 text-sm font-medium"
                            disabled={localLoading}
                        />
                    </div>
                </div>
            </div>

            <!-- Footer -->
            <div
                class="px-8 py-5 bg-base-100 border-t border-base-100 flex justify-end gap-3 sticky bottom-0 z-10 w-full mt-4"
            >
                <button
                    type="button"
                    class="btn btn-ghost hover:bg-base-200 font-medium text-base-content/70"
                    on:click={handleClose}
                    disabled={localLoading}
                >
                    Cancel
                </button>
                <button
                    type="button"
                    class="btn btn-primary min-w-[140px] shadow-lg shadow-primary/20 border-none hover:bg-primary-focus hover:scale-[1.02] active:scale-[0.98] transition-all"
                    on:click={handleRunCrop}
                    disabled={localLoading ||
                        !sourceDir ||
                        !outputDir ||
                        !parentLabel.trim()}
                >
                    {#if localLoading}
                        <span
                            class="loading loading-spinner loading-sm text-primary-content"
                        ></span>
                    {:else}
                        <span>Run Processing</span>
                        <span class="material-symbols-rounded text-lg"
                            >arrow_forward</span
                        >
                    {/if}
                </button>
            </div>
        </div>
        <form
            method="dialog"
            class="modal-backdrop bg-base-300/50 backdrop-blur-sm"
        >
            <button on:click={handleClose} disabled={localLoading}>close</button
            >
        </form>
    </dialog>
{/if}

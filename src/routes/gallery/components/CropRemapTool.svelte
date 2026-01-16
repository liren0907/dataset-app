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
        <div class="modal-box max-w-2xl">
            <!-- Header -->
            <div class="flex justify-between items-center mb-4">
                <h3
                    id="crop-modal-title"
                    class="text-xl font-bold text-base-content flex items-center gap-2"
                >
                    <span class="material-symbols-rounded text-secondary"
                        >crop</span
                    >
                    Crop & Remap Tool
                </h3>
                <button
                    on:click={handleClose}
                    class="btn btn-sm btn-circle btn-ghost"
                    aria-label="Close modal"
                    disabled={localLoading}>✕</button
                >
            </div>

            <!-- Content -->
            <div class="space-y-6">
                {#if localError}
                    <div class="alert alert-error">
                        <span class="material-symbols-rounded">error</span>
                        <span>{localError}</span>
                    </div>
                {/if}

                {#if localSuccess}
                    <div class="alert alert-success">
                        <span class="material-symbols-rounded"
                            >check_circle</span
                        >
                        <span>{localSuccess}</span>
                    </div>
                {/if}

                <!-- Description -->
                <div
                    class="text-sm text-base-content/70 bg-base-200 rounded-lg p-4"
                >
                    <p class="flex items-start gap-2">
                        <span class="material-symbols-rounded text-info mt-0.5"
                            >info</span
                        >
                        <span>
                            This tool crops images around a parent label (e.g.,
                            "person") and remaps child label coordinates to the
                            cropped region. Useful for creating focused datasets
                            from larger annotated images.
                        </span>
                    </p>
                </div>

                <!-- Source Directory -->
                <div class="form-control">
                    <label class="label" for="cropSourceDir">
                        <span class="label-text font-medium"
                            >Source Directory</span
                        >
                    </label>
                    <div class="join w-full">
                        <input
                            type="text"
                            id="cropSourceDir"
                            bind:value={sourceDir}
                            readonly
                            placeholder="Select source directory..."
                            class="input input-bordered join-item flex-1 bg-base-200 text-base-content/70"
                        />
                        <button
                            on:click={() => selectDirectory("source")}
                            class="btn btn-neutral join-item"
                            disabled={localLoading}
                        >
                            Browse...
                        </button>
                    </div>
                </div>

                <!-- Output Directory -->
                <div class="form-control">
                    <label class="label" for="cropOutputDir">
                        <span class="label-text font-medium"
                            >Output Directory</span
                        >
                    </label>
                    <div class="join w-full">
                        <input
                            type="text"
                            id="cropOutputDir"
                            bind:value={outputDir}
                            readonly
                            placeholder="Select output directory..."
                            class="input input-bordered join-item flex-1 bg-base-200 text-base-content/70"
                        />
                        <button
                            on:click={() => selectDirectory("output")}
                            class="btn btn-neutral join-item"
                            disabled={localLoading}
                        >
                            Browse...
                        </button>
                    </div>
                    <p class="text-xs text-base-content/50 mt-1 ml-1">
                        Results will be loaded into the gallery automatically.
                    </p>
                </div>

                <!-- Parent Label -->
                <div class="form-control">
                    <label class="label" for="cropParentLabel">
                        <span class="label-text font-medium">Parent Label</span>
                    </label>
                    <input
                        type="text"
                        id="cropParentLabel"
                        bind:value={parentLabel}
                        placeholder="e.g., person, car, animal"
                        class="input input-bordered"
                        disabled={localLoading}
                    />
                    <p class="text-xs text-base-content/50 mt-1 ml-1">
                        The label of the object to crop around.
                    </p>
                </div>
            </div>

            <!-- Footer -->
            <div class="modal-action">
                <button
                    type="button"
                    class="btn btn-ghost"
                    on:click={handleClose}
                    disabled={localLoading}
                >
                    Cancel
                </button>
                <button
                    type="button"
                    class="btn btn-ghost border border-neutral"
                    on:click={handleRunCrop}
                    disabled={localLoading ||
                        !sourceDir ||
                        !outputDir ||
                        !parentLabel.trim()}
                >
                    {#if localLoading}
                        <span class="loading loading-spinner loading-sm"></span>
                        Processing...
                    {:else}
                        <span class="material-symbols-rounded text-sm"
                            >play_arrow</span
                        >
                        Run Crop & Remap
                    {/if}
                </button>
            </div>
        </div>
        <form method="dialog" class="modal-backdrop">
            <button on:click={handleClose} disabled={localLoading}>close</button
            >
        </form>
    </dialog>
{/if}

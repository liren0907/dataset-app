<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import { performCropAndRemap } from "../datasetService";

    export let isOpen: boolean = false;

    const dispatch = createEventDispatcher();

    // Internal state
    let sourceDir: string = "";
    let outputDir: string = "";
    let parentLabel: string = "person";
    let localLoading: boolean = false;
    let localError: string = "";
    let localSuccess: string = "";

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
                if (type === "source") sourceDir = selected;
                else outputDir = selected;
            }
        } catch (err) {
            localError = `Failed to select directory.`;
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
        if (!localLoading) dispatch("close");
    }
</script>

{#if isOpen}
    <dialog class="modal modal-open">
        <!-- Main Fixed-Size Container -->
        <div
            class="modal-box w-11/12 max-w-4xl h-[550px] p-0 flex overflow-hidden rounded-2xl shadow-2xl bg-base-100"
        >
            <!-- Left Sidebar (40%) -->
            <div
                class="w-[40%] bg-base-200/50 border-r border-base-200 flex flex-col p-6 gap-6 justify-between"
            >
                <div class="space-y-6">
                    <!-- Header -->
                    <div>
                        <h2
                            class="text-xl font-bold text-base-content flex items-center gap-2"
                        >
                            <span
                                class="material-symbols-rounded text-secondary"
                                >crop</span
                            >
                            Crop & Remap
                        </h2>
                        <p class="text-xs text-base-content/60 mt-1">
                            Isolate objects to new dataset
                        </p>
                    </div>

                    <!-- Paths Configuration -->
                    <div class="space-y-4">
                        <label
                            class="text-xs font-bold text-base-content/40 uppercase tracking-wider"
                            >I/O Configuration</label
                        >

                        <!-- Source Path (Compact) -->
                        <div>
                            <label
                                class="text-[10px] font-semibold text-base-content/50 mb-1 block pl-1"
                                >Source Dataset</label
                            >
                            <div
                                class="flex items-center w-full px-2 py-1.5 rounded-lg border border-base-300 bg-base-100 focus-within:border-secondary focus-within:ring-1 focus-within:ring-secondary/20 transition-all"
                            >
                                <span
                                    class="material-symbols-rounded text-secondary/70 text-sm ml-1"
                                    >folder_open</span
                                >
                                <input
                                    type="text"
                                    bind:value={sourceDir}
                                    readonly
                                    placeholder="Select source..."
                                    class="input input-ghost w-full h-8 text-xs focus:outline-none border-none bg-transparent px-2"
                                />
                                <button
                                    on:click={() => selectDirectory("source")}
                                    class="btn btn-sm btn-ghost bg-base-200 hover:bg-base-300 text-base-content/70 ml-2 px-4 rounded-md border-none font-normal"
                                    >Browse</button
                                >
                            </div>
                        </div>

                        <!-- Output Path -->
                        <div>
                            <label
                                class="text-[10px] font-semibold text-base-content/50 mb-1 block pl-1"
                                >Output Destination</label
                            >
                            <div
                                class="flex items-center w-full px-2 py-1.5 rounded-lg border border-base-300 bg-base-100 focus-within:border-secondary focus-within:ring-1 focus-within:ring-secondary/20 transition-all"
                            >
                                <span
                                    class="material-symbols-rounded text-secondary/70 text-sm ml-1"
                                    >output</span
                                >
                                <input
                                    type="text"
                                    bind:value={outputDir}
                                    readonly
                                    placeholder="Select output..."
                                    class="input input-ghost w-full h-8 text-xs focus:outline-none border-none bg-transparent px-2"
                                />
                                <button
                                    on:click={() => selectDirectory("output")}
                                    class="btn btn-sm btn-ghost bg-base-200 hover:bg-base-300 text-base-content/70 ml-2 px-4 rounded-md border-none font-normal"
                                    >Browse</button
                                >
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Info Block (Bottom of Sidebar) -->
                <div
                    class="p-4 bg-base-100 rounded-xl border border-base-300/50 shadow-sm text-xs text-base-content/70 leading-relaxed"
                >
                    <span class="font-bold block mb-1 text-base-content"
                        >How it works</span
                    >
                    This tool creates a new dataset by cropping around the
                    <span class="text-secondary font-medium">Parent Label</span
                    >. All internal annotations are remapped to the new image
                    coordinates.
                </div>
            </div>

            <!-- Right Content Panel (60%) -->
            <div class="w-[60%] flex flex-col h-full bg-base-100 relative">
                <!-- Close Button -->
                <button
                    on:click={handleClose}
                    class="absolute top-4 right-4 z-10 btn btn-sm btn-circle btn-ghost text-base-content/40 hover:text-base-content"
                    disabled={localLoading}>✕</button
                >

                <div
                    class="flex-1 flex flex-col items-center justify-center p-8 space-y-8"
                >
                    {#if localError}
                        <div
                            class="absolute top-16 left-8 right-8 alert alert-error py-2 px-4 rounded-lg text-sm shadow-sm animate-in fade-in slide-in-from-top-2"
                        >
                            <span class="material-symbols-rounded text-lg"
                                >error</span
                            >
                            <span>{localError}</span>
                        </div>
                    {/if}

                    {#if localSuccess}
                        <div
                            class="absolute top-16 left-8 right-8 alert alert-success py-2 px-4 rounded-lg text-sm shadow-sm animate-in fade-in slide-in-from-top-2"
                        >
                            <span class="material-symbols-rounded"
                                >check_circle</span
                            >
                            <span>{localSuccess}</span>
                        </div>
                    {/if}

                    <!-- Process Visualization -->
                    <div class="w-full max-w-sm">
                        <label
                            class="text-xs font-bold text-base-content/40 uppercase tracking-wider mb-6 block text-center"
                            >Operation Parameters</label
                        >

                        <div
                            class="bg-base-200/30 border border-base-200 rounded-2xl p-6 flex flex-col items-center gap-6"
                        >
                            <!-- Visual Flow -->
                            <div
                                class="flex items-center gap-3 text-base-content/30 select-none"
                            >
                                <div
                                    class="w-12 h-16 border-2 border-dashed border-current rounded flex items-center justify-center"
                                >
                                    <div
                                        class="w-6 h-8 bg-current opacity-20 transform scale-75"
                                    ></div>
                                </div>
                                <span
                                    class="material-symbols-rounded text-2xl animate-pulse"
                                    >arrow_forward</span
                                >
                                <div
                                    class="w-10 h-10 bg-secondary/10 border-2 border-secondary rounded flex items-center justify-center text-secondary relative"
                                >
                                    <span
                                        class="material-symbols-rounded text-lg"
                                        >crop</span
                                    >
                                    <div
                                        class="absolute -top-1 -right-1 w-2 h-2 bg-success rounded-full"
                                    ></div>
                                </div>
                            </div>

                            <!-- Input -->
                            <div class="w-full">
                                <label
                                    class="label text-xs font-bold text-base-content/50 uppercase pl-1 pb-1"
                                    for="parentLabel">Parent Class Label</label
                                >
                                <div class="relative">
                                    <input
                                        id="parentLabel"
                                        type="text"
                                        bind:value={parentLabel}
                                        placeholder="e.g. person"
                                        class="input input-lg input-bordered w-full text-center text-lg font-bold bg-base-100 shadow-sm focus:border-secondary focus:ring-4 focus:ring-secondary/10 transition-all"
                                    />
                                    <div
                                        class="absolute right-3 top-1/2 -translate-y-1/2 text-base-content/20 pointer-events-none"
                                    >
                                        <span class="material-symbols-rounded"
                                            >label</span
                                        >
                                    </div>
                                </div>
                                <p
                                    class="text-[10px] text-center mt-2 text-base-content/40"
                                >
                                    Enter the class name to crop around
                                </p>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Footer Action -->
                <div
                    class="p-6 border-t border-base-100 bg-base-50/50 flex justify-end gap-3"
                >
                    <button
                        class="btn btn-sm btn-ghost bg-base-200 hover:bg-base-300 text-base-content/70 border-none font-normal"
                        on:click={handleClose}
                        disabled={localLoading}>Cancel</button
                    >
                    <button
                        class="btn btn-sm bg-base-200 hover:bg-base-300 text-base-content border-none font-normal min-w-[140px]"
                        on:click={handleRunCrop}
                        disabled={localLoading ||
                            !sourceDir ||
                            !outputDir ||
                            !parentLabel.trim()}
                    >
                        {#if localLoading}
                            <span class="loading loading-spinner loading-xs"
                            ></span>
                        {:else}
                            Start Processing
                        {/if}
                    </button>
                </div>
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

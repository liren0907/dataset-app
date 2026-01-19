<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import { performCropAndRemap } from "../datasetService";
    import {
        Button,
        BrowseInput,
        SectionLabel,
        SplitPaneModal,
        Alert,
    } from "$lib/components/ui";

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

<SplitPaneModal
    show={isOpen}
    title="Crop & Remap"
    subtitle="Isolate objects to new dataset"
    icon="crop"
    on:close={handleClose}
>
    <!-- Sidebar -->
    <div slot="sidebar" class="flex flex-col h-full justify-between">
        <div class="space-y-6">
            <!-- Paths Configuration -->
            <div class="space-y-4">
                <SectionLabel>I/O Configuration</SectionLabel>

                <!-- Source Path -->
                <div>
                    <span
                        class="text-[10px] font-semibold text-base-content/50 mb-1 block pl-1"
                        >Source Dataset</span
                    >
                    <BrowseInput
                        value={sourceDir}
                        placeholder="Select source..."
                        icon="folder_open"
                        on:browse={() => selectDirectory("source")}
                    />
                </div>

                <!-- Output Path -->
                <div>
                    <span
                        class="text-[10px] font-semibold text-base-content/50 mb-1 block pl-1"
                        >Output Destination</span
                    >
                    <BrowseInput
                        value={outputDir}
                        placeholder="Select output..."
                        icon="output"
                        on:browse={() => selectDirectory("output")}
                    />
                </div>
            </div>
        </div>

        <!-- Info Block -->
        <div
            class="p-4 bg-base-100 rounded-xl border border-base-300/50 shadow-sm text-xs text-base-content/70 leading-relaxed"
        >
            <span class="font-bold block mb-1 text-base-content"
                >How it works</span
            >
            This tool creates a new dataset by cropping around the
            <span class="text-secondary font-medium">Parent Label</span>. All
            internal annotations are remapped to the new image coordinates.
        </div>
    </div>

    <!-- Main Content -->
    <div
        slot="content"
        class="flex flex-col items-center justify-center space-y-8 flex-1"
    >
        {#if localError}
            <div class="w-full">
                <Alert
                    variant="error"
                    dismissible
                    on:close={() => (localError = "")}>{localError}</Alert
                >
            </div>
        {/if}

        {#if localSuccess}
            <div class="w-full">
                <Alert variant="success">{localSuccess}</Alert>
            </div>
        {/if}

        <!-- Process Visualization -->
        <div class="w-full max-w-sm">
            <div class="text-center mb-6">
                <SectionLabel>Operation Parameters</SectionLabel>
            </div>

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
                        <span class="material-symbols-rounded text-lg"
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
                            <span class="material-symbols-rounded">label</span>
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

    <!-- Footer -->
    <div slot="footer" class="flex gap-3">
        <Button variant="ghost" on:click={handleClose} disabled={localLoading}
            >Cancel</Button
        >
        <Button
            on:click={handleRunCrop}
            disabled={localLoading ||
                !sourceDir ||
                !outputDir ||
                !parentLabel.trim()}
            minWidth="140px"
        >
            {#if localLoading}
                <span class="loading loading-spinner loading-xs"></span>
            {:else}
                Start Processing
            {/if}
        </Button>
    </div>
</SplitPaneModal>

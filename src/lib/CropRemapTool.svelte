<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import { performCropAndRemap } from "$lib/services/datasetService";
    import { Button, BrowseInput } from "$lib/components/ui";

    let internalCropSourceDir: string | null = null;
    let internalCropOutputDir: string | null = null;
    let internalParentLabel: string = "person";
    let internalCropLoading: boolean = false;
    let internalCropStatusMessage: string | null = null;
    let internalCropIsError: boolean = false;

    export let cropToolOpen: boolean = false;

    const dispatch = createEventDispatcher();

    async function selectDirectoryLocal(type: "source" | "output") {
        try {
            internalCropStatusMessage = null;
            internalCropIsError = false;

            const selected = await open({
                directory: true,
                multiple: false,
                title: `Select Crop ${type === "source" ? "Source" : "Output"} Directory`,
            });

            if (selected && typeof selected === "string") {
                if (type === "source") {
                    internalCropSourceDir = selected;
                } else {
                    internalCropOutputDir = selected;
                }
            }
        } catch (err) {
            internalCropStatusMessage = `Failed to select directory: ${err instanceof Error ? err.message : String(err)}`;
            internalCropIsError = true;
        }
    }

    async function handleRunCropAndRemapLocal() {
        if (
            !internalCropSourceDir ||
            !internalCropOutputDir ||
            !internalParentLabel
        ) {
            internalCropStatusMessage =
                "Please select source directory, output directory, and enter a parent label.";
            internalCropIsError = true;
            return;
        }

        internalCropLoading = true;
        internalCropStatusMessage = null;
        internalCropIsError = false;

        try {
            const message = await performCropAndRemap(
                internalCropSourceDir,
                internalCropOutputDir,
                internalParentLabel,
            );
            internalCropStatusMessage = message;
            internalCropIsError = false;
            dispatch("cropCompleted", { outputDir: internalCropOutputDir });
        } catch (err) {
            internalCropStatusMessage = `Processing failed: ${err instanceof Error ? err.message : String(err)}`;
            internalCropIsError = true;
        } finally {
            internalCropLoading = false;
        }
    }
</script>

{#if cropToolOpen}
    <div
        class="collapse collapse-arrow bg-base-200 rounded-xl mb-6"
        data-crop-tool
    >
        <input type="checkbox" checked />
        <div class="collapse-title text-lg font-medium flex items-center gap-2">
            <span class="material-symbols-rounded text-secondary">crop</span>
            Crop & Remap Tool
        </div>
        <div class="collapse-content space-y-4">
            <!-- Source Directory -->
            <div class="form-control">
                <label class="label">
                    <span class="label-text font-medium"
                        >Crop Source Directory</span
                    >
                </label>
                <BrowseInput
                    value={internalCropSourceDir || ""}
                    placeholder="Select source directory for cropping..."
                    on:browse={() => selectDirectoryLocal("source")}
                />
            </div>

            <!-- Output Directory -->
            <div class="form-control">
                <label class="label">
                    <span class="label-text font-medium"
                        >Crop Output Directory</span
                    >
                </label>
                <BrowseInput
                    value={internalCropOutputDir || ""}
                    placeholder="Select output directory for cropped results..."
                    on:browse={() => selectDirectoryLocal("output")}
                />
                <label class="label">
                    <span class="label-text-alt opacity-60"
                        >Results will be loaded into the viewer automatically
                        after processing.</span
                    >
                </label>
            </div>

            <!-- Parent Label Input -->
            <div class="form-control">
                <label class="label">
                    <span class="label-text font-medium">Parent Label</span>
                </label>
                <input
                    type="text"
                    bind:value={internalParentLabel}
                    placeholder="e.g., person, car"
                    class="input input-bordered"
                />
                <label class="label">
                    <span class="label-text-alt opacity-60"
                        >The label of the object to crop around.</span
                    >
                </label>
            </div>

            <!-- Run Button -->
            <Button
                variant="primary"
                fullWidth
                on:click={handleRunCropAndRemapLocal}
                disabled={internalCropLoading ||
                    !internalCropSourceDir ||
                    !internalCropOutputDir ||
                    !internalParentLabel}
            >
                {#if internalCropLoading}
                    <span class="loading loading-spinner loading-sm"></span>
                    Processing Crop & Remap...
                {:else}
                    <span class="material-symbols-rounded">play_arrow</span>
                    Run Crop & Remap and Load Results
                {/if}
            </Button>

            <!-- Status Messages -->
            {#if internalCropStatusMessage}
                <div
                    class="alert {internalCropIsError
                        ? 'alert-error'
                        : 'alert-success'}"
                >
                    <span class="material-symbols-rounded"
                        >{internalCropIsError ? "error" : "check_circle"}</span
                    >
                    <div>
                        <h3 class="font-bold">
                            {internalCropIsError ? "Error!" : "Status:"}
                        </h3>
                        <p class="text-sm">{internalCropStatusMessage}</p>
                    </div>
                </div>
            {/if}
        </div>
    </div>
{/if}

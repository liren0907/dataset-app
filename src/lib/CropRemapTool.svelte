<script lang="ts">
    import { Accordion, AccordionItem } from 'flowbite-svelte';
    import { createEventDispatcher } from 'svelte';
    import { open } from "@tauri-apps/plugin-dialog";
    import { performCropAndRemap } from "$lib/services/datasetService"; // Import service

    // Internal state
    let internalCropSourceDir: string | null = null;
    let internalCropOutputDir: string | null = null;
    let internalParentLabel: string = "person"; // Default value
    let internalCropLoading: boolean = false;
    let internalCropStatusMessage: string | null = null;
    let internalCropIsError: boolean = false;

    // Exported prop for accordion binding
    export let cropToolOpen: boolean = false;

    const dispatch = createEventDispatcher();

    async function selectDirectoryLocal(type: 'source' | 'output') {
        try {
            internalCropStatusMessage = null; // Clear previous messages
            internalCropIsError = false;

            const selected = await open({
                directory: true,
                multiple: false,
                title: `Select Crop ${type === 'source' ? 'Source' : 'Output'} Directory`,
            });

            if (selected && typeof selected === 'string') {
                if (type === 'source') {
                    internalCropSourceDir = selected;
                } else {
                    internalCropOutputDir = selected;
                }
            }
        } catch (err) {
            console.error("Error selecting crop directory:", err);
            internalCropStatusMessage = `Failed to select directory: ${err instanceof Error ? err.message : String(err)}`;
            internalCropIsError = true;
        }
    }

    async function handleRunCropAndRemapLocal() {
        if (!internalCropSourceDir || !internalCropOutputDir || !internalParentLabel) {
            internalCropStatusMessage = "Please select source directory, output directory, and enter a parent label.";
            internalCropIsError = true;
            return;
        }

        internalCropLoading = true;
        internalCropStatusMessage = null;
        internalCropIsError = false;

        try {
            const message = await performCropAndRemap(internalCropSourceDir, internalCropOutputDir, internalParentLabel);
            internalCropStatusMessage = message; // Success message from service
            internalCropIsError = false;
            dispatch('cropCompleted', { outputDir: internalCropOutputDir }); // Dispatch event with output dir
        } catch (err) {
            console.error("Error running crop/remap processing in tool:", err);
            internalCropStatusMessage = `Processing failed: ${err instanceof Error ? err.message : String(err)}`;
            internalCropIsError = true;
        } finally {
            internalCropLoading = false;
        }
    }

    // No longer need to dispatch individual updates for props that are now internal
    // function handleParentLabelInput(event) {
    //     parentLabel = event.target.value;
    //     dispatch('update:parentLabel', event.target.value);
    // }

</script>

<Accordion class="mb-6 border border-slate-200/60 rounded-xl bg-white/70 backdrop-blur shadow-sm" bind:open={cropToolOpen} on:change={() => dispatch('update:cropToolOpen', cropToolOpen)}>
    <AccordionItem>
        <span slot="header" class="text-lg font-medium text-slate-800">Crop & Remap Tool</span>
        <div class="p-4 bg-slate-50 space-y-4">
            <!-- Source Directory -->
            <div>
                <label class="block text-sm font-medium text-slate-700 mb-1" for="cropSourceDirInput">Crop Source Directory</label>
                <div class="flex items-center gap-2">
                    <input
                        type="text"
                        id="cropSourceDirInput"
                        readonly
                        placeholder="Select source directory for cropping..."
                        value={internalCropSourceDir || ''}
                        class="flex-1 px-3 py-2 border border-slate-300 rounded-md bg-slate-100 text-slate-600 text-sm truncate"
                    />
                    <button
                        on:click={() => selectDirectoryLocal('source')}
                        class="px-4 py-2 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded-md border border-slate-300 text-sm"
                    >
                        Browse...
                    </button>
                </div>
            </div>

            <!-- Output Directory -->
            <div>
                <label class="block text-sm font-medium text-slate-700 mb-1" for="cropOutputDirInput">Crop Output Directory</label>
                <div class="flex items-center gap-2">
                    <input
                        type="text"
                        id="cropOutputDirInput"
                        readonly
                        placeholder="Select output directory for cropped results..."
                        value={internalCropOutputDir || ''}
                        class="flex-1 px-3 py-2 border border-slate-300 rounded-md bg-slate-100 text-slate-600 text-sm truncate"
                    />
                    <button
                        on:click={() => selectDirectoryLocal('output')}
                        class="px-4 py-2 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded-md border border-slate-300 text-sm"
                    >
                        Browse...
                    </button>
                </div>
                <p class="text-xs text-slate-500 mt-1">Results will be loaded into the viewer automatically from this directory after processing.</p>
            </div>

            <!-- Parent Label Input -->
            <div>
                <label for="parentLabelInputTool" class="block text-sm font-medium text-slate-700 mb-1">Parent Label</label>
                <input
                    type="text"
                    id="parentLabelInputTool"
                    bind:value={internalParentLabel} 
                    placeholder="e.g., person, car"
                    class="w-full px-3 py-2 border border-slate-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500 text-sm"
                />
                <p class="text-xs text-slate-500 mt-1">The label of the object to crop around (only the first found instance per image will be used).</p>
            </div>

            <!-- Run Button -->
            <div class="pt-2">
                <button
                    on:click={handleRunCropAndRemapLocal}
                    disabled={internalCropLoading || !internalCropSourceDir || !internalCropOutputDir || !internalParentLabel}
                    class="w-full inline-flex justify-center items-center px-6 py-3 border border-transparent text-base font-medium rounded-md shadow-sm text-white bg-purple-600 hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                    {#if internalCropLoading}
                        <div class="mr-2 animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"></div>
                        Processing Crop & Remap...
                    {:else}
                        Run Crop & Remap and Load Results
                    {/if}
                </button>
            </div>

            <!-- Status Messages -->
            {#if internalCropStatusMessage}
                <div class={`p-3 rounded-md text-sm mt-4 ${internalCropIsError ? 'bg-red-50/80 backdrop-blur text-red-800 border border-red-200' : 'bg-green-50/80 backdrop-blur text-green-800 border border-green-200'}`}>
                    <p class="font-medium">{internalCropIsError ? 'Error!' : 'Status:'}</p>
                    <p>{internalCropStatusMessage}</p>
                </div>
            {/if}
        </div>
    </AccordionItem>
</Accordion> 
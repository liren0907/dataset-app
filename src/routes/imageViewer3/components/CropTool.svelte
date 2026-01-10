<script lang="ts">
    import { Accordion, AccordionItem } from "flowbite-svelte";
    import { createEventDispatcher } from "svelte";

    export let isOpen: boolean;
    export let sourceDir: string | null;
    export let outputDir: string | null;
    export let parentLabel: string;
    export let loading: boolean;
    export let statusMessage: string | null;
    export let isError: boolean;

    const dispatch = createEventDispatcher();
</script>

<Accordion class="mb-6 border border-gray-200 rounded-lg" bind:open={isOpen}>
    <AccordionItem>
        <span slot="header" class="text-lg font-medium text-gray-700"
            >Crop & Remap Tool</span
        >
        <div class="p-4 bg-gray-50 space-y-4">
            <!-- Source Directory -->
            <div>
                <span class="block text-sm font-medium text-gray-700 mb-1"
                    >Crop Source Directory</span
                >
                <div class="flex items-center gap-2">
                    <input
                        type="text"
                        readonly
                        placeholder="Select source directory for cropping..."
                        value={sourceDir || ""}
                        class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-100 text-gray-600 text-sm truncate"
                    />
                    <button
                        on:click={() => dispatch("selectSource")}
                        class="px-4 py-2 bg-gray-200 hover:bg-gray-300 text-gray-700 rounded-md border border-gray-300 text-sm"
                    >
                        Browse...
                    </button>
                </div>
            </div>

            <!-- Output Directory -->
            <div>
                <span class="block text-sm font-medium text-gray-700 mb-1"
                    >Crop Output Directory</span
                >
                <div class="flex items-center gap-2">
                    <input
                        type="text"
                        readonly
                        placeholder="Select output directory for cropped results..."
                        value={outputDir || ""}
                        class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-100 text-gray-600 text-sm truncate"
                    />
                    <button
                        on:click={() => dispatch("selectOutput")}
                        class="px-4 py-2 bg-gray-200 hover:bg-gray-300 text-gray-700 rounded-md border border-gray-300 text-sm"
                    >
                        Browse...
                    </button>
                </div>
                <p class="text-xs text-gray-500 mt-1">
                    Results will be loaded into the viewer automatically from
                    this directory after processing.
                </p>
            </div>

            <!-- Parent Label Input -->
            <div>
                <label
                    for="parentLabelInput"
                    class="block text-sm font-medium text-gray-700 mb-1"
                    >Parent Label</label
                >
                <input
                    type="text"
                    id="parentLabelInput"
                    bind:value={parentLabel}
                    placeholder="e.g., person, car"
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500 text-sm"
                />
                <p class="text-xs text-gray-500 mt-1">
                    The label of the object to crop around (only the first found
                    instance per image will be used).
                </p>
            </div>

            <!-- Run Button -->
            <div class="pt-2">
                <button
                    on:click={() => dispatch("runCrop")}
                    disabled={loading ||
                        !sourceDir ||
                        !outputDir ||
                        !parentLabel}
                    class="w-full inline-flex justify-center items-center px-6 py-3 border border-transparent text-base font-medium rounded-md shadow-sm text-white bg-purple-600 hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                    {#if loading}
                        <div
                            class="mr-2 animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"
                        ></div>
                        Processing Crop & Remap...
                    {:else}
                        Run Crop & Remap and Load Results
                    {/if}
                </button>
            </div>

            <!-- Status Messages -->
            {#if statusMessage}
                <div
                    class={`p-3 rounded-md text-sm mt-4 ${isError ? "bg-red-50 text-red-700" : "bg-green-50 text-green-700"}`}
                >
                    <p class="font-medium">{isError ? "Error!" : "Status:"}</p>
                    <p>{statusMessage}</p>
                </div>
            {/if}
        </div>
    </AccordionItem>
</Accordion>

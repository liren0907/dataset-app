<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { ProcessedImage } from "../datasetService";

    export let loading: boolean;
    export let directoryPath: string;
    export let images: ProcessedImage[];

    const dispatch = createEventDispatcher();
</script>

{#if loading}
    <div
        class="flex flex-col items-center justify-center p-12 bg-base-100 rounded-box shadow-sm border border-base-300 h-full"
    >
        <span class="loading loading-spinner loading-lg text-primary mb-4"
        ></span>
        <div class="text-lg font-medium">Loading dataset...</div>
        <div class="text-sm text-base-content/60 mt-2">
            Please wait while we process the images
        </div>
    </div>
{:else if !directoryPath}
    <!-- Empty State - No Directory -->
    <div
        class="flex flex-col items-center justify-center p-12 bg-base-100 rounded-box shadow-sm border border-base-300 h-full text-center"
    >
        <div
            class="w-24 h-24 bg-base-200 rounded-full flex items-center justify-center mb-6"
        >
            <span class="material-symbols-rounded text-6xl text-base-content/30"
                >folder_open</span
            >
        </div>
        <h3 class="text-xl font-bold mb-2">No Dataset Selected</h3>
        <p class="text-base-content/60 max-w-md mb-6">
            Please select a directory properly formatted for LabelMe or YOLO to
            inspect the dataset statistics.
        </p>
        <button
            on:click={() => dispatch("selectDirectory")}
            class="btn btn-primary"
        >
            Select Directory
        </button>
    </div>
{:else if images.length === 0}
    <!-- Empty State - No Images -->
    <div class="card bg-base-100 shadow-sm border border-base-300 h-full">
        <div class="card-body items-center text-center justify-center">
            <span
                class="material-symbols-rounded text-6xl text-base-content/20 mb-4"
                >image_not_supported</span
            >
            <h3 class="font-bold text-lg">No images found</h3>
            <p class="text-base-content/60">
                The selected directory doesn't contain any supported image
                files.
            </p>
        </div>
    </div>
{/if}

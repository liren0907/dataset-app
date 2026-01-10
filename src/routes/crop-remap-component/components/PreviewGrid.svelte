<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { PreviewImage } from "../services/dataService";

    export let images: PreviewImage[] = [];
    export let loading: boolean = false;

    const dispatch = createEventDispatcher();
</script>

<div class="bg-purple-50 p-4 rounded-md border border-purple-200">
    <h4 class="font-medium text-purple-900 mb-3 flex items-center">
        <span class="mr-2">👁️</span>
        Dataset Preview
        {#if loading}
            <div
                class="ml-2 animate-spin h-4 w-4 border-2 border-purple-600 border-t-transparent rounded-full"
            ></div>
        {/if}
    </h4>

    {#if loading}
        <div class="text-sm text-purple-700">Loading preview images...</div>
    {:else if images.length > 0}
        <div class="text-sm text-purple-700 mb-3">
            Here are {images.length} random annotated images from your dataset with
            annotations already drawn on them:
        </div>
        <div
            class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-3"
        >
            {#each images as image (image.path)}
                <div
                    class="bg-white rounded-lg shadow-sm overflow-hidden border border-purple-200"
                >
                    <div class="relative pb-[75%]">
                        <button
                            type="button"
                            class="absolute inset-0 w-full h-full focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2"
                            on:click={() => dispatch("select", image)}
                            aria-label={`View full size image: ${image.name}`}
                        >
                            <img
                                src={image.previewUrl}
                                alt={image.name}
                                class="w-full h-full object-cover hover:opacity-90 transition-opacity"
                            />
                        </button>
                        <div
                            class="absolute top-1 right-1 bg-purple-600 text-white text-xs px-2 py-0.5 rounded-full"
                        >
                            Preview
                        </div>
                    </div>
                    <div class="p-2">
                        <p
                            class="text-xs text-gray-600 truncate"
                            title={image.name}
                        >
                            {image.name}
                        </p>
                    </div>
                </div>
            {/each}
        </div>
        <div class="text-xs text-purple-600 mt-2">
            Click any image to view full size (annotations are already drawn on
            the images)
        </div>
    {:else}
        <div class="text-sm text-purple-700">
            No annotated images found for preview
        </div>
    {/if}
</div>

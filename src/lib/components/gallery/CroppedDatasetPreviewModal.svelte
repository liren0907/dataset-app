<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import SimpleModal from "$lib/components/ui/SimpleModal.svelte";
    import type { KonvaImageData } from "$lib/services/gallery/konvaService";

    export let isOpen: boolean = false;
    export let loading: boolean = false;
    export let error: string = "";
    export let outputPath: string = "";
    export let images: KonvaImageData[] = [];
    export let sampleCount: number = 0;

    const dispatch = createEventDispatcher<{
        close: void;
        selectImage: { image: KonvaImageData };
    }>();

    function handleClose() {
        dispatch("close");
    }

    function handleSelect(image: KonvaImageData) {
        dispatch("selectImage", { image });
    }

    function formatPath(path: string): string {
        if (!path) return "";
        const segments = path.split(/[/\\]/);
        if (segments.length <= 2) return path;
        return `.../${segments.slice(-2).join("/")}`;
    }

    $: displayPath = formatPath(outputPath);
    $: title = outputPath
        ? `Preview: ${displayPath} (${images.length}/${sampleCount})`
        : `Preview (${images.length}/${sampleCount})`;
</script>

<SimpleModal isOpen={isOpen} title={title} maxWidth="max-w-5xl" on:close={handleClose}>
    {#if loading}
        <div class="flex items-center justify-center py-10 text-base-content/60">
            <span class="loading loading-spinner loading-md"></span>
            <span class="ml-3">Generating previews...</span>
        </div>
    {:else if error}
        <div class="alert alert-error">
            <span class="material-symbols-rounded">error</span>
            <span>{error}</span>
        </div>
    {:else if images.length === 0}
        <div class="alert alert-info">
            <span class="material-symbols-rounded">info</span>
            <span>No previews available for this dataset.</span>
        </div>
    {:else}
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
            {#each images as image (image.path)}
                <button
                    type="button"
                    class="card bg-base-100 border border-base-200 hover:border-primary/40 hover:shadow-md transition-all text-left"
                    on:click={() => handleSelect(image)}
                >
                    <figure class="relative w-full pb-[75%] bg-base-200">
                        <img
                            src={image.previewUrl}
                            alt={image.name}
                            class="absolute inset-0 w-full h-full object-cover"
                        />
                    </figure>
                    <div class="p-2 flex items-center justify-between gap-2">
                        <span class="text-xs font-medium truncate">{image.name}</span>
                        <span class="badge badge-xs badge-primary">
                            {image.annotations?.length || 0}
                        </span>
                    </div>
                </button>
            {/each}
        </div>
    {/if}
</SimpleModal>

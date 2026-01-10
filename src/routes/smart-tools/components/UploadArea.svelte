<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let isDragHover: boolean = false;

    const dispatch = createEventDispatcher();

    function handleSelectFile() {
        dispatch("selectFile");
    }
</script>

<div
    class="border-2 border-dashed rounded-xl p-12 text-center transition-all duration-200
    {isDragHover
        ? 'border-primary bg-primary/10'
        : 'border-base-300 hover:border-primary/50'}"
>
    <div class="space-y-4">
        {#if isDragHover}
            <span class="material-symbols-rounded text-6xl text-primary"
                >upload_file</span
            >
            <p class="text-lg font-medium">Drop your image here</p>
        {:else}
            <span class="material-symbols-rounded text-6xl opacity-50"
                >image</span
            >
            <p class="text-lg">Drag and drop an image here</p>
            <p class="text-sm opacity-60">or</p>
        {/if}
        <button on:click={handleSelectFile} class="btn btn-primary">
            <span class="material-symbols-rounded">cloud_upload</span>
            Choose Image
        </button>
        <p class="text-xs opacity-50">
            Supported formats: PNG, JPG, JPEG, WEBP, BMP, GIF
        </p>
    </div>
</div>

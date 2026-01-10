<script lang="ts">
    import { createEventDispatcher } from "svelte";
    const dispatch = createEventDispatcher();
</script>

<div class="card bg-base-200 border border-base-content/10">
    <div class="card-body items-center text-center py-16">
        <span class="material-symbols-rounded text-6xl text-primary opacity-50"
            >folder_open</span
        >
        <h2 class="card-title mt-4">No Directory Selected</h2>
        <p class="opacity-70 max-w-md">
            Select a directory containing your images and LabelMe (.json)
            annotations to begin.
        </p>
        <div class="card-actions mt-6">
            <button
                class="btn btn-ghost border border-base-content/20"
                on:click={() => dispatch("selectDirectory")}
            >
                <span class="material-symbols-rounded">folder_open</span>
                Select Directory
            </button>
        </div>
        <p class="text-sm opacity-50 mt-4">
            Alternatively, you can use the Crop & Remap tool to process a
            dataset.
        </p>
    </div>
</div>

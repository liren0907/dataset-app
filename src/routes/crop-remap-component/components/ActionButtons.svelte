<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let loading: boolean = false;
    export let disabled: boolean = false;
    export let successMessage: string | null = null;
    export let errorMessage: string | null = null;
    export let validationError: string | null = null;

    const dispatch = createEventDispatcher();
</script>

<div class="mb-6">
    <button
        on:click={() => dispatch("click")}
        disabled={loading || disabled}
        class="w-full inline-flex justify-center items-center px-6 py-3 border border-transparent text-base font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
    >
        {#if loading}
            <div
                class="mr-2 animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"
            ></div>
            Processing...
        {:else}
            Run Crop & Remap
        {/if}
    </button>

    <!-- Validation Warning -->
    {#if validationError}
        <p class="text-sm text-red-600 mt-2">⚠️ {validationError}</p>
    {/if}
</div>

<!-- Status Messages -->
<div class="space-y-3">
    {#if successMessage}
        <div class="bg-green-50 text-green-700 p-3 rounded-md text-sm">
            <p class="font-medium">Success!</p>
            <p>{successMessage}</p>
        </div>
    {/if}
    {#if errorMessage}
        <div class="bg-red-50 text-red-700 p-3 rounded-md text-sm">
            <p class="font-medium">Error!</p>
            <p>{errorMessage}</p>
        </div>
    {/if}
</div>

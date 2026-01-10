<script lang="ts">
    import { createEventDispatcher } from "svelte";
    export let error: string;
    const dispatch = createEventDispatcher();
</script>

{#if error}
    <div class="alert alert-error mb-6">
        <span class="material-symbols-rounded">error</span>
        <span>{error}</span>
        <button class="btn btn-ghost btn-sm" on:click={() => dispatch("close")}>
            <span class="material-symbols-rounded">close</span>
        </button>
    </div>
{/if}

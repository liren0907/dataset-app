<script lang="ts">
    let { error, onclose }: { error: string; onclose?: () => void } = $props();
</script>

{#if error}
    <div class="alert alert-error mb-6">
        <span class="material-symbols-rounded">error</span>
        <span>{error}</span>
        <button class="btn btn-ghost btn-sm" onclick={() => onclose?.()}>
            <span class="material-symbols-rounded">close</span>
        </button>
    </div>
{/if}

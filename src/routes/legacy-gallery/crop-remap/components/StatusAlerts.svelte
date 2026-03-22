<script lang="ts">
    let { successMessage = null, errorMessage = null }: {
        successMessage: string | null;
        errorMessage: string | null;
    } = $props();
</script>

{#if successMessage}
    <div class="alert alert-success mt-4">
        <span class="material-symbols-rounded">check_circle</span>
        <div>
            <h3 class="font-bold">Success!</h3>
            <p class="text-sm">{successMessage}</p>
        </div>
    </div>
{/if}
{#if errorMessage}
    <div class="alert alert-error mt-4">
        <span class="material-symbols-rounded">error</span>
        <div>
            <h3 class="font-bold">Error!</h3>
            <p class="text-sm">{errorMessage}</p>
        </div>
    </div>
{/if}

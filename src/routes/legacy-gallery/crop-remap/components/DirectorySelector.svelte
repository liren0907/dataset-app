<script lang="ts">
    let { label, value = null, placeholder = "Select directory...", description = null, onbrowse }: {
        label: string;
        value: string | null;
        placeholder?: string;
        description?: string | null;
        onbrowse?: () => void;
    } = $props();
</script>

<div class="form-control">
    <label class="label">
        <span class="label-text font-medium">{label}</span>
    </label>
    <div class="join w-full">
        <input
            type="text"
            readonly
            {placeholder}
            value={value || ""}
            class="input input-bordered join-item flex-1"
        />
        <button
            onclick={() => onbrowse?.()}
            class="btn btn-ghost join-item"
        >
            <span class="material-symbols-rounded">folder_open</span>
        </button>
    </div>
    {#if description}
        <label class="label">
            <span class="label-text-alt opacity-60">{description}</span>
        </label>
    {/if}
</div>

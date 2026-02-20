<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let label: string;
    export let value: string | null = null;
    export let placeholder: string = "Select directory...";
    export let description: string | null = null;

    const dispatch = createEventDispatcher();
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
            on:click={() => dispatch("browse")}
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

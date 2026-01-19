<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let label: string;
    export let count: number | undefined = undefined;
    export let state: "active" | "excluded" | "neutral" = "active";
    // neutral is for non-interactive display or default state

    const dispatch = createEventDispatcher();

    $: badgeClass = {
        active: "badge-primary",
        excluded: "badge-ghost opacity-50 line-through decoration-1",
        neutral: "badge-neutral",
    }[state];
</script>

<button
    type="button"
    class="badge {badgeClass} gap-2 cursor-pointer hover:scale-105 transition-transform duration-100 p-3 h-auto"
    on:click={() => dispatch("click")}
    on:contextmenu|preventDefault={() => dispatch("toggle")}
    title="Click to toggle inclusion"
>
    <span class="font-mono text-sm">{label}</span>
    {#if count !== undefined}
        <div class="badge badge-sm badge-outline opacity-80 border-current">
            {count}
        </div>
    {/if}
</button>

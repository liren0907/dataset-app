<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let label: string;
    export let count: number | undefined = undefined;
    export let state: "active" | "excluded" | "neutral" = "active";
    // neutral is for non-interactive display or default state

    const dispatch = createEventDispatcher();

    // Updated styling with borders and better feedback
    $: badgeClass = {
        active: "bg-primary text-primary-content border-2 border-primary shadow-sm",
        excluded:
            "bg-transparent text-base-content/50 border border-base-300 line-through decoration-1",
        neutral:
            "bg-transparent text-base-content border border-base-300 hover:bg-base-200 hover:border-base-400",
    }[state];
</script>

<button
    type="button"
    class="badge {badgeClass} gap-2 cursor-pointer transition-all duration-150 p-3 h-auto
           hover:scale-105 active:scale-95"
    on:click={() => dispatch("click")}
    on:contextmenu|preventDefault={() => dispatch("toggle")}
    title="Click to toggle inclusion"
>
    <span class="font-mono text-sm">{label}</span>
    {#if count !== undefined}
        <div
            class="badge badge-sm {state === 'active'
                ? 'bg-white/90 text-primary border-0 font-semibold'
                : 'badge-outline opacity-80 border-current'}"
        >
            {count}
        </div>
    {/if}
</button>

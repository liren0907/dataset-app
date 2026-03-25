<!-- Escape the parent layout's padding so this page fills the full content pane -->
<script lang="ts">
    import type { Snippet } from 'svelte';

    let { children }: { children: Snippet } = $props();
</script>

<div class="h-screen -mx-6 -mt-8 overflow-hidden">
    {@render children()}
</div>

<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let outputPath: string;
    export let imageCount: number = 0;
    export let parentLabel: string = "";
    export let childLabels: string[] = [];
    export let createdAt: Date = new Date();

    const dispatch = createEventDispatcher<{
        preview: { outputPath: string };
        openInGallery: { outputPath: string };
        remove: { outputPath: string };
    }>();

    function formatPath(path: string): string {
        // Show only the last 2 segments of the path
        const segments = path.split(/[/\\]/);
        if (segments.length <= 2) return path;
        return "..." + segments.slice(-2).join("/");
    }

    function formatTime(date: Date): string {
        return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    }

    function handlePreview() {
        dispatch("preview", { outputPath });
    }

    function handleOpenInGallery() {
        dispatch("openInGallery", { outputPath });
    }

    function handleRemove() {
        dispatch("remove", { outputPath });
    }
</script>

<div class="card bg-base-100 border border-base-200 shadow-sm hover:shadow-md transition-shadow">
    <div class="card-body p-4 gap-3">
        <!-- Header Row -->
        <div class="flex items-start justify-between gap-2">
            <div class="flex items-center gap-2 min-w-0">
                <div class="w-8 h-8 rounded-lg bg-success/10 flex items-center justify-center flex-shrink-0">
                    <span class="material-symbols-rounded text-success text-lg">check_circle</span>
                </div>
                <div class="min-w-0">
                    <h4 class="font-bold text-sm text-base-content truncate" title={outputPath}>
                        {formatPath(outputPath)}
                    </h4>
                    <p class="text-xs text-base-content/50">
                        Created at {formatTime(createdAt)}
                    </p>
                </div>
            </div>
            <button
                on:click={handleRemove}
                class="btn btn-ghost btn-xs btn-square text-base-content/40 hover:text-error hover:bg-error/10"
                title="Remove from list"
            >
                <span class="material-symbols-rounded text-sm">close</span>
            </button>
        </div>

        <!-- Stats Row -->
        <div class="flex items-center gap-4 text-sm">
            <div class="flex items-center gap-1.5">
                <span class="material-symbols-rounded text-base text-primary">photo_library</span>
                <span class="font-bold">{imageCount}</span>
                <span class="text-base-content/50">images</span>
            </div>
            <div class="flex items-center gap-1.5">
                <span class="material-symbols-rounded text-base text-secondary">label</span>
                <span class="font-medium text-base-content/70">{parentLabel}</span>
            </div>
            {#if childLabels.length > 0}
                <div class="flex items-center gap-1">
                    <span class="text-base-content/40">→</span>
                    <div class="flex gap-1">
                        {#each childLabels.slice(0, 3) as child}
                            <span class="badge badge-xs badge-outline">{child}</span>
                        {/each}
                        {#if childLabels.length > 3}
                            <span class="badge badge-xs badge-ghost">+{childLabels.length - 3}</span>
                        {/if}
                    </div>
                </div>
            {/if}
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center gap-2 pt-1">
            <button
                on:click={handlePreview}
                class="btn btn-sm btn-ghost gap-1.5 text-base-content/70 hover:text-primary hover:bg-primary/10"
            >
                <span class="material-symbols-rounded text-base">visibility</span>
                Preview
            </button>
            <button
                on:click={handleOpenInGallery}
                class="btn btn-sm btn-primary gap-1.5"
            >
                <span class="material-symbols-rounded text-base">folder_open</span>
                Open in Gallery
            </button>
        </div>
    </div>
</div>

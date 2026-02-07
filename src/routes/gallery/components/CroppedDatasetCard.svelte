<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import { IconButton } from "$lib/components/ui";

    export let tempPath: string;
    export let exportedPath: string | undefined = undefined;
    export let imageCount: number = 0;
    export let parentLabel: string = "";
    export let childLabels: string[] = [];
    export let createdAt: Date = new Date();

    const dispatch = createEventDispatcher<{
        preview: { tempPath: string };
        openInGallery: { tempPath: string };
        remove: { tempPath: string };
        export: { tempPath: string; destPath: string };
    }>();

    let exporting = false;

    function formatPath(path: string): string {
        // Show only the last 2 segments of the path
        const segments = path.split(/[/\\]/);
        if (segments.length <= 2) return path;
        return "..." + segments.slice(-2).join("/");
    }

    function formatTime(date: Date): string {
        return date.toLocaleTimeString([], {
            hour: "2-digit",
            minute: "2-digit",
        });
    }

    function handlePreview() {
        dispatch("preview", { tempPath });
    }

    function handleOpenInGallery() {
        dispatch("openInGallery", { tempPath });
    }

    function handleRemove() {
        dispatch("remove", { tempPath });
    }

    async function handleExport() {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select Export Destination",
            });
            if (selected && typeof selected === "string") {
                exporting = true;
                dispatch("export", { tempPath, destPath: selected });
            }
        } catch (err) {
            console.error("Error selecting directory:", err);
        } finally {
            exporting = false;
        }
    }

    $: isExported = !!exportedPath;
    $: displayPath = exportedPath || tempPath;
</script>

<div
    class="card bg-base-100 border border-base-200 shadow-sm hover:shadow-md transition-shadow"
>
    <div class="card-body p-4 gap-3">
        <!-- Header Row -->
        <div class="flex items-start justify-between gap-2">
            <div class="flex items-center gap-2 min-w-0">
                <div
                    class="w-8 h-8 rounded-lg {isExported
                        ? 'bg-success/10'
                        : 'bg-warning/10'} flex items-center justify-center flex-shrink-0"
                >
                    <span
                        class="material-symbols-rounded {isExported
                            ? 'text-success'
                            : 'text-warning'} text-lg"
                    >
                        {isExported ? "check_circle" : "schedule"}
                    </span>
                </div>
                <div class="min-w-0">
                    <div class="flex items-center gap-2">
                        <h4
                            class="font-bold text-sm text-base-content truncate"
                            title={displayPath}
                        >
                            {formatPath(displayPath)}
                        </h4>
                        {#if !isExported}
                            <span class="badge badge-warning badge-xs"
                                >Temp</span
                            >
                        {/if}
                    </div>
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
                <span class="material-symbols-rounded text-base text-primary"
                    >photo_library</span
                >
                <span class="font-bold">{imageCount}</span>
                <span class="text-base-content/50">images</span>
            </div>
            <div class="flex items-center gap-1.5">
                <span class="material-symbols-rounded text-base text-secondary"
                    >label</span
                >
                <span class="font-medium text-base-content/70"
                    >{parentLabel}</span
                >
            </div>
            {#if childLabels.length > 0}
                <div class="flex items-center gap-1">
                    <span class="text-base-content/40">→</span>
                    <div class="flex gap-1">
                        {#each childLabels.slice(0, 3) as child}
                            <span class="badge badge-xs badge-outline"
                                >{child}</span
                            >
                        {/each}
                        {#if childLabels.length > 3}
                            <span class="badge badge-xs badge-ghost"
                                >+{childLabels.length - 3}</span
                            >
                        {/if}
                    </div>
                </div>
            {/if}
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center gap-2 pt-1">
            <IconButton
                icon="visibility"
                label="Preview"
                tooltip="Preview cropped images"
                variant="ghost"
                size="sm"
                on:click={handlePreview}
            />
            <IconButton
                icon="folder_open"
                label="Open in Gallery"
                tooltip="View in gallery"
                variant="ghost"
                size="sm"
                on:click={handleOpenInGallery}
            />
            {#if !isExported}
                <IconButton
                    icon="download"
                    label="Export"
                    tooltip="Export to permanent location"
                    variant="soft"
                    size="sm"
                    loading={exporting}
                    on:click={handleExport}
                />
            {:else}
                <span class="text-xs text-success flex items-center gap-1">
                    <span class="material-symbols-rounded text-sm">check</span>
                    Exported
                </span>
            {/if}
        </div>
    </div>
</div>

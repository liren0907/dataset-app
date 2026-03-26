<script lang="ts">
    import { IconButton, Badge } from "$lib/components/ui";

    let {
        tempPath,
        exportedPath = undefined,
        imageCount = 0,
        parentLabel = "",
        childLabels = [],
        createdAt = new Date(),
        onpreview,
        onopeningallery,
        onremove,
        onexport,
    }: {
        tempPath: string;
        exportedPath?: string | undefined;
        imageCount?: number;
        parentLabel?: string;
        childLabels?: string[];
        createdAt?: Date;
        onpreview?: (data: { tempPath: string }) => void;
        onopeningallery?: (data: { tempPath: string }) => void;
        onremove?: (data: { tempPath: string }) => void;
        onexport?: (data: { tempPath: string; destPath: string }) => void;
    } = $props();

    let exporting = $state(false);

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
        onpreview?.({ tempPath });
    }

    function handleOpenInGallery() {
        onopeningallery?.({ tempPath });
    }

    function handleRemove() {
        onremove?.({ tempPath });
    }

    function handleExportClick() {
        onexport?.({ tempPath, destPath: "" });
    }

    let isExported = $derived(!!exportedPath);
    let displayPath = $derived(exportedPath || tempPath);
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
                            <Badge variant="warning" size="xs">Temp</Badge>
                        {/if}
                    </div>
                    <p class="text-xs text-base-content/50">
                        Created at {formatTime(createdAt)}
                    </p>
                </div>
            </div>
            <button
                onclick={handleRemove}
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
                    <span class="text-base-content/40">-></span>
                    <div class="flex gap-1">
                        {#each childLabels.slice(0, 3) as child}
                            <Badge variant="outline" size="xs">{child}</Badge>
                        {/each}
                        {#if childLabels.length > 3}
                            <Badge variant="ghost" size="xs">+{childLabels.length - 3}</Badge>
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
                variant="soft"
                size="sm"
                onclick={handlePreview}
            />
            <IconButton
                icon="folder_open"
                label="Open in Gallery"
                tooltip="View in gallery"
                variant="soft"
                size="sm"
                onclick={handleOpenInGallery}
            />
            {#if !isExported}
                <IconButton
                    icon="download"
                    label="Export"
                    tooltip="Export to permanent location"
                    variant="soft"
                    size="sm"
                    loading={exporting}
                    onclick={handleExportClick}
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

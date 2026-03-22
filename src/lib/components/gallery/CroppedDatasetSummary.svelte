<script lang="ts">
    let {
        outputPath = "",
        parentLabel = "",
        childLabels = [],
        imageCount = 0,
        isActive = false,
        createdAt = new Date(),
        onswitch,
        onexport,
        onpreview,
        onremove,
    }: {
        outputPath?: string;
        parentLabel?: string;
        childLabels?: string[];
        imageCount?: number;
        isActive?: boolean;
        createdAt?: Date;
        onswitch?: () => void;
        onexport?: () => void;
        onpreview?: () => void;
        onremove?: () => void;
    } = $props();

    function formatDate(date: Date): string {
        return new Intl.DateTimeFormat("en-US", {
            month: "short",
            day: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        }).format(date);
    }

    function getDirectoryName(path: string): string {
        return path.split("/").pop() || path;
    }
</script>

<!-- Cropped Dataset Summary Bar -->
<div
    class="group relative overflow-hidden rounded-xl border transition-all duration-200
           {isActive
        ? 'bg-primary/5 border-primary shadow-md shadow-primary/10'
        : 'bg-base-100 border-base-200 hover:border-primary/30 hover:shadow-sm'}"
>
    <!-- Active indicator -->
    {#if isActive}
        <div class="absolute left-0 top-0 bottom-0 w-1 bg-primary"></div>
    {/if}

    <div class="p-4 pl-5">
        <div class="flex items-center justify-between gap-4">
            <!-- Left: Info -->
            <div class="flex items-center gap-4 min-w-0 flex-1">
                <!-- Icon -->
                <div
                    class="flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center
                           {isActive
                        ? 'bg-primary text-primary-content'
                        : 'bg-base-200 text-base-content/60'}"
                >
                    <span class="material-symbols-rounded">crop</span>
                </div>

                <!-- Details -->
                <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2 mb-1">
                        <h4 class="font-semibold text-base-content truncate">
                            {getDirectoryName(outputPath)}
                        </h4>
                        {#if isActive}
                            <span class="badge badge-primary badge-sm"
                                >Active</span
                            >
                        {/if}
                    </div>
                    <div
                        class="flex items-center gap-3 text-sm text-base-content/60"
                    >
                        <span class="flex items-center gap-1">
                            <span class="material-symbols-rounded text-sm"
                                >photo_library</span
                            >
                            {imageCount} images
                        </span>
                        <span class="flex items-center gap-1">
                            <span class="material-symbols-rounded text-sm"
                                >label</span
                            >
                            {parentLabel}
                        </span>
                        <span class="hidden sm:flex items-center gap-1">
                            <span class="material-symbols-rounded text-sm"
                                >schedule</span
                            >
                            {formatDate(createdAt)}
                        </span>
                    </div>
                </div>
            </div>

            <!-- Right: Actions -->
            <div class="flex items-center gap-2">
                <!-- Child labels preview -->
                <div class="hidden md:flex items-center gap-1">
                    {#each childLabels.slice(0, 3) as label}
                        <span class="badge badge-sm badge-ghost">{label}</span>
                    {/each}
                    {#if childLabels.length > 3}
                        <span class="badge badge-sm badge-ghost"
                            >+{childLabels.length - 3}</span
                        >
                    {/if}
                </div>

                <div
                    class="divider divider-horizontal mx-1 h-6 hidden md:flex"
                ></div>

                <!-- Switch Button -->
                <button
                    class="btn btn-sm gap-2 {isActive
                        ? 'btn-primary'
                        : 'btn-ghost border border-base-300'}"
                    onclick={() => onswitch?.()}
                >
                    <span class="material-symbols-rounded text-sm">
                        {isActive ? "check_circle" : "swap_horiz"}
                    </span>
                    {isActive ? "Viewing" : "Switch"}
                </button>

                <!-- Export Button -->
                <button
                    class="btn btn-sm btn-ghost border border-base-300 gap-2"
                    onclick={() => onexport?.()}
                >
                    <span class="material-symbols-rounded text-sm"
                        >ios_share</span
                    >
                    Export
                </button>

                <!-- Preview Button -->
                <button
                    class="btn btn-sm btn-ghost btn-square"
                    onclick={() => onpreview?.()}
                    title="Preview images"
                >
                    <span class="material-symbols-rounded">visibility</span>
                </button>

                <!-- Remove Button -->
                <button
                    class="btn btn-sm btn-ghost btn-square text-error/60 hover:text-error hover:bg-error/10"
                    onclick={() => onremove?.()}
                    title="Remove from list"
                >
                    <span class="material-symbols-rounded">close</span>
                </button>
            </div>
        </div>
    </div>
</div>

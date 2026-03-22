<script lang="ts">
    import SectionLabel from "$lib/components/ui/SectionLabel.svelte";
    import {
        imageStatusStore,
        STATUS_CONFIG,
        type ImageStatus,
    } from "$lib/stores/imageStatusStore";

    let {
        images = [],
        selectedIndex = -1,
        onselect,
    }: {
        images?: { name: string; path: string; hasJson: boolean }[];
        selectedIndex?: number;
        onselect?: (index: number) => void;
    } = $props();

    // Filtering state
    let statusFilter = $state<ImageStatus | "all">("all");

    let filteredImages = $derived(
        images
            .map((img, originalIndex) => ({ ...img, originalIndex }))
            .filter((img) => {
                if (statusFilter !== "all") {
                    const status = $imageStatusStore.get(img.path) || "todo";
                    if (status !== statusFilter) return false;
                }
                return true;
            }),
    );

    function cycleStatus(path: string, event: MouseEvent | KeyboardEvent) {
        event.stopPropagation();
        const current = $imageStatusStore.get(path) || "todo";
        const order: ImageStatus[] = [
            "todo",
            "in_progress",
            "done",
            "needs_review",
        ];
        const nextIdx = (order.indexOf(current) + 1) % order.length;
        imageStatusStore.setStatus(path, order[nextIdx]);
    }

    function getStatusConfig(path: string) {
        const status = $imageStatusStore.get(path) || "todo";
        return STATUS_CONFIG[status];
    }

    function setStatusFilter(key: string) {
        statusFilter = key as ImageStatus | "all";
    }
</script>

<div
    class="flex w-56 flex-col border-r border-base-200 bg-base-100 shrink-0 overflow-hidden"
>
    <!-- Header -->
    <div class="p-3 border-b border-base-200 space-y-2">
        <SectionLabel
            >Images ({filteredImages.length}/{images.length})</SectionLabel
        >

        <!-- Status filter pills -->
        <div class="flex flex-wrap gap-1">
            <button
                class="badge badge-xs cursor-pointer {statusFilter === 'all'
                    ? 'badge-primary'
                    : 'badge-ghost'}"
                onclick={() => (statusFilter = "all")}>All</button
            >
            {#each Object.entries(STATUS_CONFIG) as [key, cfg]}
                <button
                    class="badge badge-xs cursor-pointer"
                    class:badge-ghost={statusFilter !== key}
                    style={statusFilter === key
                        ? `background-color: ${cfg.color}; color: white; border-color: ${cfg.color};`
                        : ""}
                    onclick={() => setStatusFilter(key)}
                    title={cfg.label}
                >
                    {cfg.label}
                </button>
            {/each}
        </div>
    </div>

    <!-- Image list -->
    <div class="flex-1 overflow-y-auto">
        {#each filteredImages as image (image.path)}
            {@const sc = getStatusConfig(image.path)}
            <button
                class="w-full text-left px-3 py-2 flex items-center gap-2 text-xs transition-colors duration-150
                    {image.originalIndex === selectedIndex
                    ? 'bg-primary/10 text-primary border-r-2 border-primary'
                    : 'hover:bg-base-200 text-base-content'}"
                onclick={() => onselect?.(image.originalIndex)}
            >
                <!-- Status dot (click to cycle) -->
                <span
                    role="button"
                    tabindex="0"
                    class="w-3 h-3 rounded-full shrink-0 border border-base-300 transition-colors hover:scale-125 cursor-pointer"
                    style="background-color: {sc.color};"
                    title="Status: {sc.label} (click to change)"
                    onclick={(e) => cycleStatus(image.path, e)}
                    onkeydown={(e) => e.key === 'Enter' && cycleStatus(image.path, e)}
                ></span>

                <span class="truncate flex-1">{image.name}</span>

                {#if image.hasJson}
                    <span
                        class="badge badge-xs badge-success shrink-0"
                        title="Has LabelMe JSON">JSON</span
                    >
                {/if}
            </button>
        {/each}

        {#if filteredImages.length === 0}
            <p class="text-xs text-base-content/40 text-center py-4">
                No images match filter
            </p>
        {/if}
    </div>
</div>

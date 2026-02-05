<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { safeConvertFileSrc } from "../utils/tauriUtils";

    // Props
    export let selectedImage: any = null;

    // Event dispatcher
    const dispatch = createEventDispatcher();

    // State
    let scale = 1;
    let translateX = 0;
    let translateY = 0;
    let isDragging = false;
    let dragStartX = 0;
    let dragStartY = 0;
    let imageLoaded = false;
    let imageError = false;

    // Constants
    const MIN_SCALE = 0.1;
    const MAX_SCALE = 5;
    const ZOOM_STEP = 0.25;

    // Computed
    $: imageUrl =
        selectedImage?.previewUrl ||
        safeConvertFileSrc(selectedImage?.path || "");
    $: zoomPercentage = Math.round(scale * 100);

    // Reset state when image changes
    $: if (selectedImage) {
        resetView();
        imageLoaded = false;
        imageError = false;
    }

    // Methods
    function zoomIn() {
        scale = Math.min(scale + ZOOM_STEP, MAX_SCALE);
    }

    function zoomOut() {
        scale = Math.max(scale - ZOOM_STEP, MIN_SCALE);
    }

    function resetZoom() {
        scale = 1;
        translateX = 0;
        translateY = 0;
    }

    function fitToScreen() {
        // Reset to fit - simple approach
        scale = 1;
        translateX = 0;
        translateY = 0;
    }

    function resetView() {
        scale = 1;
        translateX = 0;
        translateY = 0;
    }

    function handleClose() {
        dispatch("close");
    }

    // Drag handlers
    function handleMouseDown(event: MouseEvent) {
        if (event.button !== 0) return; // Only left click
        isDragging = true;
        dragStartX = event.clientX - translateX;
        dragStartY = event.clientY - translateY;
        event.preventDefault();
    }

    function handleMouseMove(event: MouseEvent) {
        if (!isDragging) return;
        translateX = event.clientX - dragStartX;
        translateY = event.clientY - dragStartY;
    }

    function handleMouseUp() {
        isDragging = false;
    }

    function handleMouseLeave() {
        isDragging = false;
    }

    // Wheel zoom
    function handleWheel(event: WheelEvent) {
        event.preventDefault();
        const delta = event.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP;
        scale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, scale + delta));
    }

    function handleImageLoad() {
        imageLoaded = true;
        imageError = false;
    }

    function handleImageError() {
        imageLoaded = false;
        imageError = true;
    }
</script>

<div class="flex flex-col h-full bg-base-100 preview-panel">
    <!-- Header -->
    <div
        class="flex items-center justify-between p-3 border-b border-base-300 bg-base-200/50"
    >
        <h3
            class="font-semibold text-sm truncate pr-2 flex-1"
            title={selectedImage?.name}
        >
            {selectedImage?.name || "Preview"}
        </h3>
        <button
            on:click={handleClose}
            class="btn btn-sm btn-ghost btn-square"
            aria-label="Close panel"
        >
            <span class="material-symbols-rounded text-lg">close</span>
        </button>
    </div>

    <!-- Toolbar -->
    <div
        class="flex items-center justify-between gap-2 px-3 py-2 border-b border-base-300 bg-base-100"
    >
        <div class="join">
            <button
                on:click={zoomOut}
                class="join-item btn btn-xs btn-ghost"
                title="Zoom Out"
                disabled={scale <= MIN_SCALE}
            >
                <span class="material-symbols-rounded text-sm">remove</span>
            </button>
            <button
                on:click={resetZoom}
                class="join-item btn btn-xs btn-ghost min-w-[50px]"
                title="Reset Zoom"
            >
                <span class="text-xs font-medium">{zoomPercentage}%</span>
            </button>
            <button
                on:click={zoomIn}
                class="join-item btn btn-xs btn-ghost"
                title="Zoom In"
                disabled={scale >= MAX_SCALE}
            >
                <span class="material-symbols-rounded text-sm">add</span>
            </button>
        </div>
        <button
            on:click={fitToScreen}
            class="btn btn-xs btn-ghost gap-1"
            title="Fit to Screen"
        >
            <span class="material-symbols-rounded text-sm">fit_screen</span>
        </button>
    </div>

    <!-- Image Preview Area -->
    <div
        class="flex-1 relative overflow-hidden bg-base-200 cursor-grab"
        class:cursor-grabbing={isDragging}
        role="img"
        aria-label="Image preview area"
        on:mousedown={handleMouseDown}
        on:mousemove={handleMouseMove}
        on:mouseup={handleMouseUp}
        on:mouseleave={handleMouseLeave}
        on:wheel={handleWheel}
    >
        {#if !imageLoaded && !imageError}
            <div class="absolute inset-0 flex items-center justify-center">
                <span class="loading loading-spinner loading-lg text-primary"
                ></span>
            </div>
        {/if}

        {#if imageError}
            <div
                class="absolute inset-0 flex flex-col items-center justify-center text-base-content/50"
            >
                <span class="material-symbols-rounded text-4xl mb-2"
                    >broken_image</span
                >
                <span class="text-sm">Failed to load image</span>
            </div>
        {/if}

        {#if selectedImage}
            <div class="absolute inset-0 flex items-center justify-center">
                <img
                    src={imageUrl}
                    alt={selectedImage.name || "Preview"}
                    class="max-w-none select-none transition-transform duration-100"
                    class:opacity-0={!imageLoaded}
                    class:opacity-100={imageLoaded}
                    style="transform: scale({scale}) translate({translateX /
                        scale}px, {translateY / scale}px);"
                    draggable="false"
                    on:load={handleImageLoad}
                    on:error={handleImageError}
                />
            </div>
        {/if}
    </div>

    <!-- Footer Info -->
    <div
        class="flex items-center justify-between px-3 py-2 border-t border-base-300 bg-base-100 text-xs text-base-content/60"
    >
        <div class="flex items-center gap-3">
            {#if selectedImage?.dimensions}
                <span
                    >{selectedImage.dimensions.width} × {selectedImage
                        .dimensions.height}</span
                >
            {/if}
            {#if selectedImage?.size}
                <span>{(selectedImage.size / 1024).toFixed(1)} KB</span>
            {/if}
        </div>
        {#if selectedImage?.annotated}
            <div class="badge badge-success badge-sm gap-1">
                <span class="material-symbols-rounded text-xs">check</span>
                Annotated
            </div>
        {/if}
    </div>
</div>

<style>
    .preview-panel {
        height: 100%;
        max-height: 100%;
        display: flex;
        flex-direction: column;
    }

    .cursor-grab {
        cursor: grab;
    }

    .cursor-grabbing {
        cursor: grabbing;
    }
</style>

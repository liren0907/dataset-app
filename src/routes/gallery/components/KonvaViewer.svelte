<script lang="ts">
    import { onMount, onDestroy, createEventDispatcher } from 'svelte';
    import { createKonvaManager, type KonvaManager, type KonvaImageData, type KonvaAnnotation } from './konvaService';

    // Props
    export let showModal: boolean = false;
    export let imageData: KonvaImageData | null = null;

    // Event dispatcher for communication with parent
    const dispatch = createEventDispatcher();

    // KonvaJS manager instance
    let konvaManager: KonvaManager;
    let konvaContainer: HTMLDivElement;
    let isInitialized = false;

    // Reactive statements
    $: if (showModal && imageData && !isInitialized) {
        initializeViewer();
    }

    $: if (!showModal && isInitialized) {
        cleanupViewer();
    }

    onMount(() => {
        konvaManager = createKonvaManager();
    });

    onDestroy(() => {
        if (konvaManager) {
            konvaManager.cleanup();
        }
    });

    // Initialize the Konva viewer
    async function initializeViewer(): Promise<void> {
        if (!imageData || !konvaContainer) return;

        try {
            console.log('Initializing Konva viewer for image:', imageData.name);

            // Calculate optimal stage dimensions
            const rect = konvaContainer.getBoundingClientRect();
            const stageWidth = Math.max(rect.width || 1000, 1000);
            const stageHeight = Math.max(rect.height || 700, 700);

            // Initialize the stage
            konvaManager.initializeStage(konvaContainer, stageWidth, stageHeight);

            // Load image and annotations
            await konvaManager.loadImageWithAnnotations(imageData, (scale, offsetX, offsetY) => {
                konvaManager.drawAnnotations(imageData.annotations, scale, offsetX, offsetY);
            });

            isInitialized = true;
            console.log('Konva viewer initialized successfully');
        } catch (error) {
            console.error('Failed to initialize Konva viewer:', error);
        }
    }

    // Cleanup the viewer
    function cleanupViewer(): void {
        if (konvaManager) {
            konvaManager.cleanup();
        }
        isInitialized = false;
    }

    // Close modal handler
    function handleClose(): void {
        dispatch('close');
    }

    // Keyboard event handler
    function handleKeydown(event: KeyboardEvent): void {
        if (!konvaManager) return;

        // Prevent default behavior for handled keys
        const handledKeys = ['Delete', 'Backspace', 'a', 'A', 'Escape', '=', '+', '-', '_', '0', 'r', 'R'];

        if (handledKeys.includes(event.key) || (event.ctrlKey && event.key.toLowerCase() === 'a')) {
            event.preventDefault();
        }

        switch (event.key.toLowerCase()) {
            case 'escape':
                handleClose();
                break;
            case 'delete':
            case 'backspace':
                konvaManager.deleteSelectedAnnotation();
                break;
            case 'a':
                if (event.ctrlKey || event.metaKey) {
                    konvaManager.selectAllAnnotations();
                }
                break;
            case '=':
            case '+':
                konvaManager.zoomIn();
                break;
            case '-':
            case '_':
                konvaManager.zoomOut();
                break;
            case '0':
                konvaManager.resetZoom();
                break;
            case 'r':
                konvaManager.fitToScreen();
                break;
        }
    }

    // Zoom control handlers
    function handleZoomIn(): void {
        konvaManager?.zoomIn();
    }

    function handleZoomOut(): void {
        konvaManager?.zoomOut();
    }

    function handleResetZoom(): void {
        konvaManager?.resetZoom();
    }

    function handleFitToScreen(): void {
        konvaManager?.fitToScreen();
    }

    // Annotation control handlers
    function handleSelectAll(): void {
        konvaManager?.selectAllAnnotations();
    }

    function handleDeselect(): void {
        konvaManager?.deselectAnnotation();
    }

    function handleDeleteSelected(): void {
        konvaManager?.deleteSelectedAnnotation();
    }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if showModal && imageData}
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
        role="dialog"
        aria-modal="true"
        aria-labelledby="konva-viewer-title"
        on:click={handleClose}
        on:keydown={(e) => {
            if (e.key === 'Escape') handleClose();
        }}
        tabindex="-1"
    >
        <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
        <div
            class="bg-white rounded-lg shadow-xl max-w-6xl max-h-[95vh] overflow-hidden"
            role="document"
            on:click|stopPropagation
            on:keydown|stopPropagation
        >
            <!-- Header -->
            <div class="p-4 border-b border-gray-200 flex justify-between items-center">
                <h3 id="konva-viewer-title" class="text-lg font-medium text-gray-900">{imageData.name}</h3>
                <button
                    on:click={handleClose}
                    class="text-gray-400 hover:text-gray-600 text-2xl leading-none focus:outline-none focus:ring-2 focus:ring-gray-500 rounded"
                    aria-label="Close viewer"
                >
                    ×
                </button>
            </div>

            <!-- Content -->
            <div class="p-4">
                <!-- Control Panel -->
                <div class="flex flex-wrap items-center gap-2 mb-4 p-3 bg-gray-50 rounded-lg">
                    <div class="text-sm text-gray-600 mr-4">Controls:</div>

                    <!-- Zoom Controls -->
                    <button
                        on:click={handleZoomOut}
                        class="px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white text-sm rounded transition-colors"
                        title="Zoom Out (-)"
                        aria-label="Zoom out"
                    >
                        🔍-
                    </button>
                    <button
                        on:click={handleResetZoom}
                        class="px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white text-sm rounded transition-colors"
                        title="Reset Zoom (0)"
                        aria-label="Reset zoom"
                    >
                        100%
                    </button>
                    <button
                        on:click={handleZoomIn}
                        class="px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white text-sm rounded transition-colors"
                        title="Zoom In (=)"
                        aria-label="Zoom in"
                    >
                        🔍+
                    </button>

                    <!-- Fit to Screen -->
                    <button
                        on:click={handleFitToScreen}
                        class="px-3 py-1 bg-green-500 hover:bg-green-600 text-white text-sm rounded transition-colors ml-4"
                        title="Fit to Screen (R)"
                        aria-label="Fit to screen"
                    >
                        📐 Fit
                    </button>

                    <!-- Annotation Controls -->
                    <div class="ml-4 flex items-center gap-2">
                        <span class="text-sm text-gray-600">Annotations:</span>
                        <button
                            on:click={handleSelectAll}
                            class="px-3 py-1 bg-purple-500 hover:bg-purple-600 text-white text-sm rounded transition-colors"
                            title="Select All (Ctrl+A)"
                            aria-label="Select all annotations"
                        >
                            Select All
                        </button>
                        <button
                            on:click={handleDeselect}
                            class="px-3 py-1 bg-gray-500 hover:bg-gray-600 text-white text-sm rounded transition-colors"
                            title="Deselect (Esc)"
                            aria-label="Deselect annotations"
                        >
                            Deselect
                        </button>
                        <button
                            on:click={handleDeleteSelected}
                            class="px-3 py-1 bg-red-500 hover:bg-red-600 text-white text-sm rounded transition-colors"
                            title="Delete Selected (Del)"
                            aria-label="Delete selected annotations"
                        >
                            🗑️ Delete
                        </button>
                    </div>

                    <!-- Status -->
                    <div class="ml-auto text-sm text-gray-600">
                        {#if konvaManager}
                            Zoom: {konvaManager.getZoomPercentage()}% |
                            Annotations: {konvaManager.getAnnotationCount()}
                            {#if konvaManager.getSelectedCount() > 0}
                                | Selected: {konvaManager.getSelectedCount()}
                            {/if}
                        {/if}
                    </div>
                </div>

                <!-- Keyboard Shortcuts Info -->
                <div class="text-xs text-gray-500 mb-4 bg-blue-50 p-2 rounded">
                    <strong>Keyboard Shortcuts:</strong> Zoom (+/-), Reset (0), Fit (R), Select All (Ctrl+A), Delete (Del), Deselect (Esc), Close (Esc)
                </div>

                <!-- KonvaJS Canvas Container -->
                <div class="relative max-w-full max-h-[70vh] overflow-hidden rounded-lg bg-gray-100">
                    <div
                        bind:this={konvaContainer}
                        class="w-full h-full min-h-[600px] border-2 border-gray-300 bg-gray-50 flex items-center justify-center"
                        style="width: 1000px; height: 700px;"
                        role="img"
                        aria-label="Interactive annotation viewer for {imageData.name}"
                    >
                        {#if !isInitialized}
                            <div class="text-gray-500">Loading advanced canvas...</div>
                        {/if}
                    </div>

                    <!-- Status Indicators -->
                    {#if isInitialized}
                        <div class="absolute top-2 left-2 bg-green-600 text-white text-xs px-2 py-1 rounded shadow z-10">
                            Enhanced KonvaJS
                        </div>
                        <div class="absolute bottom-2 right-2 bg-blue-600 text-white text-xs px-2 py-1 rounded shadow z-10">
                            {konvaManager?.getAnnotationCount() || 0} annotations loaded
                        </div>
                    {/if}
                </div>

                <!-- Footer Info -->
                <div class="mt-4 text-sm text-gray-600 text-center">
                    Advanced interactive annotations with zoom, pan, selection, and editing capabilities
                </div>
            </div>
        </div>
    </div>
{/if}

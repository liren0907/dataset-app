<script lang="ts">
    import { onMount, onDestroy, createEventDispatcher } from 'svelte';
    import { createKonvaManager, type KonvaManager, type KonvaImageData } from './konvaService';

    // Props
    export let selectedImage: any = null; // ProcessedImage from parent
    export let currentAnnotations: any[] = []; // Current annotations for editing

    // Event dispatcher for communication with parent
    const dispatch = createEventDispatcher();

    // KonvaJS manager instance
    let konvaManager: KonvaManager;
    let konvaContainer: HTMLDivElement;
    let isInitialized = false;

    // Local state for editing
    let editedAnnotations: any[] = [];
    let originalAnnotations: any[] = [];

    // Initialize with props
    $: if (currentAnnotations) {
        editedAnnotations = [...currentAnnotations];
        originalAnnotations = [...currentAnnotations];
    }

    // Sync with prop changes
    $: if (selectedImage && currentAnnotations && selectedImage.previewUrl) {
        editedAnnotations = [...currentAnnotations];
        originalAnnotations = [...currentAnnotations];
        console.log('AnnotationEditor: Props updated', {
            imageName: selectedImage.name,
            annotationsCount: currentAnnotations.length,
            hasPreviewUrl: !!selectedImage.previewUrl
        });
        if (isInitialized) {
            updateAnnotations();
        }
    }

    onMount(() => {
        konvaManager = createKonvaManager();
    });

    onDestroy(() => {
        if (konvaManager) {
            konvaManager.cleanup();
        }
    });

    // Reactive: Initialize when image is selected
    $: if (selectedImage && !isInitialized) {
        initializeEditor();
    }

    $: if (!selectedImage && isInitialized) {
        cleanupEditor();
    }

    // Initialize the annotation editor
    async function initializeEditor(): Promise<void> {
        if (!selectedImage || !konvaContainer) {
            console.error('Cannot initialize editor: missing selectedImage or konvaContainer');
            return;
        }

        // Check if image has required properties
        if (!selectedImage.previewUrl) {
            console.error('Cannot initialize editor: selectedImage missing previewUrl', {
                imageName: selectedImage.name,
                imagePath: selectedImage.path,
                hasPreviewUrl: !!selectedImage.previewUrl
            });
            return;
        }

        try {
            console.log('Initializing annotation editor for image:', selectedImage.name);
            console.log('Image properties:', {
                name: selectedImage.name,
                path: selectedImage.path,
                previewUrl: selectedImage.previewUrl ? 'present' : 'missing',
                annotations: editedAnnotations?.length || 0
            });

            // Calculate optimal stage dimensions
            const rect = konvaContainer.getBoundingClientRect();
            const stageWidth = Math.max(rect.width || 1000, 1000);
            const stageHeight = Math.max(rect.height || 600, 600);

            // Initialize the stage
            konvaManager.initializeStage(konvaContainer, stageWidth, stageHeight);

            // Prepare image data for KonvaJS
            const konvaImageData: KonvaImageData = {
                id: selectedImage.path,
                path: selectedImage.path,
                previewUrl: selectedImage.previewUrl,
                name: selectedImage.name,
                annotations: editedAnnotations
            };

            console.log('Loading image with KonvaJS:', konvaImageData.previewUrl);

            // Load image and annotations
            await konvaManager.loadImageWithAnnotations(konvaImageData, (scale, offsetX, offsetY) => {
                console.log('Drawing annotations with scale:', scale);
                konvaManager.drawAnnotations(editedAnnotations, scale, offsetX, offsetY);
            });

            isInitialized = true;
            console.log('Annotation editor initialized successfully');
        } catch (error) {
            console.error('Failed to initialize annotation editor:', error);
            // Show error state to user
            isInitialized = false;
        }
    }

    // Update annotations when they change
    function updateAnnotations(): void {
        if (!konvaManager || !isInitialized) return;

        // Update the annotations in the Konva manager
        // This would need to be implemented based on how we track changes
        const scale = konvaManager.getState().scale;
        const offsetX = konvaManager.getState().stageX;
        const offsetY = konvaManager.getState().stageY;

        konvaManager.drawAnnotations(editedAnnotations, scale, -offsetX, -offsetY);
    }

    // Cleanup the editor
    function cleanupEditor(): void {
        if (konvaManager) {
            konvaManager.cleanup();
        }
        isInitialized = false;
    }

    // Handle save
    function handleSave(): void {
        // Get current annotations from Konva manager
        // For now, we'll assume the annotations are updated in the editedAnnotations array
        dispatch('save', {
            image: selectedImage,
            annotations: editedAnnotations
        });
    }

    // Handle cancel
    function handleCancel(): void {
        // Reset to original annotations
        editedAnnotations = [...originalAnnotations];
        dispatch('cancel');
    }

    // Handle close/back to gallery
    function handleClose(): void {
        dispatch('close');
    }

    // Keyboard shortcuts
    function handleKeydown(event: KeyboardEvent): void {
        if (!konvaManager) return;

        // Handle keyboard shortcuts for the editor
        switch (event.key.toLowerCase()) {
            case 'escape':
                handleCancel();
                break;
            case 's':
                if (event.ctrlKey || event.metaKey) {
                    event.preventDefault();
                    handleSave();
                }
                break;
        }

        // Pass other keyboard events to Konva manager
        const handledKeys = ['Delete', 'Backspace', 'a', 'A', '=', '+', '-', '_', '0', 'r', 'R'];

        if (handledKeys.includes(event.key) || (event.ctrlKey && event.key.toLowerCase() === 'a')) {
            event.preventDefault();

            switch (event.key.toLowerCase()) {
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

{#if selectedImage}
<div class="annotation-editor-panel bg-white/95 backdrop-blur rounded-xl shadow-xl border border-slate-200/60 overflow-hidden h-full flex flex-col">
    <!-- Header -->
    <div class="flex justify-between items-center p-4 border-b bg-slate-50">
        <div class="flex items-center gap-3">
            <button
                on:click={handleClose}
                class="text-slate-600 hover:text-slate-800 p-2 -ml-2 rounded-md hover:bg-slate-200 transition-colors"
                aria-label="Back to gallery"
            >
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18" />
                </svg>
            </button>
            <div>
                <h2 class="text-lg font-semibold text-slate-800">Annotation Editor</h2>
                <p class="text-sm text-slate-600">{selectedImage.name}</p>
            </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center gap-2">
            <button
                on:click={handleCancel}
                class="px-4 py-2 border border-slate-300 rounded-md text-slate-700 hover:bg-slate-50 transition-colors"
            >
                Cancel
            </button>
            <button
                on:click={handleSave}
                class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md transition-colors flex items-center gap-2"
            >
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                </svg>
                Save Changes
            </button>
        </div>
    </div>

    <!-- Editor Content -->
    <div class="flex-1 flex flex-col p-4 min-h-0">
        <!-- Control Panel -->
        <div class="flex flex-wrap items-center gap-2 mb-4 p-3 bg-slate-50 rounded-lg border">
            <div class="text-sm text-slate-600 mr-4">Tools:</div>

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
                <span class="text-sm text-slate-600">Annotations:</span>
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
                    class="px-3 py-1 bg-slate-500 hover:bg-slate-600 text-white text-sm rounded transition-colors"
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
            <div class="ml-auto text-sm text-slate-600">
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
        <div class="text-xs text-slate-500 mb-4 bg-blue-50 p-2 rounded">
            <strong>Keyboard Shortcuts:</strong> Zoom (+/-), Reset (0), Fit (R), Select All (Ctrl+A), Delete (Del), Cancel (Esc), Save (Ctrl+S)
        </div>

        <!-- KonvaJS Canvas Container -->
        <div class="flex-1 relative bg-slate-100 rounded-lg overflow-hidden border-2 border-slate-300 min-h-[400px]">
            <div
                bind:this={konvaContainer}
                class="w-full h-full bg-slate-50 flex items-center justify-center"
                style="width: 100%; height: 100%;"
                role="img"
                aria-label="Interactive annotation editor for {selectedImage.name}"
            >
                {#if !isInitialized}
                    {#if !selectedImage?.previewUrl}
                        <!-- Error state: Missing image URL -->
                        <div class="text-center p-8">
                            <div class="text-red-500 text-4xl mb-4">⚠️</div>
                            <h3 class="text-lg font-semibold text-slate-800 mb-2">Image Not Available</h3>
                            <p class="text-slate-600 mb-4">The selected image cannot be loaded for editing.</p>
                            <div class="text-sm text-slate-500 bg-slate-100 p-3 rounded">
                                <strong>Debug Info:</strong><br>
                                Name: {selectedImage?.name || 'Unknown'}<br>
                                Path: {selectedImage?.path || 'Unknown'}<br>
                                Preview URL: {selectedImage?.previewUrl ? 'Present' : 'Missing'}
                            </div>
                        </div>
                    {:else}
                        <!-- Loading state -->
                        <div class="text-center">
                            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
                            <div class="text-slate-500">Loading annotation editor...</div>
                            <div class="text-sm text-slate-400 mt-2">Initializing canvas and loading image</div>
                            <div class="text-xs text-slate-400 mt-1">This may take longer for high-resolution images</div>
                        </div>
                    {/if}
                {:else}
                    <!-- Successfully loaded - show canvas (KonvaJS will render here) -->
                    <div class="opacity-0">Canvas loaded - KonvaJS active</div>
                {/if}
            </div>

            <!-- Status Indicators -->
            {#if isInitialized}
                <div class="absolute top-2 left-2 bg-green-600 text-white text-xs px-2 py-1 rounded shadow z-10">
                    Annotation Editor Active
                </div>
                <div class="absolute bottom-2 right-2 bg-blue-600 text-white text-xs px-2 py-1 rounded shadow z-10">
                    {konvaManager?.getAnnotationCount() || 0} annotations
                </div>
            {/if}
        </div>

        <!-- Instructions -->
        <div class="mt-4 text-sm text-slate-600 text-center bg-slate-50 p-3 rounded">
            Use the tools above to edit annotations. Click and drag to select, use Delete key to remove, or click Save when done.
        </div>
    </div>
</div>
{/if}

<style>
    .annotation-editor-panel {
        max-width: 100%;
        margin: 0 auto;
    }
</style>

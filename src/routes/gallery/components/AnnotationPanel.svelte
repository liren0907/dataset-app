<script lang="ts">
    import { onMount, onDestroy, createEventDispatcher, tick } from "svelte";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { invoke } from "@tauri-apps/api/core";
    import {
        createKonvaManager,
        type KonvaManager,
        type KonvaImageData,
    } from "./konvaService";

    // Props
    export let selectedImage: any = null; // ProcessedImage from parent

    // Event dispatcher
    const dispatch = createEventDispatcher();

    // State variables
    let konvaManager: KonvaManager;
    let konvaContainer: HTMLDivElement;
    let isInitialized = false;
    let isLoading = false;
    let isInitializing = false;
    let annotatedImageData: KonvaImageData | null = null;
    let tempPreviewPath: string | null = null;

    // React to selectedImage changes
    $: if (selectedImage && !isInitializing) {
        console.log("📸 Image changed in panel:", selectedImage.path);
        // Force re-initialization when image changes
        cleanupAndReinit();
    }

    async function cleanupAndReinit() {
        if (konvaManager) {
            konvaManager.cleanup(); // Clear previous stage
            isInitialized = false;
        }
        await tick();
        if (selectedImage) {
            initializePanel();
        }
    }

    onMount(() => {
        konvaManager = createKonvaManager();
        if (selectedImage) {
            initializePanel();
        }
    });

    onDestroy(() => {
        if (konvaManager) {
            konvaManager.cleanup();
        }
        annotatedImageData = null;
    });

    // Parse annotations from LabelMe format to Konva format
    function parseAnnotationsFromLabelMe(metadata: any): any[] {
        const shapes = metadata.shapes || [];
        return shapes.map((shape: any) => ({
            label: shape.label || "",
            shape_type: shape.shape_type || "rectangle",
            points: shape.points || [],
        }));
    }

    async function initializePanel(): Promise<void> {
        if (!selectedImage || isInitializing) return;

        isInitializing = true;
        isLoading = true;

        try {
            await tick(); // Wait for DOM

            // Step 1: Load metadata
            await loadAnnotationMetadata();

            // Step 2: Initialize Konva
            if (annotatedImageData && konvaContainer) {
                const stageWidth = konvaContainer.clientWidth || 400;
                const stageHeight = 400; // Fixed height for sidebar preview or dynamic?
                // Let's make it responsive to container

                konvaManager.initializeStage(
                    konvaContainer,
                    stageWidth,
                    stageHeight,
                );

                await konvaManager.loadImageWithAnnotations(
                    annotatedImageData,
                    (scale, offsetX, offsetY) => {
                        konvaManager.drawAnnotations(
                            annotatedImageData!.annotations,
                            scale,
                            offsetX,
                            offsetY,
                        );
                    },
                );

                isInitialized = true;
            }
        } catch (error) {
            console.error("❌ Panel initialization failed:", error);
            // Fallback could be added here similar to modal
        } finally {
            isLoading = false;
            isInitializing = false;
        }
    }

    async function loadAnnotationMetadata(): Promise<void> {
        if (!selectedImage?.path) return;
        try {
            const result = (await invoke("generate_single_annotated_preview", {
                imagePath: selectedImage.path,
            })) as string;

            const data = JSON.parse(result);
            if (data.annotation_metadata) {
                annotatedImageData = {
                    id: `panel_${selectedImage.path}`,
                    path: selectedImage.path,
                    previewUrl:
                        selectedImage.previewUrl ||
                        convertFileSrc(selectedImage.path),
                    name: selectedImage.name,
                    annotations: parseAnnotationsFromLabelMe(
                        data.annotation_metadata,
                    ),
                };
            }
        } catch (error) {
            console.error("Failed to load metadata", error);
            throw error;
        }
    }

    // Event handlers
    function handleClose(): void {
        dispatch("close");
    }

    function handleSave(): void {
        if (annotatedImageData) {
            dispatch("save", {
                image: selectedImage,
                annotations: annotatedImageData.annotations,
            });
        }
    }

    // Keyboard shortcuts (simplified for panel)
    function handleKeydown(event: KeyboardEvent): void {
        if (!konvaManager || !isInitialized) return;
        // Only handle if panel likely has focus or specific keys
        // (Implementation can be refined)
    }

    // Tools
    function handleZoomIn() {
        konvaManager?.zoomIn();
    }
    function handleZoomOut() {
        konvaManager?.zoomOut();
    }
    function handleResetZoom() {
        konvaManager?.resetZoom();
    }
    function handleFitToScreen() {
        konvaManager?.fitToScreen();
    }
</script>

<div class="flex flex-col h-full bg-base-100 image-panel">
    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b border-base-300">
        <h3 class="font-bold text-lg truncate pr-2" title={selectedImage?.name}>
            {selectedImage?.name || "Image Details"}
        </h3>
        <button
            on:click={handleClose}
            class="btn btn-sm btn-ghost btn-square"
            aria-label="Close panel"
        >
            <span class="material-symbols-rounded">close</span>
        </button>
    </div>

    <!-- Toolbar -->
    <div
        class="flex items-center gap-1 p-2 border-b border-base-300 bg-base-200/50 overflow-x-auto"
    >
        <div class="join">
            <button
                on:click={handleZoomOut}
                class="join-item btn btn-xs btn-ghost"
                title="Zoom Out"
            >
                <span class="material-symbols-rounded text-sm">remove</span>
            </button>
            <button
                on:click={handleResetZoom}
                class="join-item btn btn-xs btn-ghost"
                title="Reset"
            >
                <span class="text-xs">100%</span>
            </button>
            <button
                on:click={handleZoomIn}
                class="join-item btn btn-xs btn-ghost"
                title="Zoom In"
            >
                <span class="material-symbols-rounded text-sm">add</span>
            </button>
        </div>
        <div class="divider divider-horizontal mx-1"></div>
        <button
            on:click={handleFitToScreen}
            class="btn btn-xs btn-ghost"
            title="Fit to Screen"
        >
            <span class="material-symbols-rounded text-sm">fit_screen</span>
        </button>
    </div>

    <!-- Canvas Area -->
    <div
        class="flex-1 relative overflow-hidden bg-base-200"
        role="region"
        aria-label="Annotation Viewer"
    >
        {#if isLoading}
            <div class="absolute inset-0 flex items-center justify-center">
                <span class="loading loading-spinner loading-lg"></span>
            </div>
        {/if}

        <div
            bind:this={konvaContainer}
            class="w-full h-full"
            style="visibility: {isLoading ? 'hidden' : 'visible'}"
        ></div>
    </div>

    <!-- Footer Actions -->
    <div class="p-4 border-t border-base-300 bg-base-100">
        <button on:click={handleSave} class="btn btn-primary w-full gap-2">
            <span class="material-symbols-rounded">save</span>
            Save Changes
        </button>
    </div>
</div>

<style>
    .image-panel {
        height: 100%;
        max-height: 100%;
        display: flex;
        flex-direction: column;
    }
</style>

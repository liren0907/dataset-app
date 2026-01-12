<script lang="ts">
    import { onMount, onDestroy, createEventDispatcher, tick } from "svelte";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { invoke } from "@tauri-apps/api/core";
    import {
        createKonvaManager,
        type KonvaManager,
        type KonvaImageData,
        type KonvaAnnotation,
    } from "./konvaService";

    // Props
    export let showModal: boolean = false;
    export let selectedImage: any = null; // ProcessedImage from parent
    export let autoAnnotationEnabled: boolean = true;

    // Event dispatcher for communication with parent
    const dispatch = createEventDispatcher();

    // State variables
    let konvaManager: KonvaManager;
    let konvaContainer: HTMLDivElement;
    let isInitialized = false;
    let isLoading = false;
    let isInitializing = false; // ← NEW: Prevent duplicate initialization
    let annotatedImageData: KonvaImageData | null = null;
    let tempPreviewPath: string | null = null;

    // Debug: Track konvaContainer binding
    $: if (konvaContainer && showModal) {
        console.log("🔗 Konva container bound:", konvaContainer);
        console.log(
            "📏 Container dimensions:",
            konvaContainer.offsetWidth,
            "x",
            konvaContainer.offsetHeight,
        );
        console.log("📍 Container in DOM:", document.contains(konvaContainer));
    }

    // Force container discovery when modal opens
    $: if (showModal && !konvaContainer) {
        console.log(
            "🔍 Modal opened but konvaContainer not bound, attempting manual discovery...",
        );
        // Small delay to ensure DOM is ready
        setTimeout(() => {
            const foundContainer = document.querySelector(
                '[aria-label*="Interactive annotation editor"]',
            ) as HTMLDivElement;
            if (foundContainer) {
                console.log(
                    "🎯 Manual discovery successful, setting konvaContainer",
                );
                konvaContainer = foundContainer;
            } else {
                console.log("❌ Manual discovery failed");
            }
        }, 100);
    }

    // Manual trigger approach to avoid reactive statement issues
    let hasTriggeredInit = false;

    // Watch for modal opening - allow re-initialization if previous failed
    $: {
        if (showModal && selectedImage && !isInitializing) {
            console.log(
                "🔄 Modal opened with image, checking initialization state",
            );
            console.log(
                "State: isInitialized:",
                isInitialized,
                "hasTriggeredInit:",
                hasTriggeredInit,
                "isInitializing:",
                isInitializing,
            );

            // Allow initialization if:
            // 1. Never initialized before (!hasTriggeredInit)
            // 2. Previous initialization failed (hasTriggeredInit but !isInitialized)
            if (!hasTriggeredInit || (hasTriggeredInit && !isInitialized)) {
                console.log("🚀 Triggering modal initialization");
                hasTriggeredInit = true;

                // Always use delayed initialization to ensure DOM is ready
                // The modal needs time to fully render before Konva can attach
                console.log(
                    "⏳ Using delayed initialization to ensure DOM readiness",
                );
                setTimeout(() => {
                    // Double-check modal is still open before initializing
                    if (showModal && selectedImage) {
                        initializeModal();
                    }
                }, 100); // Reduced from 500ms to 100ms for better UX
            } else if (hasTriggeredInit && isInitialized) {
                console.log(
                    "⚠️ Skipping re-initialization - already successfully initialized",
                );
            }
        }
    }

    // Watch for modal closing - reset all state
    $: if (!showModal) {
        console.log("🔄 Modal closed, cleaning up");
        cleanupModal();
    }

    onMount(() => {
        konvaManager = createKonvaManager();
    });

    onDestroy(() => {
        cleanupModal();
        if (konvaManager) {
            konvaManager.cleanup();
        }
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

    // Robust DOM readiness checking with retry mechanism
    async function ensureKonvaContainerReady(
        maxRetries: number = 15,
        baseDelay: number = 100,
    ): Promise<void> {
        console.log("🔍 Starting DOM readiness check for Konva container...");

        for (let attempt = 1; attempt <= maxRetries; attempt++) {
            // Wait for Svelte to update DOM
            await tick();

            // Additional wait for DOM to stabilize
            await new Promise((resolve) => setTimeout(resolve, 50));

            // First, try to find the element directly using querySelector as fallback
            let container = konvaContainer;
            if (!container) {
                console.log(
                    `🔍 Attempt ${attempt}: Trying to find container with querySelector...`,
                );
                container = document.querySelector(
                    '[aria-label*="Interactive annotation editor"]',
                ) as HTMLDivElement;
                if (container) {
                    console.log(
                        `✅ Found container via querySelector on attempt ${attempt}`,
                    );
                    konvaContainer = container; // Update the bound variable
                }
            }

            // Check if container is available
            if (container) {
                console.log(
                    `🔍 Attempt ${attempt}: Container exists, checking DOM attachment...`,
                );

                // Additional verification: check if element is in document
                if (document.contains(container)) {
                    console.log(
                        `🔍 Attempt ${attempt}: Container is in DOM, checking dimensions...`,
                    );

                    // Final check: ensure element has dimensions (fully rendered)
                    const rect = container.getBoundingClientRect();
                    if (rect.width > 0 && rect.height > 0) {
                        console.log(
                            `✅ Konva container ready on attempt ${attempt} (${rect.width}x${rect.height})`,
                        );
                        return;
                    } else {
                        console.log(
                            `⏳ Container exists but has zero dimensions (${rect.width}x${rect.height})`,
                        );
                    }
                } else {
                    console.log(`⏳ Container exists but not attached to DOM`);
                }
            } else {
                console.log(`⏳ Container is null/undefined`);
            }

            // Progressive delay with cap
            const delay = Math.min(baseDelay * attempt, 500); // Cap at 500ms
            console.log(
                `⏳ Waiting ${delay}ms before next attempt (${attempt}/${maxRetries})...`,
            );

            await new Promise((resolve) => setTimeout(resolve, delay));

            // Additional tick after delay
            await tick();
        }

        // FINAL FALLBACK: Try to find and set container manually
        console.log(
            "🔄 Final fallback: Attempting manual container discovery...",
        );
        const manualContainer = document.querySelector(
            '[aria-label*="Interactive annotation editor"]',
        ) as HTMLDivElement;

        if (manualContainer) {
            console.log("✅ Manual container discovery successful!");
            konvaContainer = manualContainer;

            const rect = manualContainer.getBoundingClientRect();
            if (rect.width > 0 && rect.height > 0) {
                console.log(
                    `🎉 Final fallback successful! Container ready (${rect.width}x${rect.height})`,
                );
                return;
            } else {
                console.log(
                    "⚠️ Manual container found but has zero dimensions, will still proceed",
                );
                return;
            }
        }

        // If we get here, all retries failed
        console.error("❌ All attempts to find Konva container failed");
        console.error("📊 Debug info:", {
            showModal,
            selectedImage: !!selectedImage,
            konvaContainer: !!konvaContainer,
            documentContains: konvaContainer
                ? document.contains(konvaContainer)
                : false,
            querySelectorFound: !!document.querySelector(
                '[aria-label*="Interactive annotation editor"]',
            ),
            dimensions: konvaContainer
                ? (() => {
                      const rect = konvaContainer.getBoundingClientRect();
                      return `${rect.width}x${rect.height}`;
                  })()
                : "N/A",
        });

        throw new Error(
            `Konva container not ready after ${maxRetries} attempts. This usually indicates a DOM timing issue.`,
        );
    }

    // Initialize the modal with backend-preprocessed image
    async function initializeModal(): Promise<void> {
        if (!selectedImage) return;

        // Multiple guards to prevent any duplicate calls
        if (isInitializing || (hasTriggeredInit && isInitialized)) {
            console.log(
                "⚠️ Initialization blocked - isInitializing:",
                isInitializing,
                "isInitialized:",
                isInitialized,
                "hasTriggeredInit:",
                hasTriggeredInit,
            );
            return;
        }

        isInitializing = true; // ← NEW: Prevent duplicate calls
        console.log(
            "🚀 STARTING modal initialization for:",
            selectedImage.name,
        );

        // DEBUG: Log initialization state
        console.log("🔧 DEBUG: Initialization state:", {
            selectedImage: !!selectedImage,
            konvaContainer: !!konvaContainer,
            konvaManager: !!konvaManager,
            isLoading,
            isInitialized,
            isInitializing,
        });

        // Ensure konvaManager is ready
        if (!konvaManager) {
            throw new Error(
                "KonvaManager not initialized - check konvaService import",
            );
        }

        try {
            isLoading = true;
            console.log(
                "🚀 Initializing modal annotation viewer for:",
                selectedImage.name,
            );

            // Step 0: Ensure DOM is ready with robust checking
            await ensureKonvaContainerReady();

            console.log("✅ Konva container is ready:", konvaContainer);

            // Step 1: Load annotation metadata from backend
            await loadAnnotationMetadata();

            // Step 2: Initialize KonvaJS with original image and parsed annotations
            if (annotatedImageData && konvaContainer) {
                await initializeKonvaViewer();
            } else {
                throw new Error(
                    "Missing required data for KonvaJS initialization",
                );
            }
        } catch (error) {
            console.error("❌ Failed to initialize modal:", error);
            console.log("🔄 Attempting fallback initialization...");
            // Fallback to basic image display
            await fallbackInitialization();
        } finally {
            isLoading = false;
            isInitializing = false; // ← NEW: Reset flag
            // Only reset hasTriggeredInit if initialization actually succeeded
            if (isInitialized) {
                // Keep hasTriggeredInit = true to prevent re-initialization of successful state
                console.log(
                    "✅ MODAL INITIALIZATION COMPLETED for:",
                    selectedImage?.name,
                );
            } else {
                // Reset hasTriggeredInit if initialization failed, allowing retry
                hasTriggeredInit = false;
                console.log(
                    "❌ MODAL INITIALIZATION FAILED for:",
                    selectedImage?.name,
                );
            }
        }
    }

    // Load annotation metadata from backend and parse for Konva
    async function loadAnnotationMetadata(): Promise<void> {
        if (!selectedImage?.path) return;

        try {
            console.log("🔧 Loading annotation metadata from backend...");
            console.log("📂 Image path:", selectedImage.path);

            // BACKEND METADATA LOADING: The Rust backend provides:
            // 1. Loading the JSON annotation file for the image
            // 2. Returning complete annotation metadata in LabelMe format
            // 3. Frontend parses and converts to Konva annotation format
            // 4. Original image is used with parsed annotations for drawing

            console.log(
                "🚀 Invoking Tauri command: generate_single_annotated_preview",
            );
            const result = (await invoke("generate_single_annotated_preview", {
                imagePath: selectedImage.path,
            })) as string;
            console.log(
                "✅ Tauri command completed, result length:",
                result.length,
            );

            const data = JSON.parse(result);

            if (data.annotation_metadata) {
                // Parse annotations from LabelMe format
                const parsedAnnotations = parseAnnotationsFromLabelMe(
                    data.annotation_metadata,
                );

                // Use original image with parsed annotations
                annotatedImageData = {
                    id: `modal_${selectedImage.path}`,
                    path: selectedImage.path,
                    previewUrl:
                        selectedImage.previewUrl ||
                        convertFileSrc(selectedImage.path),
                    name: selectedImage.name,
                    annotations: parsedAnnotations,
                };

                console.log(
                    "✅ Backend metadata loaded:",
                    parsedAnnotations.length,
                    "annotations from LabelMe format",
                );
            } else {
                throw new Error("No annotation metadata returned from backend");
            }
        } catch (error) {
            console.warn(
                "⚠️ Backend preview generation failed, using fallback:",
                error,
            );
            throw error; // Will trigger fallback
        }
    }

    // Initialize KonvaJS viewer with pre-annotated image
    async function initializeKonvaViewer(): Promise<void> {
        if (!konvaContainer || !annotatedImageData) return;

        console.log("🎨 Initializing KonvaJS viewer...");

        // Calculate optimal stage dimensions (fixed for performance)
        const stageWidth = 1000;
        const stageHeight = 700;

        // Initialize the stage
        konvaManager.initializeStage(konvaContainer, stageWidth, stageHeight);

        // DEBUG: Log before KonvaJS operations
        console.log("🎨 DEBUG: About to initialize KonvaJS");
        console.log("Annotated image data:", !!annotatedImageData);
        console.log(
            "Konva container dimensions:",
            konvaContainer?.clientWidth,
            "x",
            konvaContainer?.clientHeight,
        );

        try {
            // Load original image with parsed annotations from backend metadata
            await konvaManager.loadImageWithAnnotations(
                annotatedImageData,
                (scale, offsetX, offsetY) => {
                    console.log("📋 Drawing annotations from backend metadata");
                    konvaManager.drawAnnotations(
                        annotatedImageData!.annotations,
                        scale,
                        offsetX,
                        offsetY,
                    );
                },
            );

            isInitialized = true;
            console.log(
                "✅ Modal annotation viewer initialized successfully - isInitialized set to true",
            );
        } catch (konvaError) {
            console.error("❌ KonvaJS initialization failed:", konvaError);
            throw new Error(`KonvaJS setup failed: ${konvaError.message}`);
        }
    }

    // Fallback initialization for when backend preprocessing fails
    async function fallbackInitialization(): Promise<void> {
        if (!selectedImage) return;

        console.log("🔄 Using fallback initialization...");

        try {
            // Use the same robust checking mechanism
            await ensureKonvaContainerReady(8, 100); // Fewer retries, longer base delay for fallback
            console.log(
                "✅ Fallback: Konva container is ready:",
                konvaContainer,
            );
        } catch (error) {
            console.error("❌ Fallback failed:", error);
            return;
        }

        try {
            const stageWidth = 1000;
            const stageHeight = 700;

            // DEBUG: Log fallback state
            console.log("🔄 Fallback: Initializing KonvaJS stage");
            console.log(
                "Container dimensions:",
                konvaContainer?.clientWidth,
                "x",
                konvaContainer?.clientHeight,
            );

            konvaManager.initializeStage(
                konvaContainer,
                stageWidth,
                stageHeight,
            );

            // Convert selectedImage to KonvaImageData format
            const fallbackImageData: KonvaImageData = {
                id: `fallback_${selectedImage.path}`,
                path: selectedImage.path,
                previewUrl:
                    selectedImage.previewUrl ||
                    convertFileSrc(selectedImage.path),
                name: selectedImage.name,
                annotations: selectedImage.annotations || [],
            };

            console.log(
                "🔄 Fallback: Loading image with annotations:",
                fallbackImageData.annotations.length,
            );

            // Load image with progressive annotation rendering
            await konvaManager.loadImageWithAnnotations(
                fallbackImageData,
                (scale, offsetX, offsetY) => {
                    console.log("📋 Drawing annotations progressively");
                    konvaManager.drawAnnotations(
                        fallbackImageData.annotations,
                        scale,
                        offsetX,
                        offsetY,
                    );
                },
            );

            isInitialized = true;
            console.log("✅ Fallback initialization completed successfully");
        } catch (fallbackError) {
            console.error(
                "❌ Fallback initialization also failed:",
                fallbackError,
            );
            console.error("Full error details:", fallbackError);
            // Reset hasTriggeredInit to allow retry if user tries again
            hasTriggeredInit = false;
            // At this point, both main and fallback have failed
            // The modal will show the error state
        }
    }

    // Cleanup modal resources
    function cleanupModal(): void {
        if (konvaManager) {
            konvaManager.cleanup();
        }

        // Clean up temporary preview file
        if (tempPreviewPath) {
            // Note: In a real implementation, you might want to clean up temp files
            // For now, we'll let the OS handle cleanup
            tempPreviewPath = null;
        }

        // Reset all state flags for clean slate
        isInitialized = false;
        isInitializing = false; // ← Reset initialization flag
        hasTriggeredInit = false; // ← Reset trigger flag (IMPORTANT: allows re-init)
        annotatedImageData = null;
        isLoading = false;
        console.log(
            "🧹 Modal cleanup completed - all flags reset, ready for next open",
        );
    }

    // Event handlers
    function handleClose(): void {
        dispatch("close");
    }

    function handleSave(): void {
        // Emit save event with current annotations
        if (annotatedImageData) {
            dispatch("save", {
                image: selectedImage,
                annotations: annotatedImageData.annotations,
            });
        }
    }

    function handleKeydown(event: KeyboardEvent): void {
        if (!konvaManager) return;

        // Handle keyboard shortcuts
        const handledKeys = [
            "Delete",
            "Backspace",
            "a",
            "A",
            "Escape",
            "=",
            "+",
            "-",
            "_",
            "0",
            "r",
            "R",
            "s",
            "S",
        ];

        if (
            handledKeys.includes(event.key) ||
            (event.ctrlKey && event.key.toLowerCase() === "a") ||
            (event.ctrlKey && event.key.toLowerCase() === "s")
        ) {
            event.preventDefault();
        }

        switch (event.key.toLowerCase()) {
            case "escape":
                handleClose();
                break;
            case "s":
                if (event.ctrlKey || event.metaKey) {
                    handleSave();
                }
                break;
            case "delete":
            case "backspace":
                konvaManager.deleteSelectedAnnotation();
                break;
            case "a":
                if (event.ctrlKey || event.metaKey) {
                    konvaManager.selectAllAnnotations();
                }
                break;
            case "=":
            case "+":
                konvaManager.zoomIn();
                break;
            case "-":
            case "_":
                konvaManager.zoomOut();
                break;
            case "0":
                konvaManager.resetZoom();
                break;
            case "r":
                konvaManager.fitToScreen();
                break;
        }
    }

    // Control handlers
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

{#if showModal}
    <!-- Modal Overlay -->
    {console.log(
        "🎨 Modal HTML is rendering, showModal:",
        showModal,
        "selectedImage:",
        !!selectedImage,
    )}
    <dialog
        class="modal modal-open"
        role="dialog"
        aria-modal="true"
        aria-labelledby="annotation-modal-title"
        aria-describedby="annotation-modal-description"
    >
        <!-- Modal Content -->
        <div
            class="modal-box max-w-6xl w-full max-h-[95vh] flex flex-col p-0"
            role="document"
            aria-label="Annotation editor content"
            on:click|stopPropagation
            on:keydown|stopPropagation
        >
            <!-- Header -->
            <div
                class="flex justify-between items-center p-4 border-b border-base-300 bg-base-200"
            >
                <div class="flex items-center gap-3">
                    <button
                        on:click={handleClose}
                        class="btn btn-ghost btn-sm btn-circle"
                        aria-label="Close annotation viewer"
                    >
                        <span class="material-symbols-rounded">arrow_back</span>
                    </button>
                    <div>
                        <h3
                            id="annotation-modal-title"
                            class="text-lg font-semibold text-base-content"
                        >
                            Annotation Editor
                        </h3>
                        <p
                            id="annotation-modal-description"
                            class="text-sm text-base-content/60"
                        >
                            {selectedImage?.name || "Image"}
                        </p>
                    </div>
                </div>

                <!-- Action Buttons -->
                <div class="flex items-center gap-2">
                    <button on:click={handleClose} class="btn btn-ghost btn-sm">
                        Cancel
                    </button>
                    <button
                        on:click={handleSave}
                        class="btn btn-ghost border border-neutral btn-sm"
                    >
                        <span class="material-symbols-rounded text-lg"
                            >check</span
                        >
                        Save Changes
                    </button>
                </div>
            </div>

            <!-- Content Area -->
            <div class="flex-1 flex flex-col p-4 min-h-0">
                {#if isInitialized}
                    <!-- Control Panel -->
                    <div
                        class="flex flex-wrap items-center gap-2 mb-4 p-3 bg-base-200 rounded-lg border border-base-300"
                    >
                        <div class="text-sm text-base-content/70 mr-4">
                            Tools:
                        </div>

                        <!-- Zoom Controls -->
                        <div class="join">
                            <button
                                on:click={handleZoomOut}
                                class="join-item btn btn-sm btn-ghost border border-neutral"
                                title="Zoom Out (-)"
                                aria-label="Zoom out"
                            >
                                🔍-
                            </button>
                            <button
                                on:click={handleResetZoom}
                                class="join-item btn btn-sm btn-ghost border border-neutral"
                                title="Reset Zoom (0)"
                                aria-label="Reset zoom"
                            >
                                100%
                            </button>
                            <button
                                on:click={handleZoomIn}
                                class="join-item btn btn-sm btn-ghost border border-neutral"
                                title="Zoom In (=)"
                                aria-label="Zoom in"
                            >
                                🔍+
                            </button>
                        </div>

                        <!-- Fit to Screen -->
                        <button
                            on:click={handleFitToScreen}
                            class="btn btn-sm btn-ghost border border-neutral ml-2"
                            title="Fit to Screen (R)"
                            aria-label="Fit to screen"
                        >
                            📐 Fit
                        </button>

                        <!-- Annotation Controls -->
                        <div class="ml-4 flex items-center gap-2">
                            <span class="text-sm text-base-content/70"
                                >Annotations:</span
                            >
                            <button
                                on:click={handleSelectAll}
                                class="btn btn-sm btn-ghost border border-neutral"
                                title="Select All (Ctrl+A)"
                                aria-label="Select all annotations"
                            >
                                Select All
                            </button>
                            <button
                                on:click={handleDeselect}
                                class="btn btn-sm btn-ghost"
                                title="Deselect (Esc)"
                                aria-label="Deselect annotations"
                            >
                                Deselect
                            </button>
                            <button
                                on:click={handleDeleteSelected}
                                class="btn btn-sm btn-ghost border border-neutral"
                                title="Delete Selected (Del)"
                                aria-label="Delete selected annotations"
                            >
                                🗑️ Delete
                            </button>
                        </div>

                        <!-- Status -->
                        <div class="ml-auto text-sm text-base-content/70">
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
                    <div class="alert alert-info py-2 mb-4">
                        <span class="text-xs"
                            ><strong>Keyboard Shortcuts:</strong> Zoom (+/-), Reset
                            (0), Fit (R), Select All (Ctrl+A), Delete (Del), Save
                            (Ctrl+S), Cancel (Esc)</span
                        >
                    </div>

                    <!-- Konva container is now always present below, outside the conditional blocks -->
                {/if}

                <!-- Single Konva Container - Always Present -->
                <div
                    class="flex-1 relative bg-slate-100 rounded-lg overflow-hidden border-2 border-slate-300 min-h-[500px]"
                >
                    <div
                        bind:this={konvaContainer}
                        class="w-full h-full bg-slate-50"
                        style="width: 1000px; height: 700px; min-width: 1000px; min-height: 700px;"
                        role="img"
                        aria-label="Interactive annotation editor for {selectedImage?.name ||
                            'image'}"
                    >
                        <!-- KonvaJS will render here -->
                    </div>

                    <!-- Error Overlay -->
                    {#if !isLoading && !isInitialized}
                        <div
                            class="absolute inset-0 bg-slate-50/90 backdrop-blur-sm flex flex-col justify-center items-center z-20"
                        >
                            <div class="text-red-500 text-4xl mb-4">⚠️</div>
                            <h3
                                class="text-lg font-semibold text-slate-800 mb-2"
                            >
                                Failed to Load
                            </h3>
                            <p class="text-slate-600 mb-4">
                                Unable to initialize the annotation editor.
                            </p>
                            <button
                                on:click={handleClose}
                                class="px-4 py-2 bg-slate-600 hover:bg-slate-700 text-white rounded-md"
                            >
                                Close
                            </button>
                        </div>
                    {/if}

                    <!-- Loading Overlay -->
                    {#if isLoading}
                        <div
                            class="absolute inset-0 bg-slate-50/90 backdrop-blur-sm flex flex-col justify-center items-center z-20"
                        >
                            <div
                                class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mb-4"
                            ></div>
                            <p class="text-slate-600 text-lg mb-2">
                                Loading Annotation Editor...
                            </p>
                            <p class="text-sm text-slate-500">
                                {#if autoAnnotationEnabled}
                                    Loading annotation metadata from backend...
                                {:else}
                                    Preparing interactive canvas...
                                {/if}
                            </p>
                        </div>
                    {/if}

                    <!-- Status Indicators -->
                    {#if isInitialized}
                        <div
                            class="absolute top-2 left-2 bg-green-600 text-white text-xs px-2 py-1 rounded shadow z-10"
                        >
                            {#if autoAnnotationEnabled}
                                Backend Metadata
                            {:else}
                                Live Processing
                            {/if}
                        </div>
                        <div
                            class="absolute bottom-2 right-2 bg-blue-600 text-white text-xs px-2 py-1 rounded shadow z-10"
                        >
                            {konvaManager?.getAnnotationCount() || 0} annotations
                        </div>
                    {/if}
                </div>

                <!-- Instructions -->
                {#if isInitialized}
                    <div
                        class="mt-4 text-sm text-base-content/70 text-center bg-base-200 p-3 rounded-lg"
                    >
                        Use the tools above to edit annotations. Click and drag
                        to select, use Delete key to remove, or click Save when
                        done.
                    </div>
                {/if}
            </div>
        </div>
        <form method="dialog" class="modal-backdrop">
            <button on:click={handleClose}>close</button>
        </form>
    </dialog>
{/if}

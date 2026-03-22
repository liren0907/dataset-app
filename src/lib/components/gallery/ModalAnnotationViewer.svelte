<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { safeConvertFileSrc } from "$lib/utils/tauriUtils";
    import { invoke } from "@tauri-apps/api/core";
    import {
        createKonvaManager,
        type KonvaManager,
        type KonvaImageData,
        type KonvaAnnotation,
    } from "$lib/services/gallery/konvaService";
    import { mockGeneratePreview } from "$lib/../mocks/mockFileSystem";

    let {
        showModal = false,
        selectedImage = null,
        autoAnnotationEnabled = true,
        isMockMode = false,
        onclose,
        onsave,
    }: {
        showModal?: boolean;
        selectedImage?: any;
        autoAnnotationEnabled?: boolean;
        isMockMode?: boolean;
        onclose?: () => void;
        onsave?: (data: { image: any; annotations: any[] }) => void;
    } = $props();

    // State variables
    let konvaManager: KonvaManager;
    let konvaContainer: HTMLDivElement;
    let isInitialized = $state(false);
    let isLoading = $state(false);
    let isInitializing = $state(false);
    let annotatedImageData: KonvaImageData | null = $state(null);
    let tempPreviewPath: string | null = $state(null);
    let annotationCount = $state(0);

    // Helpers
    function updateMetadata(): void {
        if (konvaManager) {
            annotationCount = konvaManager.getAnnotationCount();
        } else {
            annotationCount = 0;
        }
    }

    // ResizeObserver for robust container detection
    let resizeObserver: ResizeObserver | null = null;

    // Debug: Track konvaContainer binding
    $effect(() => {
        if (konvaContainer && showModal) {
            console.log("Konva container bound:", konvaContainer);
            console.log(
                "Container dimensions:",
                konvaContainer.offsetWidth,
                "x",
                konvaContainer.offsetHeight,
            );
            console.log("Container in DOM:", document.contains(konvaContainer));
        }
    });

    // Force container discovery when modal opens
    $effect(() => {
        if (showModal && !konvaContainer) {
            console.log(
                "Modal opened but konvaContainer not bound, attempting manual discovery...",
            );
            setTimeout(() => {
                const foundContainer = document.querySelector(
                    '[aria-label*="Interactive annotation editor"]',
                ) as HTMLDivElement;
                if (foundContainer) {
                    console.log(
                        "Manual discovery successful, setting konvaContainer",
                    );
                    konvaContainer = foundContainer;
                } else {
                    console.log("Manual discovery failed");
                }
            }, 100);
        }
    });

    // Reactive: Trigger initialization when all conditions are met
    $effect(() => {
        if (
            showModal &&
            selectedImage &&
            konvaContainer &&
            !isInitialized &&
            !isInitializing
        ) {
            console.log("Triggering modal initialization (Reactive)");
            initializeModal();
        }
    });

    // Watch for modal closing - reset all state
    $effect(() => {
        if (!showModal) {
            if (isInitialized || isInitializing) {
                console.log("Modal closed, cleaning up");
                cleanupModal();
            }
        }
    });

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

    // Robust DOM readiness checking using ResizeObserver
    function waitForContainerDimensions(): Promise<void> {
        return new Promise((resolve, reject) => {
            if (!konvaContainer) {
                reject(new Error("Konva container not bound"));
                return;
            }

            if (
                konvaContainer.clientWidth > 0 &&
                konvaContainer.clientHeight > 0
            ) {
                console.log(
                    `Container ready immediately (${konvaContainer.clientWidth}x${konvaContainer.clientHeight})`,
                );
                resolve();
                return;
            }

            console.log(
                "Waiting for container dimensions via ResizeObserver...",
            );

            if (resizeObserver) resizeObserver.disconnect();

            resizeObserver = new ResizeObserver((entries) => {
                for (const entry of entries) {
                    const { width, height } = entry.contentRect;
                    if (width > 0 && height > 0) {
                        console.log(
                            `Container resized to ${width}x${height}`,
                        );
                        resizeObserver?.disconnect();
                        resizeObserver = null;
                        resolve();
                    }
                }
            });

            resizeObserver.observe(konvaContainer);

            setTimeout(() => {
                if (resizeObserver) {
                    resizeObserver.disconnect();
                    resizeObserver = null;
                    const width = konvaContainer?.clientWidth || 0;
                    const height = konvaContainer?.clientHeight || 0;
                    if (width > 0 && height > 0) {
                        resolve();
                    } else {
                        reject(
                            new Error(
                                "Timeout waiting for container dimensions",
                            ),
                        );
                    }
                }
            }, 5000);
        });
    }

    // Initialize the modal with backend-preprocessed image
    async function initializeModal(): Promise<void> {
        if (!selectedImage || isInitializing || isInitialized) return;

        isInitializing = true;
        console.log(
            "STARTING modal initialization for:",
            selectedImage.name,
        );

        if (!konvaManager) {
            isInitializing = false;
            return;
        }

        try {
            isLoading = true;

            await waitForContainerDimensions();

            console.log("Step 1: Loading annotation metadata...");
            await loadAnnotationMetadata();

            console.log("Step 2: Initializing Konva Viewer...");
            if (annotatedImageData && konvaContainer) {
                await initializeKonvaViewer();
            } else {
                throw new Error(
                    "Missing required data for KonvaJS initialization",
                );
            }
        } catch (error) {
            console.error("Failed to initialize modal:", error);
        } finally {
            isLoading = false;
            isInitializing = false;
        }
    }

    // Load annotation metadata from backend and parse for Konva
    async function loadAnnotationMetadata(): Promise<void> {
        if (!selectedImage?.path) return;

        try {
            console.log("Loading annotation metadata from backend...");
            console.log("Image path:", selectedImage.path);

            const isTauri =
                typeof window !== "undefined" && "__TAURI__" in window;
            let result: string;

            if (isTauri && !isMockMode) {
                console.log(
                    "Invoking Tauri command: generate_single_annotated_preview",
                );
                result = (await invoke("generate_single_annotated_preview", {
                    imagePath: selectedImage.path,
                })) as string;
            } else {
                console.log("Mode: Using Mock Preview Data");
                const startTime = Date.now();
                result = await mockGeneratePreview(selectedImage.path);
                console.log(
                    `mockGeneratePreview returned in ${Date.now() - startTime}ms`,
                );
            }

            console.log("Parsing result...");
            const data = JSON.parse(result);

            if (data.annotation_metadata) {
                const parsedAnnotations = parseAnnotationsFromLabelMe(
                    data.annotation_metadata,
                );

                annotatedImageData = {
                    id: `modal_${selectedImage.path}`,
                    path: selectedImage.path,
                    previewUrl:
                        selectedImage.previewUrl ||
                        safeConvertFileSrc(selectedImage.path),
                    name: selectedImage.name,
                    annotations: parsedAnnotations,
                };

                console.log(
                    "Backend metadata loaded:",
                    parsedAnnotations.length,
                    "annotations from LabelMe format",
                );
            } else {
                throw new Error("No annotation metadata returned from backend");
            }
        } catch (error) {
            console.warn(
                "Backend preview generation failed, using fallback:",
                error,
            );
            throw error;
        }
    }

    // Initialize KonvaJS viewer with pre-annotated image
    async function initializeKonvaViewer(): Promise<void> {
        if (!konvaContainer || !annotatedImageData) return;

        console.log("Initializing KonvaJS viewer...");

        const stageWidth = konvaContainer.clientWidth || 1000;
        const stageHeight = konvaContainer.clientHeight || 700;

        konvaManager.initializeStage(konvaContainer, stageWidth, stageHeight);

        console.log("DEBUG: About to initialize KonvaJS");
        console.log("Annotated image data:", !!annotatedImageData);
        console.log(
            "Konva container dimensions:",
            konvaContainer?.clientWidth,
            "x",
            konvaContainer?.clientHeight,
        );

        try {
            await konvaManager.loadImageWithAnnotations(
                annotatedImageData,
                (scale, offsetX, offsetY) => {
                    console.log("Drawing annotations from backend metadata");
                    konvaManager.drawAnnotations(
                        annotatedImageData!.annotations,
                        scale,
                        offsetX,
                        offsetY,
                    );
                    updateMetadata();
                },
            );

            isInitialized = true;
            updateMetadata();
            console.log(
                "Modal annotation viewer initialized successfully - isInitialized set to true",
            );
        } catch (konvaError) {
            console.error("KonvaJS initialization failed:", konvaError);
            const errorMessage =
                konvaError instanceof Error
                    ? konvaError.message
                    : String(konvaError);
            throw new Error(`KonvaJS setup failed: ${errorMessage}`);
        }
    }

    // Fallback initialization for when backend preprocessing fails
    async function fallbackInitialization(): Promise<void> {
        if (!selectedImage) return;

        console.log("Using fallback initialization...");

        try {
            await waitForContainerDimensions();
            console.log("Fallback: Konva container is ready");
        } catch (error) {
            console.error("Fallback failed:", error);
            return;
        }

        try {
            const stageWidth = konvaContainer.clientWidth || 1000;
            const stageHeight = konvaContainer.clientHeight || 700;

            console.log("Fallback: Initializing KonvaJS stage");
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

            const fallbackImageData: KonvaImageData = {
                id: `fallback_${selectedImage.path}`,
                path: selectedImage.path,
                previewUrl:
                    selectedImage.previewUrl ||
                    safeConvertFileSrc(selectedImage.path),
                name: selectedImage.name,
                annotations: selectedImage.annotations || [],
            };

            console.log(
                "Fallback: Loading image with annotations:",
                fallbackImageData.annotations.length,
            );

            await konvaManager.loadImageWithAnnotations(
                fallbackImageData,
                (scale, offsetX, offsetY) => {
                    console.log("Drawing annotations progressively");
                    konvaManager.drawAnnotations(
                        fallbackImageData.annotations,
                        scale,
                        offsetX,
                        offsetY,
                    );
                    updateMetadata();
                },
            );

            isInitialized = true;
            updateMetadata();
            console.log("Fallback initialization completed successfully");
        } catch (fallbackError) {
            console.error(
                "Fallback initialization also failed:",
                fallbackError,
            );
            isInitializing = false;
        }
    }

    // Cleanup modal resources
    function cleanupModal(): void {
        if (konvaManager) {
            konvaManager.cleanup();
        }

        if (resizeObserver) {
            resizeObserver.disconnect();
            resizeObserver = null;
        }

        if (tempPreviewPath) {
            tempPreviewPath = null;
        }

        isInitialized = false;
        isInitializing = false;
        annotatedImageData = null;
        isLoading = false;
        annotationCount = 0;
        console.log("Modal cleanup completed");
    }

    // Event handlers
    function handleClose(event?: Event): void {
        if (event) {
            event.stopPropagation();
            event.preventDefault();
        }

        console.log("Closing modal via handleClose");
        onclose?.();
    }

    function handleSave(): void {
        updateMetadata();
        if (annotatedImageData) {
            onsave?.({
                image: selectedImage,
                annotations: annotatedImageData.annotations,
            });
        }
    }

    function handleKeydown(event: KeyboardEvent): void {
        if (!konvaManager) return;

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
                updateMetadata();
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
        updateMetadata();
    }
    // Resize handler for responsiveness
    function handleResize() {
        if (konvaManager && isInitialized) {
            konvaManager.resize();
        }
    }
</script>

<svelte:window onkeydown={handleKeydown} onresize={handleResize} />

{#if showModal}
    <!-- Modal Overlay -->
    {console.log(
        "Modal HTML is rendering, showModal:",
        showModal,
        "selectedImage:",
        !!selectedImage,
    )}
    <dialog
        class="modal modal-open"
        aria-modal="true"
        aria-labelledby="annotation-modal-title"
        aria-describedby="annotation-modal-description"
    >
        <!-- Modal Content -->
        <div
            class="modal-box max-w-6xl w-full max-h-[95vh] flex flex-col p-0"
            role="document"
            aria-label="Annotation editor content"
        >
            <!-- Header -->
            <div
                class="flex justify-between items-center p-4 border-b border-base-300 bg-base-200"
            >
                <div class="flex items-center gap-3">
                    <button
                        onclick={(e) => handleClose(e)}
                        class="btn btn-ghost btn-sm btn-circle"
                        aria-label="Close modal"
                    >
                        <span class="material-symbols-rounded">close</span>
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

                <div class="flex items-center gap-2">
                    {#if autoAnnotationEnabled}
                        <div class="badge badge-success gap-1 shadow-sm">
                            <span class="material-symbols-rounded text-xs"
                                >dns</span
                            >
                            Backend Data
                        </div>
                    {:else}
                        <div class="badge badge-info gap-1 shadow-sm">
                            <span class="material-symbols-rounded text-xs"
                                >computer</span
                            >
                            Live Mode
                        </div>
                    {/if}
                </div>
            </div>

            <!-- Content Area -->
            <div class="flex-1 flex flex-col p-4 min-h-0">
                {#if isInitialized}
                    <!-- Control Panel (Modern Toolbar matched to Gallery Navbar) -->
                    <div
                        class="navbar bg-base-100 min-h-0 h-14 border border-base-200 shadow-sm rounded-lg px-3 gap-2 mb-4"
                    >
                        <!-- Left: Zoom Controls -->
                        <div class="flex items-center gap-2">
                            <div class="join">
                                <button
                                    onclick={handleZoomOut}
                                    class="join-item btn btn-sm btn-ghost text-base-content/70 hover:text-base-content"
                                    title="Zoom Out (-)"
                                >
                                    <span
                                        class="material-symbols-rounded text-lg"
                                        >remove</span
                                    >
                                </button>
                                <button
                                    onclick={handleResetZoom}
                                    class="join-item btn btn-sm btn-ghost text-base-content/70 hover:text-base-content font-normal min-w-[60px]"
                                    title="Reset Zoom (0)"
                                >
                                    {konvaManager
                                        ? Math.round(
                                              konvaManager.getZoomPercentage(),
                                          ) + "%"
                                        : "100%"}
                                </button>
                                <button
                                    onclick={handleZoomIn}
                                    class="join-item btn btn-sm btn-ghost text-base-content/70 hover:text-base-content"
                                    title="Zoom In (=)"
                                >
                                    <span
                                        class="material-symbols-rounded text-lg"
                                        >add</span
                                    >
                                </button>
                            </div>

                            <div
                                class="divider divider-horizontal mx-0 h-6"
                            ></div>

                            <button
                                onclick={handleFitToScreen}
                                class="btn btn-sm btn-ghost gap-2 text-base-content/70 hover:text-base-content"
                                title="Fit to Screen (R)"
                            >
                                <span class="material-symbols-rounded text-lg"
                                    >fit_screen</span
                                >
                                <span class="hidden sm:inline font-normal"
                                    >Fit</span
                                >
                            </button>
                        </div>

                        <!-- Right: Annotation Tools (Hidden for current dev stage) -->
                        <div class="flex-1 flex justify-end">
                        </div>
                    </div>
                {/if}

                <!-- Single Konva Container - Always Present -->
                <div
                    class="flex-1 flex flex-col relative bg-slate-100 rounded-lg overflow-hidden border-2 border-slate-300 min-h-[500px]"
                >
                    <div
                        bind:this={konvaContainer}
                        class="w-full flex-1 bg-slate-50"
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
                            <div class="text-red-500 text-4xl mb-4">Warning</div>
                            <h3
                                class="text-lg font-semibold text-slate-800 mb-2"
                            >
                                Failed to Load
                            </h3>
                            <p class="text-slate-600 mb-4">
                                Unable to initialize the annotation editor.
                            </p>
                            <button
                                onclick={handleClose}
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

                    {#if isInitialized}
                        <div class="absolute bottom-4 right-4 z-10">
                            <div
                                class="badge badge-neutral badge-lg shadow-sm gap-2 p-3"
                            >
                                <span class="font-mono font-bold"
                                    >{annotationCount}</span
                                >
                                <span class="text-xs font-normal opacity-70"
                                    >annotations</span
                                >
                            </div>
                        </div>
                    {/if}
                </div>

                <!-- Instructions -->
                {#if isInitialized}
                    <div
                        class="mt-4 flex flex-col items-center justify-center text-sm text-base-content/50 gap-1"
                    >
                        <p>
                            Click and drag to select - Delete to remove - Save
                            when done
                        </p>
                        <p class="text-xs opacity-70">
                            Shortcuts: Zoom (+/-) - Reset (0) - Fit (R) - Select
                            All (Cmd+A) - Save (Cmd+S)
                        </p>
                    </div>
                {/if}
            </div>
        </div>
        <form method="dialog" class="modal-backdrop">
            <button onclick={(e) => handleClose(e)}>close</button>
        </form>
    </dialog>
{/if}

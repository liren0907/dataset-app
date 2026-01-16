<script lang="ts">
    import { onMount, tick } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import DatasetSummaryCard from "./components/DatasetSummaryCard.svelte";
    import ImagePreviewPanel from "./components/ImagePreviewPanel.svelte";
    import ImageGallery from "./components/ImageGallery.svelte";
    import ExportModal from "./components/ExportModal.svelte";
    import ModalAnnotationViewer from "./components/ModalAnnotationViewer.svelte";
    import CropRemapTool from "./components/CropRemapTool.svelte";
    import ExtractLabelsModal from "./components/ExtractLabelsModal.svelte";
    import {
        fetchPaginatedImages,
        fetchImageDetails,
        fetchDatasetSummary,
        performAutoAnnotation,
        performDatasetExport,
        performCropAndRemap,
    } from "./datasetService";
    import type {
        ProcessedImage,
        ImageDetailData,
        DatasetSummary,
        AutoAnnotationResult,
        AnnotatedImageInfo,
        DatasetExportParams,
    } from "./datasetService";
    import {
        MOCK_DIRECTORY_PATH,
        mockGetImages,
    } from "../../mocks/mockFileSystem";

    // State variables
    let directoryPath = "";
    let images: ProcessedImage[] = [];
    let loading = false;
    let loadingMore = false;
    let error = "";
    let viewMode = "grid";
    let selectedImage: ProcessedImage | null = null;
    let annotating = false;
    let autoAnnotating = false;
    let annotationType = "bounding_box";
    let datasetSummary: DatasetSummary | null = null;
    let autoAnnotationEnabled = true;
    let editMode: "modal" | "sidebar" = "modal";
    let showAnnotationModal = false;
    let isMockMode = false;

    // Export Modal State
    let showActualExportModal = false;
    let pageExportLoading = false;
    let pageExportError = "";
    let pageExportSuccess = "";

    // Crop & Remap Tool State
    let showCropTool = false;

    // Extract Labels Modal State
    let showExtractModal = false;
    let extractLoading = false;
    let extractError = "";
    let extractSuccess = "";

    // Pagination
    let currentPage = 1;
    let pageSize = 30;
    $: itemsPerPage = pageSize;
    let totalPages = 0;
    let totalImages = 0;

    // --- Initialization ---
    // Auto-load Mock Data if in Browser Environment
    onMount(async () => {
        const isTauri = typeof window !== "undefined" && "__TAURI__" in window;
        if (!isTauri) {
            console.log("🌍 Browser environment detected (No Tauri).");
            isMockMode = true;
            // Auto-load mock data
            await loadMockData();
        }
    });

    // Helper to load mock data
    async function loadMockData() {
        directoryPath = MOCK_DIRECTORY_PATH;
        loading = true;
        try {
            // Cast to any[] to avoid strict type checks on mock data properties
            const mockImages = (await mockGetImages(directoryPath)) as any[];

            // Map mock images to match ProcessedImage interface
            images = mockImages.map((img) => ({
                ...img,
                previewUrl: img.src,
                assetUrl: img.src,
                annotated: (img.k || 0) > 0, // Mock annotation status
                // Fill missing required properties with defaults
                name: img.name || "Mock Image",
                path: img.path || "",
            }));

            // Calculate Mock Stats
            const totalAnn = images.reduce(
                (acc, img) => acc + (img.annotated ? 1 : 0),
                0,
            );
            const classDist = { opening: 50, crane: 30, liftcar: 20 };

            datasetSummary = {
                total_images: images.length,
                images_with_annotations: totalAnn,
                total_annotations: totalAnn * 3, // Fake total annotations
                unique_labels: 3,
                label_counts: classDist,
                annotation_types: ["rectangle"], // Matches 'shape_type' in mock json
            };

            totalImages = images.length;
            totalPages = 1;
        } catch (err) {
            console.error("❌ Mock load failed", err);
            error = "Failed to load mock data.";
        } finally {
            loading = false;
        }
    }

    // Keyboard shortcuts
    function handleKeydown(event: KeyboardEvent) {
        if (event.key === "Escape" && selectedImage) {
            selectedImage = null;
        }
    }

    async function selectDirectory() {
        try {
            loading = true;
            error = "";

            if (isMockMode) {
                await loadMockData();
                return;
            }

            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select Image Directory",
            });

            if (selected) {
                directoryPath = selected as string;
                currentPage = 1;
                images = [];

                if (!directoryPath.trim()) {
                    error = "Invalid directory path: path is empty";
                    loading = false;
                    return;
                }

                console.log("Selected directory:", directoryPath);

                try {
                    await loadImagesPage(1);
                } catch (loadErr: any) {
                    console.error("Error in loadImagesPage:", loadErr);
                    error = `Failed to load images: ${loadErr.message || String(loadErr)}`;
                }
            }
        } catch (err: any) {
            console.error("Error selecting directory:", err);
            error = `Failed to select directory: ${err.message || String(err)}`;
        } finally {
            loading = false;
        }
    }

    async function toggleMockMode() {
        isMockMode = !isMockMode;
        images = [];
        directoryPath = "";
        datasetSummary = null;
        selectedImage = null;
        currentPage = 1;
        error = "";

        if (isMockMode) {
            await loadMockData();
        }
    }

    async function loadImagesPage(page: number) {
        try {
            loading = true;
            error = "";

            if (!directoryPath) {
                throw new Error("Directory path is empty for loadImagesPage.");
            }

            // If in Mock Mode, skip real backend call (logic already handled in initial load,
            // but for pagination we might need more logic later. For now, mock data is typically 1 page)
            if (isMockMode) {
                await loadMockData();
                return;
            }

            // Using itemsPerPage/pageSize consistently
            const result = await fetchPaginatedImages(
                directoryPath,
                page,
                pageSize,
            );

            images = result.processedImages;
            totalImages = result.totalImages;
            totalPages = result.totalPages;
            currentPage = page;

            if (page === 1) {
                generateLabelMeSummary();
            }

            if (autoAnnotationEnabled && images.length > 0) {
                await autoLoadAnnotationsForPage(page);
            }

            await tick();
            if (typeof window !== "undefined") {
                window.scrollTo(0, 0);
            }
        } catch (err: any) {
            console.error("Page: Error loading images:", err);
            error = `Failed to load images: ${err.message || "Unknown error"}`;
            images = [];
            totalImages = 0;
            totalPages = 0;
        } finally {
            loading = false;
            loadingMore = false;
        }
    }

    // Updated wrapper for pagination
    function handlePageChange(event: CustomEvent) {
        loadImagesPage(event.detail.page);
    }

    // View controls
    function changeViewMode(mode: string) {
        viewMode = mode;
    }

    // Function for auto-annotating images
    async function annotateImages() {
        if (isMockMode) {
            console.log("🧪 Mock Mode: Using local mock annotations.");
            await loadMockData();
            return;
        }

        try {
            if (!directoryPath || images.length === 0) {
                error = "Please select a directory with images first";
                return;
            }

            annotating = true;
            error = "";

            console.log(
                `Page: Starting auto-annotation (all types) for images in directory: ${directoryPath}`,
            );
            const result = await performAutoAnnotation(
                directoryPath,
                currentPage,
                pageSize,
            );

            console.log("Page: Annotation service result:", result);

            if (
                result &&
                result.annotated_images &&
                result.annotated_images.length > 0
            ) {
                // Merge annotation data with existing images
                images = images.map((img) => {
                    const annotatedImgData = result.annotated_images.find(
                        (ai) => ai.path === img.path,
                    );
                    if (annotatedImgData) {
                        return {
                            ...img,
                            annotations:
                                annotatedImgData.annotations || img.annotations,
                            annotated: !!(
                                annotatedImgData.has_json &&
                                annotatedImgData.annotations &&
                                annotatedImgData.annotations.length > 0
                            ),
                            has_json:
                                annotatedImgData.has_json !== undefined
                                    ? annotatedImgData.has_json
                                    : img.has_json,
                        };
                    }
                    return img;
                });

                await generateLabelMeSummary();
            }
        } catch (err: any) {
            console.error("Page: Error annotating images:", err);
            error = `Failed to annotate images: ${err.message || "Unknown error"}`;
        } finally {
            annotating = false;
        }
    }

    async function autoLoadAnnotationsForPage(page: number) {
        if (isMockMode) {
            console.log("🧪 Mock Mode: Skipping auto-annotation backend call.");
            return;
        }

        try {
            autoAnnotating = true;
            const result = await performAutoAnnotation(
                directoryPath,
                page,
                pageSize,
            );

            const mergedAnnotatedImages = result.annotated_images;

            if (mergedAnnotatedImages && mergedAnnotatedImages.length > 0) {
                images = images.map((img) => {
                    const annotatedImgData = mergedAnnotatedImages.find(
                        (ai) => ai.path === img.path,
                    );
                    if (annotatedImgData) {
                        console.log(
                            `Auto-updating annotations for ${img.name} (all types loaded in single call).`,
                        );
                        // Combine existing annotations with new ones (avoid duplicates)
                        const existingAnnotations = img.annotations || [];
                        const newAnnotations =
                            annotatedImgData.annotations || [];
                        const combinedAnnotations = [...existingAnnotations];

                        // Add new annotations, avoiding duplicates by checking label and shape_type
                        newAnnotations.forEach((newAnn) => {
                            const exists = combinedAnnotations.some(
                                (existing) =>
                                    existing.label === newAnn.label &&
                                    existing.shape_type === newAnn.shape_type &&
                                    JSON.stringify(existing.points) ===
                                        JSON.stringify(newAnn.points),
                            );
                            if (!exists) {
                                combinedAnnotations.push(newAnn);
                            }
                        });

                        return {
                            ...img,
                            annotations: combinedAnnotations,
                            annotated: combinedAnnotations.length > 0,
                            has_json:
                                annotatedImgData.has_json !== undefined
                                    ? annotatedImgData.has_json
                                    : img.has_json,
                        };
                    }
                    return img;
                });

                const imagesWithJson = mergedAnnotatedImages.filter(
                    (img) => img.has_json,
                ).length;
                const imagesWithActualAnnotations =
                    mergedAnnotatedImages.filter(
                        (img) => img.annotations && img.annotations.length > 0,
                    ).length;
                const totalAnnotationsLoaded = mergedAnnotatedImages.reduce(
                    (total, img) => total + (img.annotations?.length || 0),
                    0,
                );

                console.log(
                    `Auto-loaded ALL annotations: ${imagesWithJson} images have JSON, ${imagesWithActualAnnotations} have annotations, ${totalAnnotationsLoaded} total annotations loaded.`,
                );
            } else {
                console.log(
                    "Auto-annotation: No annotations found for either bounding boxes or polygons.",
                );
            }
        } catch (autoErr) {
            console.warn("Auto-annotation failed (non-blocking):", autoErr);
            // Don't set global error - auto-loading is optional enhancement
            // Users can still manually load annotations if needed
        } finally {
            autoAnnotating = false;
        }
    }

    // Function to generate LabelMe summary for the dataset
    async function generateLabelMeSummary() {
        try {
            if (!directoryPath) {
                // Check directoryPath directly, images.length check removed as summary can be generated even for empty dir
                console.warn(
                    "Page: Directory path not set, cannot generate LabelMe summary.",
                );
                return;
            }

            pageExportError = ""; // Clear any previous export errors
            pageExportSuccess = "";

            console.log(
                `Page: Generating LabelMe summary for directory: ${directoryPath}`,
            );

            const summaryData = await fetchDatasetSummary(directoryPath);
            datasetSummary = summaryData;
            console.log("Page: Dataset summary updated:", datasetSummary);
        } catch (err) {
            console.error("Page: Error generating LabelMe summary:", err);
            const errorMessage =
                err?.message ||
                (typeof err === "string" ? err : "Unknown error");
            // Display a less intrusive warning for summary generation failure,
            // as it might not be critical path for all users.
            console.warn(
                `Page: Failed to generate LabelMe summary: ${errorMessage}`,
            );
            datasetSummary = null; // Clear summary on error
            // error = `Failed to generate LabelMe summary: ${errorMessage}`; // Optionally set global error
        }
    }

    async function runUnifiedExport(exportDetails: any) {
        const {
            sourceDir,
            outputDir,
            mode,
            trainRatio,
            valRatio,
            testRatio,
            shapeType,
            includedLabels,
        } = exportDetails;

        if (!sourceDir || !outputDir || !includedLabels?.length) {
            pageExportError = "Missing required export parameters.";
            return;
        }

        pageExportLoading = true;
        pageExportError = "";
        pageExportSuccess = "";

        try {
            const params: DatasetExportParams = {
                sourceDir,
                outputDir,
                mode,
                trainRatio,
                valRatio,
                testRatio,
                shapeType,
                includedLabels,
            };

            const resultMessage = await performDatasetExport(params);
            pageExportSuccess = resultMessage;
            if (pageExportSuccess) {
                showActualExportModal = false;
            }
        } catch (err: any) {
            console.error(`Page: Error during unified export:`, err);
            pageExportError = `Failed to export dataset: ${err.message || "Unknown error"}`;
        } finally {
            pageExportLoading = false;
        }
    }

    // Handler for Crop & Remap completion
    async function handleCropCompleted(
        event: CustomEvent<{ outputDir: string }>,
    ) {
        const { outputDir } = event.detail;
        console.log("Crop completed, loading from:", outputDir);

        // Reset state and load the new directory
        error = "";
        directoryPath = outputDir;
        currentPage = 1;
        images = [];
        datasetSummary = null;
        selectedImage = null;
        showCropTool = false;

        try {
            await loadImagesPage(1);
        } catch (loadErr: any) {
            error = `Failed to load images from cropped directory: ${loadErr.message || "Unknown error"}`;
        }
    }

    // Handler for Extract Labels
    async function handleExtractLabels(
        event: CustomEvent<{
            sourceDir: string;
            outputDir: string;
            includedLabels: string[];
        }>,
    ) {
        const { sourceDir, outputDir, includedLabels } = event.detail;

        if (!sourceDir || !outputDir || includedLabels.length === 0) {
            extractError = "Missing required parameters for extraction.";
            return;
        }

        extractLoading = true;
        extractError = "";
        extractSuccess = "";

        try {
            const params: DatasetExportParams = {
                sourceDir,
                outputDir,
                mode: "labelme",
                includedLabels,
            };

            const resultMessage = await performDatasetExport(params);
            extractSuccess = resultMessage;

            // Close modal after short delay to show success
            setTimeout(() => {
                showExtractModal = false;
                extractSuccess = "";
            }, 2000);
        } catch (err: any) {
            extractError = `Extraction failed: ${err.message || "Unknown error"}`;
        } finally {
            extractLoading = false;
        }
    }
</script>

<svelte:head>
    <title>Dataset Gallery</title>
    <meta
        name="description"
        content="Efficient image viewer for large image collections"
    />
</svelte:head>

<svelte:window on:keydown={handleKeydown} />

<div class="container mx-auto px-4 py-8">
    <div class="max-w-6xl mx-auto">
        <div class="mb-8">
            <h1 class="text-3xl font-bold text-base-content mb-6">
                Dataset Gallery
            </h1>

            <!-- Navbar Toolbar (Modern Minimalist) -->
            <div
                class="navbar bg-base-100 min-h-0 h-14 border border-base-200 shadow-sm rounded-lg px-3 gap-2 mb-6"
            >
                <!-- Left: Directory & Breadcrumbs -->
                <div class="flex items-center gap-2 flex-1 min-w-0">
                    <!-- Mock Mode Toggle -->
                    <div
                        class="tooltip tooltip-bottom"
                        data-tip={isMockMode
                            ? "Switch to Real Data"
                            : "Switch to Mock Data"}
                    >
                        <button
                            class={`btn btn-sm btn-circle ${isMockMode ? "btn-secondary text-secondary-content" : "btn-ghost text-base-content/40"}`}
                            on:click={toggleMockMode}
                            aria-label="Toggle Mock Mode"
                        >
                            <span class="material-symbols-rounded text-lg"
                                >science</span
                            >
                        </button>
                    </div>

                    <div class="divider divider-horizontal mx-0 h-6"></div>

                    <button
                        on:click={selectDirectory}
                        class="btn btn-ghost btn-sm gap-2 text-base-content/70 hover:text-base-content hover:bg-base-200"
                        disabled={loading}
                        title="Open Project Directory"
                    >
                        {#if loading}
                            <span class="loading loading-spinner loading-xs"
                            ></span>
                        {:else}
                            <span class="material-symbols-rounded text-lg"
                                >folder_open</span
                            >
                        {/if}
                    </button>

                    <div class="divider divider-horizontal mx-0 h-6"></div>

                    {#if directoryPath}
                        <div class="breadcrumbs text-sm ml-1 hidden sm:block">
                            <ul>
                                <li>
                                    <span class="text-base-content/50"
                                        >Project</span
                                    >
                                </li>
                                {#each directoryPath
                                    .split("/")
                                    .slice(-2) as part, i}
                                    <li>
                                        <span
                                            class={`font-medium ${i === 1 ? "text-base-content" : "text-base-content/70"}`}
                                        >
                                            {part}
                                        </span>
                                    </li>
                                {/each}
                            </ul>
                        </div>
                        <!-- Mobile fallback for path -->
                        <div
                            class="text-sm font-medium text-base-content truncated sm:hidden"
                        >
                            {directoryPath.split("/").pop()}
                        </div>
                    {:else}
                        <span class="text-sm text-base-content/50 italic ml-1"
                            >No directory selected</span
                        >
                    {/if}
                </div>

                <!-- Right: Tools & Actions -->
                <div class="flex items-center gap-1 sm:gap-2 flex-none">
                    <!-- Annotation Type Toggle -->
                    <div class="join hidden lg:flex">
                        <button
                            class={`btn btn-sm join-item gap-2 border-0 px-4 ${annotationType === "bounding_box" ? "bg-base-200 text-base-content shadow-inner" : "btn-ghost text-base-content/70"}`}
                            on:click={() => (annotationType = "bounding_box")}
                            disabled={!directoryPath || images.length === 0}
                            title="Bounding Boxes"
                        >
                            <span class="material-symbols-rounded text-lg"
                                >crop_square</span
                            >
                            <span class="hidden xl:inline font-normal"
                                >Boxes</span
                            >
                        </button>
                        <button
                            class={`btn btn-sm join-item gap-2 border-0 px-4 ${annotationType === "polygon" ? "bg-base-200 text-base-content shadow-inner" : "btn-ghost text-base-content/70"}`}
                            on:click={() => (annotationType = "polygon")}
                            disabled={!directoryPath || images.length === 0}
                            title="Polygons"
                        >
                            <span class="material-symbols-rounded text-lg"
                                >hexagon</span
                            >
                            <span class="hidden xl:inline font-normal"
                                >Polygon</span
                            >
                        </button>
                    </div>

                    <div
                        class="divider divider-horizontal mx-0 h-6 hidden lg:flex"
                    ></div>

                    <!-- Auto-load Toggle -->
                    <div
                        class="tooltip tooltip-bottom"
                        data-tip={autoAnnotationEnabled
                            ? "Auto-load active"
                            : "Auto-load inactive"}
                    >
                        <button
                            class={`btn btn-sm btn-square join-item ${autoAnnotationEnabled ? "btn-active text-primary" : "btn-ghost text-base-content/50"}`}
                            on:click={() =>
                                (autoAnnotationEnabled =
                                    !autoAnnotationEnabled)}
                            disabled={!directoryPath}
                            aria-label="Toggle Auto-load Annotations"
                        >
                            <span class="material-symbols-rounded text-xl"
                                >autorenew</span
                            >
                        </button>
                    </div>

                    <div
                        class="divider divider-horizontal mx-0 h-6 hidden lg:flex"
                    ></div>

                    <!-- Actions -->
                    <button
                        on:click={annotateImages}
                        class="btn btn-ghost btn-sm btn-square text-base-content/70 hover:text-primary hover:bg-primary/10"
                        disabled={!directoryPath ||
                            images.length === 0 ||
                            annotating ||
                            autoAnnotating}
                        title="Load Annotations"
                    >
                        {#if annotating}
                            <span class="loading loading-spinner loading-xs"
                            ></span>
                        {:else}
                            <span class="material-symbols-rounded text-xl"
                                >label</span
                            >
                        {/if}
                    </button>

                    <button
                        on:click={() => {
                            showActualExportModal = true;
                            pageExportError = "";
                            pageExportSuccess = "";
                        }}
                        class="btn btn-ghost btn-sm btn-square text-base-content/70 hover:text-primary hover:bg-primary/10"
                        disabled={!directoryPath || images.length === 0}
                        title="Export Dataset"
                    >
                        <span class="material-symbols-rounded text-xl"
                            >ios_share</span
                        >
                    </button>

                    <!-- Crop & Remap Button -->
                    <button
                        on:click={() => (showCropTool = !showCropTool)}
                        class="btn btn-ghost btn-sm btn-square text-base-content/70 hover:text-secondary hover:bg-secondary/10"
                        class:bg-secondary={showCropTool}
                        title="Crop & Remap Tool"
                    >
                        <span class="material-symbols-rounded text-xl"
                            >crop</span
                        >
                    </button>

                    <!-- Extract Labels Button -->
                    <button
                        on:click={() => {
                            extractError = "";
                            extractSuccess = "";
                            showExtractModal = true;
                        }}
                        class="btn btn-ghost btn-sm btn-square text-base-content/70 hover:text-accent hover:bg-accent/10"
                        disabled={!directoryPath || !datasetSummary}
                        title="Extract Labels"
                    >
                        <span class="material-symbols-rounded text-xl"
                            >label</span
                        >
                    </button>

                    <div class="divider divider-horizontal mx-0 h-6"></div>

                    <!-- View Mode Toggle -->
                    <div class="join">
                        <button
                            class={`btn btn-sm join-item btn-square border-0 ${viewMode === "grid" ? "bg-base-200 text-base-content shadow-inner" : "btn-ghost text-base-content/60"}`}
                            on:click={() => (viewMode = "grid")}
                            title="Grid View"
                        >
                            <span class="material-symbols-rounded text-lg"
                                >grid_view</span
                            >
                        </button>
                        <button
                            class={`btn btn-sm join-item btn-square border-0 ${viewMode === "column" ? "bg-base-200 text-base-content shadow-inner" : "btn-ghost text-base-content/60"}`}
                            on:click={() => (viewMode = "column")}
                            title="List View"
                        >
                            <span class="material-symbols-rounded text-lg"
                                >view_list</span
                            >
                        </button>
                    </div>

                    <!-- Divider -->
                    <div class="divider divider-horizontal mx-0 h-6"></div>

                    <!-- Edit Mode Toggle -->
                    <div class="join">
                        <button
                            class={`btn btn-sm join-item gap-2 border-0 px-6 ${editMode === "modal" ? "bg-base-200 text-base-content shadow-inner" : "btn-ghost text-base-content/60"}`}
                            on:click={() => (editMode = "modal")}
                            title="Pop-out Editor Mode"
                        >
                            <span class="material-symbols-rounded text-lg"
                                >open_in_new</span
                            >
                            <span class="hidden 2xl:inline">Pop-out</span>
                        </button>
                        <button
                            class={`btn btn-sm join-item gap-2 border-0 px-6 ${editMode === "sidebar" ? "bg-base-200 text-base-content shadow-inner" : "btn-ghost text-base-content/60"}`}
                            on:click={() => (editMode = "sidebar")}
                            title="Sidebar Editor Mode"
                        >
                            <span class="material-symbols-rounded text-lg"
                                >view_sidebar</span
                            >
                            <span class="hidden 2xl:inline">Sidebar</span>
                        </button>
                    </div>
                </div>
            </div>

            {#if error}
                <div class="alert alert-error mb-6">
                    <span class="material-symbols-rounded">error</span>
                    <span>{error}</span>
                </div>
            {/if}

            <!-- Main Content Area - Always Visible -->
            <!-- 1. Dataset Summary Card - Always show with empty state -->
            <div class="mb-8">
                <DatasetSummaryCard {datasetSummary} />
            </div>

            <!-- 2. Image Gallery and Loading States -->
            <!-- Main Content Area with Sidebar Layout -->
            <div
                class="flex flex-col lg:flex-row gap-6 items-start h-[calc(100vh-200px)]"
            >
                <!-- Gallery Grid Area -->
                <div
                    class="flex-1 w-full h-full overflow-hidden flex flex-col transition-all duration-300"
                >
                    {#if loading}
                        <div
                            class="flex flex-col items-center justify-center p-12 bg-base-100 rounded-box shadow-sm border border-base-300 h-full"
                        >
                            <span
                                class="loading loading-spinner loading-lg text-primary mb-4"
                            ></span>
                            <div class="text-lg font-medium">
                                Loading dataset...
                            </div>
                            <div class="text-sm text-base-content/60 mt-2">
                                Please wait while we process the images
                            </div>
                        </div>
                    {:else if !directoryPath}
                        <!-- Empty State - No Directory -->
                        <div
                            class="flex flex-col items-center justify-center p-12 bg-base-100 rounded-box shadow-sm border border-base-300 h-full text-center"
                        >
                            <div
                                class="w-24 h-24 bg-base-200 rounded-full flex items-center justify-center mb-6"
                            >
                                <span
                                    class="material-symbols-rounded text-6xl text-base-content/30"
                                    >folder_open</span
                                >
                            </div>
                            <h3 class="text-xl font-bold mb-2">
                                No Dataset Selected
                            </h3>
                            <p class="text-base-content/60 max-w-md mb-6">
                                Please select a directory properly formatted for
                                LabelMe or YOLO to inspect the dataset
                                statistics.
                            </p>
                            <button
                                on:click={selectDirectory}
                                class="btn btn-primary"
                            >
                                Select Directory
                            </button>
                        </div>
                    {:else if images.length === 0}
                        <!-- Empty State - No Images -->
                        <div
                            class="card bg-base-100 shadow-sm border border-base-300 h-full"
                        >
                            <div
                                class="card-body items-center text-center justify-center"
                            >
                                <span
                                    class="material-symbols-rounded text-6xl text-base-content/20 mb-4"
                                    >image_not_supported</span
                                >
                                <h3 class="font-bold text-lg">
                                    No images found
                                </h3>
                                <p class="text-base-content/60">
                                    The selected directory doesn't contain any
                                    supported image files.
                                </p>
                            </div>
                        </div>
                    {:else}
                        <!-- Image Gallery Component -->
                        <div
                            class="h-full overflow-y-auto pr-2 rounded-box border border-base-300 bg-base-100"
                        >
                            <ImageGallery
                                {images}
                                {viewMode}
                                {currentPage}
                                {totalPages}
                                {itemsPerPage}
                                {selectedImage}
                                on:pageChange={handlePageChange}
                                on:imageClick={(e) => {
                                    // Handle click based on Edit Mode
                                    if (editMode === "modal") {
                                        selectedImage = e.detail.image;
                                        showAnnotationModal = true;
                                    } else {
                                        // Sidebar mode: just select it
                                        selectedImage = e.detail.image;
                                    }
                                }}
                            />
                        </div>
                    {/if}
                </div>

                <!-- Right Sidebar Panel -->
                {#if selectedImage && editMode === "sidebar"}
                    <div
                        class="w-full lg:w-[450px] xl:w-[500px] h-full flex-none bg-base-100 rounded-box shadow-xl border border-base-300 overflow-hidden flex flex-col animate-in slide-in-from-right-4 duration-300"
                    >
                        <ImagePreviewPanel
                            {selectedImage}
                            on:close={() => (selectedImage = null)}
                        />
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>

<!-- Unified Export Modal -->
<ExportModal
    bind:showModal={showActualExportModal}
    currentDirectoryPath={directoryPath}
    currentDatasetSummary={datasetSummary}
    on:closeModal={() => {
        showActualExportModal = false;
    }}
    on:runExport={(event) => runUnifiedExport(event.detail)}
/>

<!-- Modal Annotation Viewer (Pop-out Mode) -->
{#if showAnnotationModal && selectedImage}
    <ModalAnnotationViewer
        bind:showModal={showAnnotationModal}
        {selectedImage}
        {autoAnnotationEnabled}
        {isMockMode}
        on:close={() => {
            showAnnotationModal = false;
        }}
        on:save={(e) => {
            console.log("Annotation retained/saved via modal");
        }}
    />
{/if}

<!-- Extract Labels Modal -->
<ExtractLabelsModal
    isOpen={showExtractModal}
    sourceDir={directoryPath}
    {datasetSummary}
    on:extract={handleExtractLabels}
    on:close={() => (showExtractModal = false)}
/>

<!-- Crop & Remap Modal -->
<CropRemapTool
    isOpen={showCropTool}
    on:cropCompleted={handleCropCompleted}
    on:close={() => (showCropTool = false)}
/>

<style>
</style>

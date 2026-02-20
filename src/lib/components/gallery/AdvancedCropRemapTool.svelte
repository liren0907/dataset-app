<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { safeConvertFileSrc } from "$lib/utils/tauriUtils";
    import { open } from "@tauri-apps/plugin-dialog";
    import { appDataDir } from "@tauri-apps/api/path";
    import { onMount } from "svelte";
    import { createEventDispatcher } from "svelte";
    import KonvaViewer from "./KonvaViewer.svelte";
    import type { KonvaImageData } from "$lib/services/gallery/konvaService";

    // Props from parent (dataset-gallery-advanced)
    export let currentDirectory: string = ""; // Current gallery directory
    export let cropToolOpen: boolean = false; // Accordion open state
    export let preSelectedParentLabel: string = ""; // Pre-selected parent label from DatasetSummary click

    // Event dispatcher for communication with parent
    const dispatch = createEventDispatcher();

    // State variables
    let sourceDir: string | null = null;
    let outputDir: string | null = null;

    // Dynamic label selection
    let datasetSummary: any = null;
    let availableLabels: string[] = [];
    let selectedParentLabel: string = "person"; // Default
    let selectedChildLabels: string[] = []; // Dynamic selection
    let datasetLoaded: boolean = false;
    let analyzing: boolean = false;
    let paddingFactor: number = 1.2; // Default 20% padding

    // Auto-preview state
    let previewImages: any[] = []; // 5 randomly selected images for preview
    let previewLoading: boolean = false;

    let loading: boolean = false;
    let successMessage: string | null = null;
    let errorMessage: string | null = null;

    // Preview modal state
    let showPreviewModal: boolean = false;
    let previewModalImage: KonvaImageData | null = null;

    // Reactive: Auto-set source directory from gallery when available
    $: if (currentDirectory && !sourceDir && cropToolOpen) {
        sourceDir = currentDirectory;
        if (sourceDir) {
            analyzeDataset();
        }
    }

    // Reactive: Auto-select parent label when pre-selected from DatasetSummary
    $: if (
        preSelectedParentLabel &&
        availableLabels.includes(preSelectedParentLabel)
    ) {
        selectedParentLabel = preSelectedParentLabel;
    }

    // Handle keyboard events for modal
    function handleKeydown(event: KeyboardEvent) {
        if (event.key === "Escape" && showPreviewModal) {
            closePreviewModal();
        }
    }

    async function selectDirectory(type: "source" | "output") {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: `Select ${type === "source" ? "Source" : "Output"} Directory`,
            });

            if (selected && typeof selected === "string") {
                if (type === "source") {
                    sourceDir = selected as string;
                    // Reset dataset analysis when source changes
                    datasetSummary = null;
                    availableLabels = [];
                    datasetLoaded = false;
                    selectedChildLabels = [];
                    // Reset preview state
                    previewImages = [];
                    previewLoading = false;
                    // Automatically analyze dataset after selecting source directory
                    await analyzeDataset();
                } else {
                    outputDir = selected;
                }
                // Clear messages when a directory is selected
                successMessage = null;
                errorMessage = null;
            }
        } catch (err) {
            console.error("Error selecting directory:", err);
            errorMessage = `Failed to select directory: ${err instanceof Error ? err.message : String(err)}`;
        }
    }

    function suggestParentLabel(labels: string[]): string {
        // Priority order for common parent labels
        const commonParents = [
            "person",
            "people",
            "human",
            "worker",
            "individual",
        ];

        for (const parent of commonParents) {
            if (labels.includes(parent)) {
                return parent;
            }
        }

        // Fallback to most common label (by count)
        if (datasetSummary?.label_counts) {
            const sortedLabels = Object.entries(datasetSummary.label_counts)
                .sort(([, a], [, b]) => (b as number) - (a as number))
                .map(([label]) => label);
            return sortedLabels[0] || "person";
        }

        return labels[0] || "person";
    }

    function suggestChildLabels(labels: string[]): string[] {
        const safetyEquipment = [
            "safety_helmet",
            "helmet",
            "hard_hat",
            "reflective_vest",
            "vest",
            "safety_vest",
            "body_harness",
            "harness",
            "safety_harness",
            "gloves",
            "safety_gloves",
            "boots",
            "safety_boots",
        ];

        return labels.filter((label) =>
            safetyEquipment.some((safety) =>
                label.toLowerCase().includes(safety.toLowerCase()),
            ),
        );
    }

    async function analyzeDataset() {
        if (!sourceDir) return;

        try {
            analyzing = true;
            errorMessage = null;

            console.log("Analyzing dataset:", sourceDir);
            const result = (await invoke("get_labelme_summary", {
                path: sourceDir as string,
            })) as string;
            datasetSummary = JSON.parse(result);

            console.log("Dataset summary:", datasetSummary);

            // Extract available labels
            availableLabels = Object.keys(datasetSummary.label_counts || {});

            if (availableLabels.length === 0) {
                errorMessage =
                    "No labels found in the dataset. Please check if the directory contains LabelMe JSON files.";
                return;
            }

            // Set smart defaults
            selectedParentLabel = suggestParentLabel(availableLabels);
            selectedChildLabels = suggestChildLabels(availableLabels);

            datasetLoaded = true;

            console.log(
                "Dataset analysis complete, triggering auto-preview...",
            );
            // Auto-preview 5 random annotated images
            await autoPreviewAnnotatedImages();
        } catch (err) {
            console.error("Error analyzing dataset:", err);
            errorMessage = `Failed to analyze dataset: ${err instanceof Error ? err.message : String(err)}`;
            datasetSummary = null;
            availableLabels = [];
            datasetLoaded = false;
        } finally {
            analyzing = false;
        }
    }

    // Function to automatically preview 5 random annotated images
    async function autoPreviewAnnotatedImages() {
        try {
            previewLoading = true;
            previewImages = [];

            if (!datasetSummary || !sourceDir) {
                console.log("No dataset available for preview");
                return;
            }

            console.log("Starting auto-preview of annotated images");

            // Create a temporary directory for previews in app data directory
            const appData = await appDataDir();
            const tempDir = `${appData}previews_${Date.now()}`;

            // Generate annotated preview images using the backend
            const result = (await invoke("generate_annotated_previews", {
                sourceDir: sourceDir as string,
                numPreviews: 5,
                tempDir: tempDir,
            })) as string;

            const data = JSON.parse(result);

            if (data.annotated_images && data.annotated_images.length > 0) {
                console.log(
                    `Loaded ${data.annotated_images.length} images with annotation data`,
                );

                // Convert file paths to proper Tauri URLs and prepare preview data
                const selectedImages: KonvaImageData[] =
                    data.annotated_images.map(
                        (imageData: any, index: number) => ({
                            id: `preview_${index}`,
                            path: imageData.path,
                            previewUrl: safeConvertFileSrc(imageData.path), // Load original clean image
                            name: `Preview ${index + 1}`,
                            annotations: imageData.annotations || [], // Draw annotations with KonvaJS
                        }),
                    );

                previewImages = selectedImages;
                console.log(
                    `Auto-preview ready with ${selectedImages.length} annotated images`,
                );
            } else {
                console.log("No annotated preview images were generated");
            }
        } catch (err) {
            console.error("Error in auto-preview:", err);
            previewImages = [];
        } finally {
            previewLoading = false;
        }
    }

    // Function to open preview modal
    function openPreviewModal(image: KonvaImageData) {
        console.log("Opening preview modal for image:", image);
        previewModalImage = image;
        showPreviewModal = true;
    }

    // Function to close preview modal
    function closePreviewModal() {
        showPreviewModal = false;
        previewModalImage = null;
    }

    // Function to handle window resize for modal
    function handleResize() {
        // Handle modal responsiveness if needed
    }

    // Function to get filtered child labels
    function getFilteredChildLabels(): string[] {
        if (!availableLabels.length) return [];
        return availableLabels;
    }

    // Function to validate selection
    function validateSelection(): string | null {
        if (!selectedParentLabel) {
            return "Please select a parent label";
        }
        if (selectedChildLabels.length === 0) {
            return "Please select at least one child label";
        }
        return null;
    }

    // Function to run the processing
    async function runProcessing() {
        if (!sourceDir || !outputDir || !selectedParentLabel) return;

        try {
            loading = true;
            errorMessage = null;

            console.log("Running crop and remap processing...");
            const message = await invoke("crop_and_remap_annotations", {
                sourceDir: sourceDir,
                outputDir: outputDir,
                parentLabel: selectedParentLabel,
                childLabels: selectedChildLabels,
                paddingFactor: paddingFactor,
            });

            successMessage = String(message);

            // Parse result to get image count if available
            let imageCount = 0;
            try {
                // Try to extract count from message (e.g., "Processed 150 images")
                const match = String(message).match(/(\d+)\s*image/i);
                if (match) {
                    imageCount = parseInt(match[1], 10);
                }
            } catch {}

            // Dispatch completion event to parent (gallery) with output directory and details
            dispatch("cropCompleted", {
                outputDir: outputDir,
                parentLabel: selectedParentLabel,
                childLabels: selectedChildLabels,
                imageCount: imageCount,
            });
        } catch (err) {
            console.error("Error running processing:", err);
            errorMessage = `Processing failed: ${err instanceof Error ? err.message : String(err)}`;
        } finally {
            loading = false;
        }
    }

    // Cleanup on component destruction
    function cleanup() {
        // Cleanup code if needed
    }

    onMount(() => {
        return cleanup;
    });

    // Handle close event from KonvaViewer
    function handleViewerClose() {
        closePreviewModal();
    }
    // --- UI Helpers for New Template ---
    function selectSourceDirectory() {
        selectDirectory("source");
    }

    function selectOutputDirectory() {
        selectDirectory("output");
    }

    function selectParent(label: string) {
        selectedParentLabel = label;
    }

    function toggleChild(label: string) {
        if (selectedChildLabels.includes(label)) {
            selectedChildLabels = selectedChildLabels.filter(
                (l) => l !== label,
            );
        } else {
            selectedChildLabels = [...selectedChildLabels, label];
        }
    }
</script>

<svelte:window on:keydown={handleKeydown} on:resize={handleResize} />

<div class="advanced-crop-remap-tool p-1">
    <div class="space-y-4">
        <!-- Source Directory -->
        <div class="form-control">
            <label
                for="sourceDirectoryInput"
                class="label font-medium text-base-content/80 pt-0"
                >Source Directory</label
            >
            <div class="join w-full">
                <input
                    id="sourceDirectoryInput"
                    type="text"
                    readonly
                    placeholder="Select source directory..."
                    value={sourceDir || ""}
                    class="input input-bordered join-item flex-1 bg-base-200 text-base-content/70 text-sm"
                />
                <button
                    on:click={() => selectDirectory("source")}
                    class="btn btn-neutral join-item border-base-300"
                    disabled={analyzing}
                >
                    Browse...
                </button>
            </div>
            {#if currentDirectory && !sourceDir}
                <p class="text-xs text-info mt-1 flex items-center gap-1">
                    <span class="material-symbols-rounded text-sm">info</span>
                    Using current gallery directory: {currentDirectory}
                </p>
            {/if}
        </div>

        <!-- Dataset Summary -->
        {#if datasetSummary}
            <div class="alert alert-info shadow-sm py-3">
                <div class="flex flex-col w-full">
                    <h4
                        class="font-bold text-sm mb-2 opacity-80 flex items-center gap-2"
                    >
                        <span class="material-symbols-rounded">analytics</span>
                        Dataset Summary
                    </h4>
                    <div class="grid grid-cols-2 gap-x-8 gap-y-1 text-sm">
                        <div class="flex justify-between">
                            <span class="opacity-70">Total Images:</span>
                            <span class="font-mono font-medium"
                                >{datasetSummary.total_images}</span
                            >
                        </div>
                        <div class="flex justify-between">
                            <span class="opacity-70">With Annotations:</span>
                            <span class="font-mono font-medium"
                                >{datasetSummary.images_with_annotations}</span
                            >
                        </div>
                        <div class="flex justify-between">
                            <span class="opacity-70">Total Annotations:</span>
                            <span class="font-mono font-medium"
                                >{datasetSummary.total_annotations}</span
                            >
                        </div>
                        <div class="flex justify-between">
                            <span class="opacity-70">Unique Labels:</span>
                            <span class="font-mono font-medium"
                                >{datasetSummary.unique_labels}</span
                            >
                        </div>
                    </div>
                </div>
            </div>
        {/if}

        <!-- Auto Preview Section -->
        {#if datasetLoaded}
            <div
                class="card bg-base-200 border border-base-300 shadow-inner p-4"
            >
                <h4
                    class="font-medium text-base-content mb-3 flex items-center gap-2"
                >
                    <span class="material-symbols-rounded text-primary"
                        >visibility</span
                    >
                    Dataset Preview
                    {#if previewLoading}
                        <span
                            class="loading loading-spinner loading-xs text-primary"
                        ></span>
                    {/if}
                </h4>

                {#if previewLoading}
                    <div class="text-sm text-base-content/60 py-4 text-center">
                        Loading preview images...
                    </div>
                {:else if previewImages.length > 0}
                    <div class="text-xs text-base-content/60 mb-3">
                        Here are {previewImages.length} random annotated images from
                        your dataset with annotations already drawn on them:
                    </div>
                    <div
                        class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-3"
                    >
                        {#each previewImages as image, index (image.path)}
                            <div
                                class="card card-compact bg-base-100 shadow-sm border border-base-200 overflow-hidden hover:shadow-md transition-all cursor-pointer group"
                                on:click={() => openPreviewModal(image)}
                                on:keydown={(e) =>
                                    e.key === "Enter" &&
                                    openPreviewModal(image)}
                                role="button"
                                tabindex="0"
                            >
                                <figure class="relative pb-[75%] bg-base-300">
                                    <img
                                        src={image.previewUrl}
                                        alt={image.name}
                                        class="absolute inset-0 w-full h-full object-cover transition-transform group-hover:scale-110"
                                    />
                                </figure>
                                <div class="card-body p-2 gap-0">
                                    <h5 class="text-xs font-medium truncate">
                                        {image.name}
                                    </h5>
                                    <div class="flex gap-1 mt-1">
                                        <div
                                            class="badge badge-xs badge-primary"
                                        >
                                            {image.annotations?.length || 0}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        {/if}
    </div>
</div>
<!-- Main Tool Card -->
<div class="card bg-base-100 shadow-xl border border-base-200 overflow-hidden">
    <!-- Header Gradient -->
    <div
        class="h-1 w-full bg-gradient-to-r from-primary/50 to-secondary/50"
    ></div>

    <div class="card-body p-6 gap-6">
        <!-- Source Directory Section -->
        <div class="space-y-3">
            <span
                class="label font-bold text-xs text-base-content/40 uppercase tracking-wider pl-1"
            >
                Input Source
            </span>

            <!-- Seamless Input Group: Source Dir -->
            <div
                class="flex items-center w-full px-1 py-1 rounded-xl border border-base-300 bg-base-100 hover:border-base-content/30 focus-within:ring-2 focus-within:ring-primary/20 focus-within:border-primary transition-all shadow-sm"
            >
                <div class="pl-3 text-base-content/40">
                    <span class="material-symbols-rounded">folder</span>
                </div>
                <input
                    type="text"
                    bind:value={sourceDir}
                    readonly
                    placeholder="Select source directory..."
                    class="input input-ghost w-full focus:outline-none border-none bg-transparent h-10 text-sm"
                />
                <button
                    on:click={selectSourceDirectory}
                    class="btn btn-sm btn-ghost bg-base-200/80 hover:bg-base-300 text-base-content/70 mr-1 rounded-lg font-medium"
                >
                    Browse
                </button>
            </div>

            <!-- Dataset Stats (Mini-dashboard) -->
            {#if datasetSummary}
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 mt-2">
                    <div
                        class="flex flex-col items-center p-2 rounded-lg bg-base-200/30 border border-base-200"
                    >
                        <span
                            class="text-xs text-base-content/50 uppercase font-bold"
                            >Total</span
                        >
                        <span class="font-bold text-lg"
                            >{datasetSummary.total_images}</span
                        >
                    </div>
                    <div
                        class="flex flex-col items-center p-2 rounded-lg bg-base-200/30 border border-base-200"
                    >
                        <span
                            class="text-xs text-base-content/50 uppercase font-bold"
                            >Classes</span
                        >
                        <span class="font-bold text-lg"
                            >{Object.keys(datasetSummary.label_counts)
                                .length}</span
                        >
                    </div>
                    <!-- Add more stats if needed -->
                </div>
            {/if}
        </div>

        {#if datasetLoaded}
            <div class="divider my-0 opacity-50"></div>

            <!-- Label Configuration Section -->
            <div class="space-y-6">
                <!-- Parent Label Selection -->
                <div>
                    <span
                        class="label font-bold text-xs text-base-content/40 uppercase tracking-wider pl-1 mb-2"
                    >
                        Target Class (Parent)
                    </span>
                    {#if availableLabels.length > 0}
                        <div class="flex flex-wrap gap-2">
                            {#each availableLabels as label}
                                <button
                                    on:click={() => selectParent(label)}
                                    class={`px-4 py-2 rounded-xl border transition-all duration-200 flex items-center gap-2
                                            ${
                                                selectedParentLabel === label
                                                    ? "bg-primary text-primary-content border-primary shadow-md shadow-primary/20 scale-[1.02]"
                                                    : "bg-base-100 text-base-content/70 border-base-300 hover:border-base-content/30 hover:bg-base-200/50"
                                            }`}
                                >
                                    <span class="font-bold">{label}</span>
                                    {#if datasetSummary?.label_counts[label]}
                                        <span
                                            class="badge badge-sm bg-base-100/20 border-0 text-current opacity-80"
                                        >
                                            {datasetSummary.label_counts[label]}
                                        </span>
                                    {/if}
                                </button>
                            {/each}
                        </div>
                    {:else}
                        <div
                            class="p-8 text-center border-2 border-dashed border-base-300 rounded-xl bg-base-100/50 text-base-content/50"
                        >
                            No labels found.
                        </div>
                    {/if}
                </div>

                <!-- Child Label Selection -->
                {#if selectedParentLabel}
                    <div class="animate-fadeIn">
                        <span
                            class="label font-bold text-xs text-base-content/40 uppercase tracking-wider pl-1 mb-2 flex justify-between"
                        >
                            <span>Children Classes (Sub-components)</span>
                            <span class="text-xs font-normal opacity-60"
                                >Select at least one</span
                            >
                        </span>

                        <div
                            class="p-4 rounded-2xl bg-base-200/30 border border-base-200"
                        >
                            <div class="flex flex-wrap gap-2">
                                {#each availableLabels.filter((l) => l !== selectedParentLabel) as label}
                                    <button
                                        on:click={() => toggleChild(label)}
                                        class={`px-3 py-1.5 rounded-lg text-sm border transition-all duration-200 flex items-center gap-2
                                                ${
                                                    selectedChildLabels.includes(
                                                        label,
                                                    )
                                                        ? "bg-secondary/10 text-secondary border-secondary/50 font-bold"
                                                        : "bg-base-100 text-base-content/60 border-base-200 hover:border-base-300"
                                                }`}
                                    >
                                        <!-- Checkbox visual -->
                                        <div
                                            class={`w-4 h-4 rounded border flex items-center justify-center transition-colors
                                                ${selectedChildLabels.includes(label) ? "bg-secondary border-secondary" : "border-base-content/30"}`}
                                        >
                                            {#if selectedChildLabels.includes(label)}
                                                <span
                                                    class="material-symbols-rounded text-[10px] text-white font-bold"
                                                    >check</span
                                                >
                                            {/if}
                                        </div>
                                        {label}
                                    </button>
                                {/each}
                            </div>
                            {#if availableLabels.length < 2}
                                <div
                                    class="text-sm text-base-content/50 italic py-2 text-center"
                                >
                                    Not enough classes to create a hierarchy.
                                </div>
                            {/if}
                        </div>
                    </div>
                {/if}
            </div>

            <!-- Output Directory -->
            <div class="space-y-3 pt-2">
                <span
                    class="label font-bold text-xs text-base-content/40 uppercase tracking-wider pl-1"
                >
                    Destination
                </span>
                <div
                    class="flex items-center w-full px-1 py-1 rounded-xl border border-base-300 bg-base-100 hover:border-base-content/30 focus-within:ring-2 focus-within:ring-primary/20 focus-within:border-primary transition-all shadow-sm"
                >
                    <div class="pl-3 text-base-content/40">
                        <span class="material-symbols-rounded">output</span>
                    </div>
                    <input
                        type="text"
                        bind:value={outputDir}
                        readonly
                        placeholder="Select output directory..."
                        class="input input-ghost w-full focus:outline-none border-none bg-transparent h-10 text-sm"
                    />
                    <button
                        on:click={selectOutputDirectory}
                        class="btn btn-sm btn-ghost bg-base-200/80 hover:bg-base-300 text-base-content/70 mr-1 rounded-lg font-medium"
                    >
                        Browse
                    </button>
                </div>
            </div>
        {/if}

        <!-- Analysis Loading State -->
        {#if !datasetLoaded && sourceDir}
            <div class="alert alert-warning shadow-sm">
                <span class="loading loading-spinner loading-sm"></span>
                <span>Analyzing dataset... Please wait.</span>
            </div>
        {/if}

        <!-- Padding Factor -->
        {#if datasetLoaded}
            <div class="form-control">
                <label
                    for="paddingFactor"
                    class="label font-medium text-base-content/80"
                >
                    <span>Padding Factor</span>
                    <span class="badge badge-neutral font-mono">
                        {((paddingFactor - 1) * 100).toFixed(0)}% {paddingFactor >
                        1
                            ? "larger"
                            : paddingFactor < 1
                              ? "smaller"
                              : "original"}
                    </span>
                </label>
                <div class="flex items-center gap-4 px-1">
                    <input
                        type="range"
                        id="paddingFactor"
                        bind:value={paddingFactor}
                        min="0.5"
                        max="2.0"
                        step="0.1"
                        class="range range-primary range-sm flex-1"
                    />
                    <input
                        type="number"
                        bind:value={paddingFactor}
                        min="0.5"
                        max="2.0"
                        step="0.1"
                        class="input input-bordered input-sm w-20 text-center font-mono"
                    />
                </div>
                <div
                    class="w-full flex justify-between text-xs px-1 text-base-content/50 mt-1"
                >
                    <span>0.5x</span>
                    <span>1.0x</span>
                    <span>2.0x</span>
                </div>
            </div>
        {/if}
    </div>
</div>

<!-- Run Button Area -->
<div class="mt-8 pt-6 border-t border-base-200">
    <div class="flex justify-between items-center">
        <div class="text-sm text-base-content/60">
            {#if datasetLoaded}
                <span class="font-bold text-base-content"
                    >{datasetSummary?.total_images || 0}</span
                > images ready to process
            {/if}
        </div>
        <button
            on:click={runProcessing}
            disabled={loading ||
                !sourceDir ||
                !outputDir ||
                !datasetLoaded ||
                !selectedParentLabel ||
                selectedChildLabels.length === 0}
            class="btn btn-primary px-8 shadow-lg shadow-primary/20 hover:scale-[1.02] active:scale-[0.98] transition-all disabled:opacity-50 disabled:shadow-none"
        >
            {#if loading}
                <span class="loading loading-spinner loading-sm"></span>
                Processing...
            {:else}
                <span class="material-symbols-rounded">play_circle</span>
                Start Processing
            {/if}
        </button>
    </div>

    <!-- Validation Warning (Toast style) -->
    {#if datasetLoaded && validateSelection()}
        <div
            class="alert alert-warning mt-4 rounded-xl border border-warning/20 bg-warning/5 text-warning-content shadow-sm flex items-start gap-3"
        >
            <span class="material-symbols-rounded text-xl mt-0.5">warning</span>
            <div class="text-sm font-medium">{validateSelection()}</div>
        </div>
    {/if}
</div>

<!-- Status Messages -->
<div class="space-y-3 mt-6">
    {#if successMessage}
        <div class="alert alert-success shadow-sm">
            <span class="material-symbols-rounded">check_circle</span>
            <div>
                <h3 class="font-bold">Success!</h3>
                <div class="text-xs">{successMessage}</div>
            </div>
        </div>
    {/if}
    {#if errorMessage}
        <div class="alert alert-error shadow-sm">
            <span class="material-symbols-rounded">error</span>
            <div>
                <h3 class="font-bold">Error!</h3>
                <div class="text-xs">{errorMessage}</div>
            </div>
        </div>
    {/if}
</div>

<!-- Preview Modal -->
{#if showPreviewModal && previewModalImage}
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
        role="dialog"
        aria-modal="true"
        aria-labelledby="preview-modal-title"
        on:click={closePreviewModal}
        on:keydown={(e) => {
            if (e.key === "Escape") closePreviewModal();
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
            <div
                class="p-4 border-b border-gray-200 flex justify-between items-center"
            >
                <h3
                    id="preview-modal-title"
                    class="text-lg font-medium text-gray-900"
                >
                    {previewModalImage.name}
                </h3>
                <button
                    on:click={closePreviewModal}
                    class="text-gray-400 hover:text-gray-600 text-2xl leading-none"
                >
                    ×
                </button>
            </div>
            <div class="p-4">
                <!-- Use KonvaViewer component for advanced annotation viewing -->
                <KonvaViewer
                    imageData={previewModalImage}
                    showModal={true}
                    on:close={closePreviewModal}
                />
            </div>
        </div>
    </div>
{/if}

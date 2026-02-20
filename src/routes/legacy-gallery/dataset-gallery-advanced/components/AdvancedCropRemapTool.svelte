<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { open } from '@tauri-apps/plugin-dialog';
    import { appDataDir } from '@tauri-apps/api/path';
    import { onMount } from 'svelte';
    import { createEventDispatcher } from 'svelte';
    import KonvaViewer from './KonvaViewer.svelte';
    import type { KonvaImageData } from './konvaService';

    // Props from parent (dataset-gallery-advanced)
    export let currentDirectory: string = ''; // Current gallery directory
    export let cropToolOpen: boolean = false; // Accordion open state

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

    // Handle keyboard events for modal
    function handleKeydown(event: KeyboardEvent) {
        if (event.key === "Escape" && showPreviewModal) {
            closePreviewModal();
        }
    }

    async function selectDirectory(type: 'source' | 'output') {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: `Select ${type === 'source' ? 'Source' : 'Output'} Directory`,
            });

            if (selected && typeof selected === 'string') {
                if (type === 'source') {
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
        const commonParents = ['person', 'people', 'human', 'worker', 'individual'];

        for (const parent of commonParents) {
            if (labels.includes(parent)) {
                return parent;
            }
        }

        // Fallback to most common label (by count)
        if (datasetSummary?.label_counts) {
            const sortedLabels = Object.entries(datasetSummary.label_counts)
                .sort(([,a], [,b]) => (b as number) - (a as number))
                .map(([label]) => label);
            return sortedLabels[0] || 'person';
        }

        return labels[0] || 'person';
    }

    function suggestChildLabels(labels: string[]): string[] {
        const safetyEquipment = [
            'safety_helmet', 'helmet', 'hard_hat',
            'reflective_vest', 'vest', 'safety_vest',
            'body_harness', 'harness', 'safety_harness',
            'gloves', 'safety_gloves',
            'boots', 'safety_boots'
        ];

        return labels.filter(label =>
            safetyEquipment.some(safety =>
                label.toLowerCase().includes(safety.toLowerCase())
            )
        );
    }

    async function analyzeDataset() {
        if (!sourceDir) return;

        try {
            analyzing = true;
            errorMessage = null;

            console.log("Analyzing dataset:", sourceDir);
            const result = await invoke("get_labelme_summary", { path: sourceDir as string });
            datasetSummary = JSON.parse(result);

            console.log("Dataset summary:", datasetSummary);

            // Extract available labels
            availableLabels = Object.keys(datasetSummary.label_counts || {});

            if (availableLabels.length === 0) {
                errorMessage = "No labels found in the dataset. Please check if the directory contains LabelMe JSON files.";
                return;
            }

            // Set smart defaults
            selectedParentLabel = suggestParentLabel(availableLabels);
            selectedChildLabels = suggestChildLabels(availableLabels);

            datasetLoaded = true;

            console.log("Dataset analysis complete, triggering auto-preview...");
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
            const result = await invoke("generate_annotated_previews", {
                sourceDir: sourceDir as string,
                numPreviews: 5,
                tempDir: tempDir
            }) as string;

            const data = JSON.parse(result);

            if (data.annotated_images && data.annotated_images.length > 0) {
                console.log(`Loaded ${data.annotated_images.length} images with annotation data`);

                // Convert file paths to proper Tauri URLs and prepare preview data
                const selectedImages: KonvaImageData[] = data.annotated_images.map((imageData: any, index: number) => ({
                    id: `preview_${index}`,
                    path: imageData.path,
                    previewUrl: convertFileSrc(imageData.path), // Load original clean image
                    name: `Preview ${index + 1}`,
                    annotations: imageData.annotations || [] // Draw annotations with KonvaJS
                }));

                previewImages = selectedImages;
                console.log(`Auto-preview ready with ${selectedImages.length} annotated images`);
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
        console.log('Opening preview modal for image:', image);
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
                paddingFactor: paddingFactor
            });

            successMessage = String(message);

            // Dispatch completion event to parent (gallery) with output directory
            dispatch('cropCompleted', { outputDir: outputDir });

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
</script>

<svelte:window on:keydown={handleKeydown} on:resize={handleResize} />

<div class="advanced-crop-remap-tool">
    <div class="space-y-4">
        <!-- Source Directory -->
        <div>
            <label for="sourceDirectoryInput" class="block text-sm font-medium text-gray-700 mb-1">Source Directory</label>
            <div class="flex items-center gap-2">
                <input
                    id="sourceDirectoryInput"
                    type="text"
                    readonly
                    placeholder="Select source directory..."
                    value={sourceDir || ''}
                    class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-600 text-sm truncate"
                />
                <button
                    on:click={() => selectDirectory('source')}
                    class="px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md border border-gray-300 text-sm"
                    disabled={analyzing}
                >
                    Browse...
                </button>
            </div>
            {#if currentDirectory && !sourceDir}
                <p class="text-xs text-indigo-600 mt-1">💡 Using current gallery directory: {currentDirectory}</p>
            {/if}
        </div>

        <!-- Dataset Summary -->
        {#if datasetSummary}
            <div class="bg-blue-50 p-4 rounded-md border border-blue-200">
                <h4 class="font-medium text-blue-900 mb-2">📊 Dataset Summary</h4>
                <div class="grid grid-cols-2 gap-4 text-sm">
                    <div>
                        <span class="font-medium">Total Images:</span> {datasetSummary.total_images}
                    </div>
                    <div>
                        <span class="font-medium">With Annotations:</span> {datasetSummary.images_with_annotations}
                    </div>
                    <div>
                        <span class="font-medium">Total Annotations:</span> {datasetSummary.total_annotations}
                    </div>
                    <div>
                        <span class="font-medium">Unique Labels:</span> {datasetSummary.unique_labels}
                    </div>
                </div>
            </div>
        {/if}

        <!-- Auto Preview Section -->
        {#if datasetLoaded}
            <div class="bg-purple-50 p-4 rounded-md border border-purple-200">
                <h4 class="font-medium text-purple-900 mb-3 flex items-center">
                    <span class="mr-2">👁️</span>
                    Dataset Preview
                    {#if previewLoading}
                        <div class="ml-2 animate-spin h-4 w-4 border-2 border-purple-600 border-t-transparent rounded-full"></div>
                    {/if}
                </h4>

                {#if previewLoading}
                    <div class="text-sm text-purple-700">Loading preview images...</div>
                {:else if previewImages.length > 0}
                    <div class="text-sm text-purple-700 mb-3">
                        Here are {previewImages.length} random annotated images from your dataset with annotations already drawn on them:
                    </div>
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-3">
                        {#each previewImages as image, index (image.path)}
                            <div class="bg-white rounded-lg shadow-sm overflow-hidden border border-purple-200">
                                <div class="relative pb-[75%]">
                                    <button
                                        type="button"
                                        class="absolute inset-0 w-full h-full focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2"
                                        on:click={() => openPreviewModal(image)}
                                        aria-label={`View full size image: ${image.name}`}
                                    >
                                        <img
                                            src={image.previewUrl}
                                            alt={image.name}
                                            class="w-full h-full object-cover hover:opacity-90 transition-opacity"
                                        />
                                    </button>
                                    <div class="absolute top-1 right-1 bg-purple-600 text-white text-xs px-2 py-0.5 rounded-full">
                                        Preview
                                    </div>
                                </div>
                                <div class="p-2">
                                    <p class="text-xs text-gray-600 truncate" title={image.name}>
                                        {image.name}
                                    </p>
                                </div>
                            </div>
                        {/each}
                    </div>
                    <div class="text-xs text-purple-600 mt-2">
                        Click any image to view full size (annotations are already drawn on the images)
                    </div>
                {:else}
                    <div class="text-sm text-purple-700">No annotated images found for preview</div>
                {/if}
            </div>
        {/if}

        <!-- Output Directory -->
        <div>
            <label for="outputDirectoryInput" class="block text-sm font-medium text-gray-700 mb-1">Output Directory</label>
            <div class="flex items-center gap-2">
                <input
                    id="outputDirectoryInput"
                    type="text"
                    readonly
                    placeholder="Select output directory..."
                    value={outputDir || ''}
                    class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-600 text-sm truncate"
                />
                <button
                    on:click={() => selectDirectory('output')}
                    class="px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md border border-gray-300 text-sm"
                >
                    Browse...
                </button>
            </div>
        </div>

        <!-- Dynamic Parent Label Selection -->
        {#if datasetLoaded && availableLabels.length > 0}
            <div>
                <label for="parentLabelSelect" class="block text-sm font-medium text-gray-700 mb-1">Parent Label</label>
                <select
                    id="parentLabelSelect"
                    bind:value={selectedParentLabel}
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500 text-sm"
                >
                    {#each availableLabels as label}
                        <option value={label}>
                            {label} ({datasetSummary?.label_counts[label] || 0} annotations)
                        </option>
                    {/each}
                </select>
                <p class="text-xs text-gray-500 mt-1">The label of the object to crop around.</p>
            </div>

            <!-- Dynamic Child Labels Selection -->
            <div>
                <div class="block text-sm font-medium text-gray-700 mb-2">Required Child Labels</div>
                <div class="max-h-48 overflow-y-auto space-y-2 p-3 bg-gray-50 rounded-md border">
                    {#each getFilteredChildLabels() as label}
                        <div class="flex items-center">
                            <input
                                type="checkbox"
                                bind:group={selectedChildLabels}
                                value={label}
                                id="childLabel_{label}"
                                class="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                            />
                            <label for="childLabel_{label}" class="ml-2 text-sm text-gray-700 cursor-pointer">
                                {label} ({datasetSummary?.label_counts[label] || 0} annotations)
                            </label>
                        </div>
                    {/each}
                </div>
                <p class="text-xs text-gray-500 mt-1">
                    Only people wearing at least one of the selected items will be processed.
                    {#if selectedChildLabels.length > 0}
                        <br><strong>Selected:</strong> {selectedChildLabels.join(", ")}
                    {:else}
                        <br><strong>Selected:</strong> None
                    {/if}
                </p>
            </div>
        {/if}

        <!-- Analysis Loading State -->
        {#if !datasetLoaded && sourceDir}
            <div class="bg-yellow-50 p-3 rounded-md border border-yellow-200">
                <p class="text-sm text-yellow-800">
                    📋 Analyzing dataset... Please wait for analysis to complete.
                </p>
            </div>
        {/if}

        <!-- Padding Factor -->
        {#if datasetLoaded}
            <div>
                <label for="paddingFactor" class="block text-sm font-medium text-gray-700 mb-1">
                    Padding Factor
                    <span class="text-indigo-600 font-medium">({((paddingFactor - 1) * 100).toFixed(0)}% {paddingFactor > 1 ? 'larger' : paddingFactor < 1 ? 'smaller' : 'original'})</span>
                </label>
                <div class="flex items-center gap-3">
                    <input
                        type="range"
                        id="paddingFactor"
                        bind:value={paddingFactor}
                        min="0.5"
                        max="2.0"
                        step="0.1"
                        class="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
                    />
                    <input
                        type="number"
                        bind:value={paddingFactor}
                        min="0.5"
                        max="2.0"
                        step="0.1"
                        class="w-20 px-2 py-1 border border-gray-300 rounded text-sm text-center"
                    />
                </div>
                <div class="flex justify-between text-xs text-gray-500 mt-1">
                    <span>0.5x (50% smaller)</span>
                    <span>1.0x (original size)</span>
                    <span>2.0x (100% larger)</span>
                </div>
                <p class="text-xs text-gray-500 mt-1">
                    Controls how much larger the crop area should be compared to the parent bounding box.
                    Larger values provide more context but may include unwanted background.
                </p>
            </div>
        {/if}
    </div>

    <!-- Run Button -->
    <div class="mt-6">
        <button
            on:click={runProcessing}
            disabled={loading || !sourceDir || !outputDir || !datasetLoaded || !selectedParentLabel || selectedChildLabels.length === 0}
            class="w-full inline-flex justify-center items-center px-6 py-3 border border-transparent text-base font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
            {#if loading}
                <div class="mr-2 animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"></div>
                Processing...
            {:else}
                Run Crop & Remap
            {/if}
        </button>

        <!-- Validation Warning -->
        {#if datasetLoaded}
            {#if validateSelection()}
                <p class="text-sm text-red-600 mt-2">⚠️ {validateSelection()}</p>
            {/if}
        {/if}
    </div>

    <!-- Status Messages -->
    <div class="space-y-3 mt-4">
        {#if successMessage}
            <div class="bg-green-50 text-green-700 p-3 rounded-md text-sm">
                <p class="font-medium">Success!</p>
                <p>{successMessage}</p>
            </div>
        {/if}
        {#if errorMessage}
            <div class="bg-red-50 text-red-700 p-3 rounded-md text-sm">
                <p class="font-medium">Error!</p>
                <p>{errorMessage}</p>
            </div>
        {/if}
    </div>
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
            if (e.key === 'Escape') closePreviewModal();
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
            <div class="p-4 border-b border-gray-200 flex justify-between items-center">
                <h3 id="preview-modal-title" class="text-lg font-medium text-gray-900">{previewModalImage.name}</h3>
                <button
                    on:click={closePreviewModal}
                    class="text-gray-400 hover:text-gray-600 text-2xl leading-none"
                >
                    ×
                </button>
            </div>
            <div class="p-4">
                <!-- Use KonvaViewer component for advanced annotation viewing -->
                <KonvaViewer {previewModalImage} on:close={closePreviewModal} />
            </div>
        </div>
    </div>
{/if}

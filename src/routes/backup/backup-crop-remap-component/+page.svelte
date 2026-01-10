<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { open } from '@tauri-apps/plugin-dialog';
    import { appDataDir } from '@tauri-apps/api/path';

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
    let previewModalImage: any = null;

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

            if (data.success && data.previews && data.previews.length > 0) {
                console.log(`Generated ${data.preview_count} annotated preview images`);

                // Convert file paths to proper Tauri URLs and prepare preview data
                const selectedImages = data.previews.map((path: string, index: number) => ({
                    id: `preview_${index}`,
                    path: path,
                    previewUrl: convertFileSrc(path),
                    name: `Preview ${index + 1}`,
                    annotations: [] // Annotations are already baked into the image
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
    function openPreviewModal(image: any) {
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

    function validateSelection(): string | null {
        if (!selectedParentLabel) return "Please select a parent label";
        if (selectedChildLabels.length === 0) return "Please select at least one child label";
        
        const parentCount = datasetSummary?.label_counts[selectedParentLabel] || 0;
        if (parentCount === 0) return `No instances of '${selectedParentLabel}' found in dataset`;
        
        return null;
    }

    function getFilteredChildLabels(): string[] {
        return availableLabels.filter(label => label !== selectedParentLabel);
    }

    async function runProcessing() {
        if (!sourceDir || !outputDir) {
            errorMessage = "Please select source directory and output directory.";
            return;
        }

        if (!datasetLoaded) {
            errorMessage = "Please analyze the dataset first.";
            return;
        }

        const validationError = validateSelection();
        if (validationError) {
            errorMessage = validationError;
            return;
        }

        loading = true;
        successMessage = null;
        errorMessage = null;

        try {
            const message = await invoke("crop_and_remap_annotations", {
                sourceDir: sourceDir,
                outputDir: outputDir,
                parentLabel: selectedParentLabel,
                requiredChildLabelsStr: selectedChildLabels.join(","),
                paddingFactor: paddingFactor
            });
            successMessage = String(message);
        } catch (err) {
            console.error("Error running processing:", err);
            errorMessage = `Processing failed: ${err instanceof Error ? err.message : String(err)}`;
        } finally {
            loading = false;
        }
    }

</script>

<svelte:head>
    <title>Crop & Remap Annotations</title>
    <meta name="description" content="Advanced crop tool with dynamic multi-label safety equipment detection." />
</svelte:head>

<svelte:window on:keydown={handleKeydown} on:resize={handleResize} />

<div class="container mx-auto px-4 py-8">
    <div class="max-w-2xl mx-auto bg-white p-6 rounded-lg shadow-md">
        <h1 class="text-2xl font-bold text-gray-800 mb-6">Crop and Remap Tool</h1>
        <p class="text-sm text-gray-600 mb-6">Advanced annotation processing with dynamic multi-label detection based on your dataset</p>

        <div class="space-y-4 mb-6">
            <!-- Source Directory -->
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Source Directory</label>
                <div class="flex items-center gap-2">
                    <input 
                        type="text" 
                        readonly 
                        placeholder="Select source directory..." 
                        value={sourceDir || ''}
                        class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-600 text-sm truncate"
                    />
                    <button 
                        on:click={() => selectDirectory('source')}
                        class="px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md border border-gray-300 text-sm"
                    >
                        Browse...
                    </button>
                </div>
            </div>

            <!-- Dataset Analysis -->
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Dataset Analysis</label>
                <div class="flex items-center gap-2">
                    <button 
                        on:click={analyzeDataset}
                        disabled={!sourceDir || analyzing}
                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-md text-sm flex items-center"
                    >
                        {#if analyzing}
                            <div class="mr-2 animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"></div>
                            Analyzing...
                        {:else}
                            Analyze Dataset
                        {/if}
                    </button>
                    {#if datasetLoaded}
                        <span class="text-sm text-green-600 flex items-center">
                            <svg class="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
                            </svg>
                            Dataset analyzed
                        </span>
                    {/if}
                </div>
                <p class="text-xs text-gray-500 mt-1">Analyze your dataset to discover available labels and get smart suggestions.</p>
            </div>

            <!-- Dataset Summary -->
            {#if datasetSummary}
                <div class="bg-blue-50 p-3 rounded-md border border-blue-200">
                    <h4 class="font-medium text-blue-900 mb-1">Dataset Summary</h4>
                    <div class="text-sm text-blue-700 space-y-1">
                        <p>📁 {datasetSummary.total_images} images</p>
                        <p>🏷️ {datasetSummary.total_annotations} annotations</p>
                        <p>🎯 {datasetSummary.unique_labels} unique labels</p>
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
                <label class="block text-sm font-medium text-gray-700 mb-1">Output Directory</label>
                <div class="flex items-center gap-2">
                    <input 
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
                    <label class="block text-sm font-medium text-gray-700 mb-1">Parent Label</label>
                    <select 
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
                    <label class="block text-sm font-medium text-gray-700 mb-2">Required Child Labels</label>
                    <div class="max-h-48 overflow-y-auto space-y-2 p-3 bg-gray-50 rounded-md border">
                        {#each getFilteredChildLabels() as label}
                            <label class="flex items-center">
                                <input 
                                    type="checkbox" 
                                    bind:group={selectedChildLabels}
                                    value={label}
                                    class="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                                />
                                <span class="ml-2 text-sm text-gray-700">
                                    {label} ({datasetSummary?.label_counts[label] || 0} annotations)
                                </span>
                            </label>
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
            {:else if !datasetLoaded}
                <div class="bg-yellow-50 p-3 rounded-md border border-yellow-200">
                    <p class="text-sm text-yellow-800">
                        📋 Please analyze your dataset first to see available labels and configure processing options.
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
        <div class="mb-6">
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
        <div class="space-y-3">
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
</div>

<!-- Preview Modal -->
{#if showPreviewModal && previewModalImage}
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
        <div
            class="bg-white rounded-lg shadow-xl max-w-4xl max-h-[90vh] overflow-hidden"
            role="document"
            on:click|stopPropagation
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
                <div class="relative max-w-full max-h-[70vh] overflow-hidden rounded-lg">
                    <img
                        src={previewModalImage.previewUrl}
                        alt={previewModalImage.name}
                        class="max-w-full max-h-full object-contain block"
                    />
                    <!-- Preview badge -->
                    <div class="absolute top-2 right-2 bg-purple-600 text-white text-sm px-3 py-1 rounded-full shadow z-10">
                        Annotated Preview
                    </div>
                </div>
                <div class="mt-4 text-sm text-gray-600 text-center">
                    This preview shows the image with annotations already drawn on it
                </div>
            </div>
        </div>
    </div>
{/if}
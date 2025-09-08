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

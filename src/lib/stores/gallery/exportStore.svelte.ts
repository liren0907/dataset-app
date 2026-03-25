
import { browser } from "$app/environment";
import { performDatasetExport } from "$lib/services/gallery/datasetService";
import type { DatasetExportParams } from "$lib/services/gallery/datasetService";
import { imageState, getDirectoryPath, loadFromPath } from "./imageStore.svelte";

export interface CroppedDataset {
    tempPath: string;  // Temp location where cropped files are stored
    exportedPath?: string;  // Final exported location (optional, set after export)
    imageCount: number;
    parentLabel: string;
    childLabels: string[];
    createdAt: Date;
}

export interface ExportState {
    showActualExportModal: boolean;
    pageExportLoading: boolean;
    pageExportError: string;
    pageExportSuccess: string;
    showCropTool: boolean;
    showAdvancedCropTool: boolean;
    showHierarchicalCrop: boolean;
    showExtractModal: boolean;
    extractLoading: boolean;
    extractError: string;
    extractSuccess: string;
    // Hierarchical crop-remap state
    croppedDatasets: CroppedDataset[];
    cropModalParentLabel: string; // Pre-selected parent label for crop modal
    activeCroppedDatasetPath: string | null; // Currently active cropped dataset for gallery view
    originalDirectoryPath: string; // Original directory to switch back to
    // Background crop processing
    cropProcessing: boolean;
    cropProgressMessage: string;
    cropStartTime: number | null;
    cropProgressCurrent: number;
    cropProgressTotal: number;
    // Dynamic property for export
    currentExportDataset?: any;
}

const CROPPED_DATASETS_STORAGE_KEY = "croppedDatasets";

function loadCroppedDatasets(): CroppedDataset[] {
    if (!browser) return [];
    try {
        const raw = localStorage.getItem(CROPPED_DATASETS_STORAGE_KEY);
        if (!raw) return [];
        const parsed = JSON.parse(raw);
        if (!Array.isArray(parsed)) return [];

        // Deduplicate by tempPath immediately on load
        const uniqueParams = new Map();
        parsed.forEach(item => {
            if (item.tempPath && !uniqueParams.has(item.tempPath)) {
                uniqueParams.set(item.tempPath, item);
            }
        });

        return Array.from(uniqueParams.values()).map((item: any) => ({
            ...item,
            createdAt: item.createdAt ? new Date(item.createdAt) : new Date(),
        })) as CroppedDataset[];
    } catch (err) {
        console.warn("Failed to load cropped datasets from storage:", err);
        return [];
    }
}

function persistCroppedDatasets(datasets: CroppedDataset[]): void {
    if (!browser) return;
    try {
        const payload = datasets.map((d) => ({
            ...d,
            createdAt: d.createdAt instanceof Date
                ? d.createdAt.toISOString()
                : new Date(d.createdAt).toISOString(),
        }));
        localStorage.setItem(
            CROPPED_DATASETS_STORAGE_KEY,
            JSON.stringify(payload)
        );
    } catch (err) {
        console.warn("Failed to persist cropped datasets:", err);
    }
}

// Validate that temp paths still exist, filter out invalid ones
async function validateAndFilterCroppedDatasets(datasets: CroppedDataset[]): Promise<CroppedDataset[]> {
    if (datasets.length === 0) return [];

    try {
        const { invoke } = await import("@tauri-apps/api/core");
        const validDatasets: CroppedDataset[] = [];

        for (const dataset of datasets) {
            const exists = await invoke<boolean>("path_exists", { path: dataset.tempPath });
            if (exists) {
                validDatasets.push(dataset);
            } else {
                console.log(`Removing invalid cropped dataset: ${dataset.tempPath}`);
            }
        }

        // Persist the filtered list
        if (validDatasets.length !== datasets.length) {
            persistCroppedDatasets(validDatasets);
        }

        return validDatasets;
    } catch (err) {
        console.warn("Failed to validate cropped datasets:", err);
        return datasets; // Return original on error
    }
}

export const exportState: ExportState = $state({
    showActualExportModal: false,
    pageExportLoading: false,
    pageExportError: "",
    pageExportSuccess: "",
    showCropTool: false,
    showAdvancedCropTool: false,
    showHierarchicalCrop: false,
    showExtractModal: false,
    extractLoading: false,
    extractError: "",
    extractSuccess: "",
    // Hierarchical crop-remap state
    croppedDatasets: loadCroppedDatasets(),
    cropModalParentLabel: "",
    activeCroppedDatasetPath: null,
    originalDirectoryPath: "",
    // Background crop processing
    cropProcessing: false,
    cropProgressMessage: "",
    cropStartTime: null,
    cropProgressCurrent: 0,
    cropProgressTotal: 0,
});

export async function runUnifiedExport(exportDetails: any) {
    const { sourceDir, outputDir, includedLabels } = exportDetails;
    if (!sourceDir || !outputDir || !includedLabels?.length) {
        exportState.pageExportError = "Missing required export parameters.";
        return;
    }

    exportState.pageExportLoading = true;
    exportState.pageExportError = "";
    exportState.pageExportSuccess = "";

    try {
        const params: DatasetExportParams = { ...exportDetails };
        const resultMessage = await performDatasetExport(params);
        exportState.pageExportSuccess = resultMessage;
        exportState.showActualExportModal = false;
    } catch (err: any) {
        console.error("Export error:", err);
        exportState.pageExportError = `Failed to export: ${err.message}`;
    } finally {
        exportState.pageExportLoading = false;
    }
}

export async function handleCropCompleted(tempPath: string, details?: { imageCount?: number; parentLabel?: string; childLabels?: string[] }) {
    // Create a cropped dataset entry
    const croppedDataset: CroppedDataset = {
        tempPath: tempPath,
        imageCount: details?.imageCount || 0,
        parentLabel: details?.parentLabel || "",
        childLabels: details?.childLabels || [],
        createdAt: new Date()
    };

    exportState.showCropTool = false;
    exportState.showAdvancedCropTool = false;
    // Note: NOT closing showHierarchicalCrop so user can see success message
    exportState.cropModalParentLabel = "";

    if (!exportState.croppedDatasets.some(d => d.tempPath === tempPath)) {
        exportState.croppedDatasets = [...exportState.croppedDatasets, croppedDataset];
    }
    persistCroppedDatasets(exportState.croppedDatasets);
}

// Open cropped dataset in gallery (navigate to the output directory)
export async function openCroppedDatasetInGallery(outputPath: string) {
    // Save original directory before switching
    exportState.originalDirectoryPath = exportState.originalDirectoryPath || getDirectoryPath();
    exportState.activeCroppedDatasetPath = outputPath;
    // Load the cropped dataset
    await loadFromPath(outputPath);
}

export async function handleExtractLabels(details: { sourceDir: string, outputDir: string, includedLabels: string[] }) {
    if (!details.sourceDir || !details.outputDir || details.includedLabels.length === 0) {
        exportState.extractError = "Missing parameters.";
        return;
    }

    exportState.extractLoading = true;
    exportState.extractError = "";
    exportState.extractSuccess = "";

    try {
        const params: DatasetExportParams = {
            sourceDir: details.sourceDir,
            outputDir: details.outputDir,
            mode: "labelme",
            includedLabels: details.includedLabels
        };
        const msg = await performDatasetExport(params);
        exportState.extractSuccess = msg;
        setTimeout(() => {
            exportState.showExtractModal = false;
            exportState.extractSuccess = "";
        }, 2000);
    } catch (err: any) {
        exportState.extractError = `Extraction failed: ${err.message}`;
    } finally {
        exportState.extractLoading = false;
    }
}

// Open crop modal with a pre-selected parent label (triggered from DatasetSummary)
export function openCropModalWithLabel(label: string) {
    exportState.cropModalParentLabel = label;
    exportState.showAdvancedCropTool = true;
}

// Close crop modal and reset pre-selected label
export function closeCropModal() {
    exportState.cropModalParentLabel = "";
    exportState.showAdvancedCropTool = false;
}

// Add a completed crop result to the list
export function addCroppedDataset(dataset: CroppedDataset) {
    exportState.croppedDatasets = [...exportState.croppedDatasets, dataset];
    persistCroppedDatasets(exportState.croppedDatasets);
}

// Remove a cropped dataset from the list
export function removeCroppedDataset(tempPath: string) {
    exportState.croppedDatasets = exportState.croppedDatasets.filter(d => d.tempPath !== tempPath);
    // If removing the active dataset, switch back to original
    if (exportState.activeCroppedDatasetPath === tempPath) {
        exportState.activeCroppedDatasetPath = null;
    }
    persistCroppedDatasets(exportState.croppedDatasets);
}

// Set active cropped dataset for gallery view
export function setActiveCroppedDataset(outputPath: string) {
    exportState.activeCroppedDatasetPath = outputPath;
    // Load the cropped dataset into imageState
    loadFromPath(outputPath);
}

// Switch back to original dataset
export function switchToOriginal(originalPath: string) {
    exportState.activeCroppedDatasetPath = null;
    // Load original dataset back
    loadFromPath(originalPath);
}

// Export cropped dataset from temp to user-selected destination
export async function exportCroppedDataset(tempPath: string, destPath: string) {
    try {
        // Use Tauri invoke to copy directory
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("copy_directory", { source: tempPath, destination: destPath });

        // Update the dataset with exported path
        exportState.croppedDatasets = exportState.croppedDatasets.map(d =>
            d.tempPath === tempPath ? { ...d, exportedPath: destPath } : d
        );
        persistCroppedDatasets(exportState.croppedDatasets);

        return destPath;
    } catch (err: any) {
        throw new Error(`Failed to export: ${err.message || String(err)}`);
    }
}

// Clear all cropped datasets from memory and localStorage
export function clearAllCroppedDatasets() {
    persistCroppedDatasets([]);
    exportState.croppedDatasets = [];
    exportState.activeCroppedDatasetPath = null;
}

// Validate and filter datasets (call on init)
export async function validateCroppedDatasets() {
    const currentDatasets = loadCroppedDatasets();
    const validDatasets = await validateAndFilterCroppedDatasets(currentDatasets);
    exportState.croppedDatasets = validDatasets;
}

// Run crop in background without blocking UI (Event-based)
export async function runCropInBackground(params: {
    sourceDir: string;
    parentLabel: string;
    childLabels: string[];
    paddingFactor: number;
}) {
    const { sourceDir, parentLabel, childLabels, paddingFactor } = params;

    // Import modules
    const { invoke } = await import("@tauri-apps/api/core");
    const { listen } = await import("@tauri-apps/api/event");
    const { appDataDir } = await import("@tauri-apps/api/path");
    const { toastStore } = await import("$lib/stores/toastStore.svelte");

    // Close panel and set processing state
    const startTime = Date.now();
    exportState.showHierarchicalCrop = false;
    exportState.cropProcessing = true;
    exportState.cropProgressMessage = `Starting crop for "${parentLabel}"...`;
    exportState.cropStartTime = startTime;
    exportState.cropProgressCurrent = 0;
    exportState.cropProgressTotal = 0;

    // Generate temp output directory path
    try {
        const appData = await appDataDir();
        const timestamp = Date.now();
        const tempOutputDir = `${appData}cropped/${timestamp}_${parentLabel}`;

        let unlistenProgress: () => void;
        let unlistenComplete: () => void;
        let unlistenError: () => void;

        // Cleanup function
        const cleanup = () => {
            if (unlistenProgress) unlistenProgress();
            if (unlistenComplete) unlistenComplete();
            if (unlistenError) unlistenError();
        };

        // Listen for progress
        unlistenProgress = await listen<any>("crop-progress", (event) => {
            exportState.cropProgressMessage = event.payload.message || "Processing...";
            exportState.cropProgressCurrent = event.payload.current || 0;
            exportState.cropProgressTotal = event.payload.total || 0;
        });

        // Listen for completion
        unlistenComplete = await listen<any>("crop-complete", (event) => {
            const elapsedSeconds = ((Date.now() - startTime) / 1000).toFixed(1);
            const { imageCount, tempPath } = event.payload;

            // Create cropped dataset entry
            const croppedDataset: CroppedDataset = {
                tempPath: tempPath, // from event payload
                imageCount,
                parentLabel,
                childLabels,
                createdAt: new Date()
            };

            exportState.cropProcessing = false;
            exportState.cropProgressMessage = "";
            exportState.cropStartTime = null;

            if (!exportState.croppedDatasets.some(d => d.tempPath === tempPath)) {
                exportState.croppedDatasets = [...exportState.croppedDatasets, croppedDataset];
            }
            persistCroppedDatasets(exportState.croppedDatasets);

            toastStore.show(
                `Cropped ${imageCount} images from "${parentLabel}" in ${elapsedSeconds}s`,
                'success',
                6000
            );

            cleanup();
        });

        // Listen for error
        unlistenError = await listen<any>("crop-error", (event) => {
            exportState.cropProcessing = false;
            exportState.cropProgressMessage = "";
            exportState.cropStartTime = null;

            toastStore.show(
                `Crop failed: ${event.payload.message}`,
                'error',
                8000
            );

            cleanup();
        });

        // Start the background process (this returns immediately now)
        await invoke("crop_and_remap_annotations", {
            sourceDir,
            outputDir: tempOutputDir,
            parentLabel,
            requiredChildLabelsStr: childLabels.join(","),
            paddingFactor,
        });

    } catch (err: any) {
        // Clear processing state on error
        exportState.cropProcessing = false;
        exportState.cropProgressMessage = "";
        exportState.cropStartTime = null;

        // Show error toast
        const { toastStore } = await import("$lib/stores/toastStore.svelte");
        toastStore.show(
            `Failed to start crop: ${err.message || String(err)}`,
            'error',
            8000
        );
    }
}

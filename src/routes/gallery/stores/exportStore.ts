
import { writable } from "svelte/store";
import { browser } from "$app/environment";
import { performDatasetExport } from "../services/datasetService";
import type { DatasetExportParams } from "../services/datasetService";
import { imageStore } from "./imageStore";

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
}

const CROPPED_DATASETS_STORAGE_KEY = "croppedDatasets";

function loadCroppedDatasets(): CroppedDataset[] {
    if (!browser) return [];
    try {
        const raw = localStorage.getItem(CROPPED_DATASETS_STORAGE_KEY);
        if (!raw) return [];
        const parsed = JSON.parse(raw);
        if (!Array.isArray(parsed)) return [];
        return parsed.map((item) => ({
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

const initialState: ExportState = {
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
};

function createExportStore() {
    const { subscribe, set, update } = writable<ExportState>(initialState);

    return {
        subscribe,
        set,
        update,

        runUnifiedExport: async (exportDetails: any) => {
            const { sourceDir, outputDir, includedLabels } = exportDetails;
            if (!sourceDir || !outputDir || !includedLabels?.length) {
                update(s => ({ ...s, pageExportError: "Missing required export parameters." }));
                return;
            }

            update(s => ({ ...s, pageExportLoading: true, pageExportError: "", pageExportSuccess: "" }));

            try {
                const params: DatasetExportParams = { ...exportDetails };
                const resultMessage = await performDatasetExport(params);
                update(s => ({ ...s, pageExportSuccess: resultMessage, showActualExportModal: false }));
            } catch (err: any) {
                console.error("Export error:", err);
                update(s => ({ ...s, pageExportError: `Failed to export: ${err.message}` }));
            } finally {
                update(s => ({ ...s, pageExportLoading: false }));
            }
        },

        handleCropCompleted: async (tempPath: string, details?: { imageCount?: number; parentLabel?: string; childLabels?: string[] }) => {
            // Create a cropped dataset entry
            const croppedDataset: CroppedDataset = {
                tempPath: tempPath,
                imageCount: details?.imageCount || 0,
                parentLabel: details?.parentLabel || "",
                childLabels: details?.childLabels || [],
                createdAt: new Date()
            };

            update(s => {
                const updated = {
                    ...s,
                    showCropTool: false,
                    showAdvancedCropTool: false,
                    showHierarchicalCrop: false,
                    cropModalParentLabel: "",
                    croppedDatasets: [...s.croppedDatasets, croppedDataset]
                };
                persistCroppedDatasets(updated.croppedDatasets);
                return updated;
            });
        },

        // Open cropped dataset in gallery (navigate to the output directory)
        openCroppedDatasetInGallery: async (outputPath: string) => {
            // Save original directory before switching
            update(s => ({
                ...s,
                originalDirectoryPath: s.originalDirectoryPath || imageStore.getDirectoryPath(),
                activeCroppedDatasetPath: outputPath
            }));
            // Load the cropped dataset
            await imageStore.loadFromPath(outputPath);
        },

        handleExtractLabels: async (details: { sourceDir: string, outputDir: string, includedLabels: string[] }) => {
            if (!details.sourceDir || !details.outputDir || details.includedLabels.length === 0) {
                update(s => ({ ...s, extractError: "Missing parameters." }));
                return;
            }

            update(s => ({ ...s, extractLoading: true, extractError: "", extractSuccess: "" }));

            try {
                const params: DatasetExportParams = {
                    sourceDir: details.sourceDir,
                    outputDir: details.outputDir,
                    mode: "labelme",
                    includedLabels: details.includedLabels
                };
                const msg = await performDatasetExport(params);
                update(s => ({ ...s, extractSuccess: msg }));
                setTimeout(() => {
                    update(s => ({ ...s, showExtractModal: false, extractSuccess: "" }));
                }, 2000);
            } catch (err: any) {
                update(s => ({ ...s, extractError: `Extraction failed: ${err.message}` }));
            } finally {
                update(s => ({ ...s, extractLoading: false }));
            }
        },

        // Open crop modal with a pre-selected parent label (triggered from DatasetSummary)
        openCropModalWithLabel: (label: string) => {
            update(s => ({
                ...s,
                cropModalParentLabel: label,
                showAdvancedCropTool: true
            }));
        },

        // Close crop modal and reset pre-selected label
        closeCropModal: () => {
            update(s => ({
                ...s,
                cropModalParentLabel: "",
                showAdvancedCropTool: false
            }));
        },

        // Add a completed crop result to the list
        addCroppedDataset: (dataset: CroppedDataset) => {
            update(s => {
                const updated = {
                    ...s,
                    croppedDatasets: [...s.croppedDatasets, dataset]
                };
                persistCroppedDatasets(updated.croppedDatasets);
                return updated;
            });
        },

        // Remove a cropped dataset from the list
        removeCroppedDataset: (tempPath: string) => {
            update(s => {
                const updated = {
                    ...s,
                    croppedDatasets: s.croppedDatasets.filter(d => d.tempPath !== tempPath),
                    // If removing the active dataset, switch back to original
                    activeCroppedDatasetPath: s.activeCroppedDatasetPath === tempPath ? null : s.activeCroppedDatasetPath
                };
                persistCroppedDatasets(updated.croppedDatasets);
                return updated;
            });
        },

        // Set active cropped dataset for gallery view
        setActiveCroppedDataset: (outputPath: string) => {
            update(s => ({
                ...s,
                activeCroppedDatasetPath: outputPath
            }));
            // Load the cropped dataset into imageStore
            imageStore.loadFromPath(outputPath);
        },

        // Switch back to original dataset
        switchToOriginal: (originalPath: string) => {
            update(s => ({
                ...s,
                activeCroppedDatasetPath: null
            }));
            // Load original dataset back
            imageStore.loadFromPath(originalPath);
        },

        // Export cropped dataset from temp to user-selected destination
        exportCroppedDataset: async (tempPath: string, destPath: string) => {
            try {
                // Use Tauri invoke to copy directory
                const { invoke } = await import("@tauri-apps/api/core");
                await invoke("copy_directory", { source: tempPath, destination: destPath });

                // Update the dataset with exported path
                update(s => {
                    const updatedDatasets = s.croppedDatasets.map(d =>
                        d.tempPath === tempPath ? { ...d, exportedPath: destPath } : d
                    );
                    persistCroppedDatasets(updatedDatasets);
                    return { ...s, croppedDatasets: updatedDatasets };
                });

                return destPath;
            } catch (err: any) {
                throw new Error(`Failed to export: ${err.message || String(err)}`);
            }
        }
    };
}

export const exportStore = createExportStore();

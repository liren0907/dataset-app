
import { writable } from "svelte/store";
import { browser } from "$app/environment";
import { performDatasetExport } from "../services/datasetService";
import type { DatasetExportParams } from "../services/datasetService";
import { imageStore } from "./imageStore";

export interface CroppedDataset {
    outputPath: string;
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
    showExtractModal: boolean;
    extractLoading: boolean;
    extractError: string;
    extractSuccess: string;
    // Hierarchical crop-remap state
    croppedDatasets: CroppedDataset[];
    cropModalParentLabel: string; // Pre-selected parent label for crop modal
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
    showExtractModal: false,
    extractLoading: false,
    extractError: "",
    extractSuccess: "",
    // Hierarchical crop-remap state
    croppedDatasets: loadCroppedDatasets(),
    cropModalParentLabel: "",
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

        handleCropCompleted: async (outputDir: string, details?: { imageCount?: number; parentLabel?: string; childLabels?: string[] }) => {
            // Create a cropped dataset entry
            const croppedDataset: CroppedDataset = {
                outputPath: outputDir,
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
                    cropModalParentLabel: "",
                    croppedDatasets: [...s.croppedDatasets, croppedDataset]
                };
                persistCroppedDatasets(updated.croppedDatasets);
                return updated;
            });
        },

        // Open cropped dataset in gallery (navigate to the output directory)
        openCroppedDatasetInGallery: async (outputPath: string) => {
            // Update Image Store to load the cropped dataset
            imageStore.update(s => ({
                ...s,
                directoryPath: outputPath,
                currentPage: 1,
                images: [],
                datasetSummary: null,
                error: ""
            }));
            await imageStore.loadImagesPage(1);
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
        removeCroppedDataset: (outputPath: string) => {
            update(s => {
                const updated = {
                    ...s,
                    croppedDatasets: s.croppedDatasets.filter(d => d.outputPath !== outputPath)
                };
                persistCroppedDatasets(updated.croppedDatasets);
                return updated;
            });
        }
    };
}

export const exportStore = createExportStore();

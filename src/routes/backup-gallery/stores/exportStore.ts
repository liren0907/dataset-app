
import { writable } from "svelte/store";
import { performDatasetExport } from "../services/datasetService";
import type { DatasetExportParams } from "../services/datasetService";
import { imageStore } from "./imageStore";

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

        handleCropCompleted: async (outputDir: string) => {
            update(s => ({
                ...s,
                showCropTool: false
            }));
            // Update Image Store
            imageStore.update(s => ({
                ...s,
                directoryPath: outputDir,
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
        }
    };
}

export const exportStore = createExportStore();

import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { appDataDir } from '@tauri-apps/api/path';
import { open } from '@tauri-apps/plugin-dialog';

export interface DatasetSummary {
    total_images: number;
    total_annotations: number;
    unique_labels: number;
    label_counts: Record<string, number>;
}

export interface PreviewImage {
    id: string;
    path: string;
    previewUrl: string;
    name: string;
    annotations: any[];
}

export const suggestParentLabel = (labels: string[], summary: DatasetSummary | null): string => {
    // Priority order for common parent labels
    const commonParents = ['person', 'people', 'human', 'worker', 'individual'];

    for (const parent of commonParents) {
        if (labels.includes(parent)) {
            return parent;
        }
    }

    // Fallback to most common label (by count)
    if (summary?.label_counts) {
        const sortedLabels = Object.entries(summary.label_counts)
            .sort(([, a], [, b]) => (b as number) - (a as number))
            .map(([label]) => label);
        return sortedLabels[0] || 'person';
    }

    return labels[0] || 'person';
};

export const suggestChildLabels = (labels: string[]): string[] => {
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
};

export const selectDirectory = async (type: 'source' | 'output'): Promise<string | null> => {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: `Select ${type === 'source' ? 'Source' : 'Output'} Directory`,
        });

        if (selected && typeof selected === 'string') {
            return selected;
        }
        return null;
    } catch (err) {
        console.error("Error selecting directory:", err);
        throw new Error(`Failed to select directory: ${err instanceof Error ? err.message : String(err)}`);
    }
};

export const analyzeDataset = async (sourceDir: string): Promise<DatasetSummary> => {
    if (!sourceDir) throw new Error("No source directory provided");

    console.log("Analyzing dataset:", sourceDir);
    const result = await invoke("get_labelme_summary", { path: sourceDir });
    const summary = JSON.parse(result as string);

    console.log("Dataset summary:", summary);
    return summary;
};

export const autoPreviewAnnotatedImages = async (
    sourceDir: string,
    datasetSummary: any
): Promise<PreviewImage[]> => {
    if (!datasetSummary || !sourceDir) {
        console.log("No dataset available for preview");
        return [];
    }

    console.log("Starting auto-preview of annotated images");

    // Create a temporary directory for previews in app data directory
    const appData = await appDataDir();
    const tempDir = `${appData}previews_${Date.now()}`;

    // Generate annotated preview images using the backend
    const result = await invoke("generate_annotated_previews", {
        sourceDir: sourceDir,
        numPreviews: 5,
        tempDir: tempDir
    }) as string;

    const data = JSON.parse(result);

    if (data.annotated_images && data.annotated_images.length > 0) {
        console.log(`Loaded ${data.annotated_images.length} images with annotation data`);

        // Convert file paths to proper Tauri URLs and prepare preview data
        return data.annotated_images.map((imageData: any, index: number) => ({
            id: `preview_${index}`,
            path: imageData.path,
            previewUrl: convertFileSrc(imageData.path), // Load original clean image
            name: `Preview ${index + 1}`,
            annotations: imageData.annotations || [] // Draw annotations with KonvaJS
        }));
    } else {
        console.log("No annotated preview images were generated");
        return [];
    }
};

export const runProcessing = async (
    sourceDir: string,
    outputDir: string,
    parentLabel: string,
    childLabels: string[],
    paddingFactor: number
): Promise<string> => {
    const message = await invoke("crop_and_remap_annotations", {
        sourceDir: sourceDir,
        outputDir: outputDir,
        parentLabel: parentLabel,
        requiredChildLabelsStr: childLabels.join(","),
        paddingFactor: paddingFactor
    });
    return String(message);
};

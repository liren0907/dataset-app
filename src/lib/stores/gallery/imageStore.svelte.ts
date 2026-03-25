
import { open } from "@tauri-apps/plugin-dialog";
import { tick } from "svelte";
import {
    fetchPaginatedImages,
    fetchDatasetSummary,
} from "$lib/services/gallery/datasetService";
import type {
    ProcessedImage,
    DatasetSummary,
} from "$lib/services/gallery/datasetService";
import {
    MOCK_DIRECTORY_PATH,
    mockGetImages,
} from "$lib/../mocks/mockFileSystem";

export interface ImageState {
    directoryPath: string;
    images: ProcessedImage[];
    loading: boolean;
    error: string;
    datasetSummary: DatasetSummary | null;
    isMockMode: boolean;
    currentPage: number;
    pageSize: number;
    totalImages: number;
    totalPages: number;
    itemsPerPage: number;
}

export const imageState: ImageState = $state({
    directoryPath: "",
    images: [],
    loading: false,
    error: "",
    datasetSummary: null,
    isMockMode: false,
    currentPage: 1,
    pageSize: 30,
    totalImages: 0,
    totalPages: 0,
    itemsPerPage: 30,
});

// Get current directory path
export function getDirectoryPath(): string {
    return imageState.directoryPath;
}

export async function setMockMode(enabled: boolean) {
    imageState.isMockMode = enabled;
    imageState.images = [];
    imageState.directoryPath = "";
    imageState.datasetSummary = null;
    imageState.currentPage = 1;
    imageState.error = "";

    if (imageState.isMockMode) {
        await loadMockData();
    }
}

export async function loadMockData() {
    imageState.directoryPath = MOCK_DIRECTORY_PATH;
    imageState.loading = true;
    imageState.error = "";

    try {
        const mockImages = (await mockGetImages(MOCK_DIRECTORY_PATH)) as any[];

        const processedImages: ProcessedImage[] = mockImages.map((img) => ({
            ...img,
            previewUrl: img.previewUrl,
            assetUrl: img.assetUrl,
            annotated: (img.k || 0) > 0,
            name: img.name || "Mock Image",
            path: img.path || "",
        }));

        const totalAnn = processedImages.reduce((acc, img) => acc + (img.annotated ? 1 : 0), 0);
        const summary: DatasetSummary = {
            total_images: processedImages.length,
            images_with_annotations: totalAnn,
            total_annotations: totalAnn * 3,
            unique_labels: 3,
            label_counts: { opening: 50, crane: 30, liftcar: 20 },
            annotation_types: ["rectangle"],
        };

        imageState.images = processedImages;
        imageState.datasetSummary = summary;
        imageState.totalImages = processedImages.length;
        imageState.totalPages = 1;
        imageState.loading = false;
    } catch (err) {
        console.error("❌ Mock load failed", err);
        imageState.error = "Failed to load mock data.";
        imageState.loading = false;
    }
}

export async function selectDirectory() {
    if (imageState.isMockMode) {
        await loadMockData();
        return;
    }

    imageState.loading = true;
    imageState.error = "";

    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: "Select Image Directory",
        });

        if (selected) {
            const newPath = selected as string;
            if (!newPath.trim()) {
                imageState.error = "Invalid directory path: path is empty";
                imageState.loading = false;
                return;
            }

            imageState.directoryPath = newPath;
            imageState.currentPage = 1;
            imageState.images = [];
            console.log("Selected directory:", newPath);
            await loadImagesPage(1);
        } else {
            imageState.loading = false;
        }
    } catch (err: any) {
        console.error("Error selecting directory:", err);
        imageState.error = `Failed to select directory: ${err.message || String(err)}`;
        imageState.loading = false;
    }
}

export async function loadImagesPage(page: number) {
    if (imageState.isMockMode) {
        await loadMockData();
        return;
    }

    if (!imageState.directoryPath) {
        return;
    }

    imageState.loading = true;
    imageState.error = "";

    try {
        const result = await fetchPaginatedImages(imageState.directoryPath, page, imageState.pageSize);

        imageState.images = result.processedImages;
        imageState.totalImages = result.totalImages;
        imageState.totalPages = result.totalPages;
        imageState.currentPage = page;
        imageState.loading = false;

        if (page === 1) {
            generateLabelMeSummary();
        }

        await tick();
        if (typeof window !== "undefined") {
            window.scrollTo(0, 0);
        }

    } catch (err: any) {
        console.error("Page: Error loading images:", err);
        imageState.error = `Failed to load images: ${err.message || "Unknown error"}`;
        imageState.images = [];
        imageState.totalImages = 0;
        imageState.totalPages = 0;
        imageState.loading = false;
    }
}

export async function generateLabelMeSummary() {
    if (!imageState.directoryPath) return;

    try {
        const summary = await fetchDatasetSummary(imageState.directoryPath);
        imageState.datasetSummary = summary;
    } catch (err: any) {
        console.warn(`Page: Failed to generate LabelMe summary: ${err.message}`);
        imageState.datasetSummary = null;
    }
}

// Load images from a specific directory path (used for switching datasets)
export async function loadFromPath(directoryPath: string) {
    if (!directoryPath.trim()) {
        imageState.error = "Invalid directory path: path is empty";
        return;
    }

    imageState.directoryPath = directoryPath;
    imageState.currentPage = 1;
    imageState.images = [];
    imageState.loading = true;
    imageState.error = "";
    console.log("Loading from path:", directoryPath);
    await loadImagesPage(1);
}

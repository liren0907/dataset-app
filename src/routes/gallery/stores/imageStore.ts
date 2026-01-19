
import { writable, get } from "svelte/store";
import { open } from "@tauri-apps/plugin-dialog";
import { tick } from "svelte";
import {
    fetchPaginatedImages,
    fetchDatasetSummary,
} from "../services/datasetService";
import type {
    ProcessedImage,
    DatasetSummary,
} from "../services/datasetService";
import {
    MOCK_DIRECTORY_PATH,
    mockGetImages,
} from "../../../mocks/mockFileSystem";

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

const initialState: ImageState = {
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
};

function createImageStore() {
    const { subscribe, set, update } = writable<ImageState>(initialState);

    return {
        subscribe,
        set,
        update,

        setMockMode: async (enabled: boolean) => {
            update(s => ({ ...s, isMockMode: enabled, images: [], directoryPath: "", datasetSummary: null, currentPage: 1, error: "" }));
            const state = get({ subscribe });
            if (state.isMockMode) {
                await imageStore.loadMockData();
            }
        },

        loadMockData: async () => {
            update(s => ({ ...s, directoryPath: MOCK_DIRECTORY_PATH, loading: true, error: "" }));
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

                update(s => ({
                    ...s,
                    images: processedImages,
                    datasetSummary: summary,
                    totalImages: processedImages.length,
                    totalPages: 1,
                    loading: false
                }));
            } catch (err) {
                console.error("❌ Mock load failed", err);
                update(s => ({ ...s, error: "Failed to load mock data.", loading: false }));
            }
        },

        selectDirectory: async () => {
            const state = get({ subscribe });
            if (state.isMockMode) {
                await imageStore.loadMockData();
                return;
            }

            update(s => ({ ...s, loading: true, error: "" }));

            try {
                const selected = await open({
                    directory: true,
                    multiple: false,
                    title: "Select Image Directory",
                });

                if (selected) {
                    const newPath = selected as string;
                    if (!newPath.trim()) {
                        update(s => ({ ...s, error: "Invalid directory path: path is empty", loading: false }));
                        return;
                    }

                    update(s => ({ ...s, directoryPath: newPath, currentPage: 1, images: [] }));
                    console.log("Selected directory:", newPath);
                    await imageStore.loadImagesPage(1);
                } else {
                    update(s => ({ ...s, loading: false }));
                }
            } catch (err: any) {
                console.error("Error selecting directory:", err);
                update(s => ({ ...s, error: `Failed to select directory: ${err.message || String(err)}`, loading: false }));
            }
        },

        loadImagesPage: async (page: number) => {
            const state = get({ subscribe });
            if (state.isMockMode) {
                await imageStore.loadMockData();
                return;
            }

            if (!state.directoryPath) {
                return;
            }

            update(s => ({ ...s, loading: true, error: "" }));

            try {
                const result = await fetchPaginatedImages(state.directoryPath, page, state.pageSize);

                update(s => ({
                    ...s,
                    images: result.processedImages,
                    totalImages: result.totalImages,
                    totalPages: result.totalPages,
                    currentPage: page,
                    loading: false
                }));

                if (page === 1) {
                    imageStore.generateLabelMeSummary();
                }

                await tick();
                if (typeof window !== "undefined") {
                    window.scrollTo(0, 0);
                }

            } catch (err: any) {
                console.error("Page: Error loading images:", err);
                update(s => ({
                    ...s,
                    error: `Failed to load images: ${err.message || "Unknown error"}`,
                    images: [],
                    totalImages: 0,
                    totalPages: 0,
                    loading: false
                }));
            }
        },

        generateLabelMeSummary: async () => {
            const state = get({ subscribe });
            if (!state.directoryPath) return;

            try {
                const summary = await fetchDatasetSummary(state.directoryPath);
                update(s => ({ ...s, datasetSummary: summary }));
            } catch (err: any) {
                console.warn(`Page: Failed to generate LabelMe summary: ${err.message}`);
                update(s => ({ ...s, datasetSummary: null }));
            }
        },
    };
}

export const imageStore = createImageStore();

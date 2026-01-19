
import { writable, get } from "svelte/store";
import { performAutoAnnotation } from "../services/datasetService";
import { imageStore } from "./imageStore";

export interface AnnotationState {
    annotating: boolean;
    autoAnnotating: boolean;
    annotationType: "bounding_box" | "polygon";
    autoAnnotationEnabled: boolean;
}

const initialState: AnnotationState = {
    annotating: false,
    autoAnnotating: false,
    annotationType: "bounding_box",
    autoAnnotationEnabled: true,
};

function createAnnotationStore() {
    const { subscribe, set, update } = writable<AnnotationState>(initialState);

    return {
        subscribe,
        set,
        update,

        annotateImages: async () => {
            const imgState = get(imageStore);
            const state = get({ subscribe });

            if (imgState.isMockMode) {
                console.log("🧪 Mock Mode: Using local mock annotations.");
                await imageStore.loadMockData();
                return;
            }

            if (!imgState.directoryPath || imgState.images.length === 0) {
                imageStore.update(s => ({ ...s, error: "Please select a directory with images first" }));
                return;
            }

            update(s => ({ ...s, annotating: true }));
            imageStore.update(s => ({ ...s, error: "" }));

            try {
                const result = await performAutoAnnotation(imgState.directoryPath, imgState.currentPage, imgState.pageSize);

                if (result && result.annotated_images && result.annotated_images.length > 0) {
                    imageStore.update(s => {
                        const updatedImages = s.images.map(img => {
                            const annotatedImgData = result.annotated_images.find(ai => ai.path === img.path);
                            if (annotatedImgData) {
                                return {
                                    ...img,
                                    annotations: annotatedImgData.annotations || img.annotations,
                                    annotated: !!(annotatedImgData.has_json && annotatedImgData.annotations && annotatedImgData.annotations.length > 0),
                                    has_json: annotatedImgData.has_json !== undefined ? annotatedImgData.has_json : img.has_json,
                                };
                            }
                            return img;
                        });
                        return { ...s, images: updatedImages };
                    });
                    await imageStore.generateLabelMeSummary();
                }
            } catch (err: any) {
                console.error("Page: Error annotating images:", err);
                imageStore.update(s => ({ ...s, error: `Failed to annotate images: ${err.message || "Unknown error"}` }));
            } finally {
                update(s => ({ ...s, annotating: false }));
            }
        },

        autoLoadAnnotationsForPage: async (page: number) => {
            const imgState = get(imageStore);
            if (imgState.isMockMode) return;

            update(s => ({ ...s, autoAnnotating: true }));

            try {
                const result = await performAutoAnnotation(imgState.directoryPath, page, imgState.pageSize);
                const mergedAnnotatedImages = result.annotated_images;

                if (mergedAnnotatedImages && mergedAnnotatedImages.length > 0) {
                    imageStore.update(s => {
                        const updatedImages = s.images.map(img => {
                            const annotatedImgData = mergedAnnotatedImages.find(ai => ai.path === img.path);
                            if (annotatedImgData) {
                                const existing = img.annotations || [];
                                const incoming = annotatedImgData.annotations || [];
                                const combined = [...existing];
                                incoming.forEach(newAnn => {
                                    const exists = combined.some(e =>
                                        e.label === newAnn.label &&
                                        e.shape_type === newAnn.shape_type &&
                                        JSON.stringify(e.points) === JSON.stringify(newAnn.points)
                                    );
                                    if (!exists) combined.push(newAnn);
                                });

                                return {
                                    ...img,
                                    annotations: combined,
                                    annotated: combined.length > 0,
                                    has_json: annotatedImgData.has_json !== undefined ? annotatedImgData.has_json : img.has_json
                                };
                            }
                            return img;
                        });
                        return { ...s, images: updatedImages };
                    });
                }
            } catch (err) {
                console.warn("Auto-annotation failed (non-blocking):", err);
            } finally {
                update(s => ({ ...s, autoAnnotating: false }));
            }
        }
    };
}

export const annotationStore = createAnnotationStore();

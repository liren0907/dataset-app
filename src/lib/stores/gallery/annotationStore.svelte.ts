
import { performAutoAnnotation } from "$lib/services/gallery/datasetService";
import { imageState, loadMockData, generateLabelMeSummary } from "./imageStore.svelte";

export interface AnnotationState {
    annotating: boolean;
    autoAnnotating: boolean;
    annotationType: "bounding_box" | "polygon";
    autoAnnotationEnabled: boolean;
}

export const annotationState: AnnotationState = $state({
    annotating: false,
    autoAnnotating: false,
    annotationType: "bounding_box",
    autoAnnotationEnabled: true,
});

export async function annotateImages() {
    if (imageState.isMockMode) {
        console.log("🧪 Mock Mode: Using local mock annotations.");
        await loadMockData();
        return;
    }

    if (!imageState.directoryPath || imageState.images.length === 0) {
        imageState.error = "Please select a directory with images first";
        return;
    }

    annotationState.annotating = true;
    imageState.error = "";

    try {
        const result = await performAutoAnnotation(imageState.directoryPath, imageState.currentPage, imageState.pageSize);

        if (result && result.annotated_images && result.annotated_images.length > 0) {
            imageState.images = imageState.images.map(img => {
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
            await generateLabelMeSummary();
        }
    } catch (err: any) {
        console.error("Page: Error annotating images:", err);
        imageState.error = `Failed to annotate images: ${err.message || "Unknown error"}`;
    } finally {
        annotationState.annotating = false;
    }
}

export async function autoLoadAnnotationsForPage(page: number) {
    if (imageState.isMockMode) return;

    annotationState.autoAnnotating = true;

    try {
        const result = await performAutoAnnotation(imageState.directoryPath, page, imageState.pageSize);
        const mergedAnnotatedImages = result.annotated_images;

        if (mergedAnnotatedImages && mergedAnnotatedImages.length > 0) {
            imageState.images = imageState.images.map(img => {
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
        }
    } catch (err) {
        console.warn("Auto-annotation failed (non-blocking):", err);
    } finally {
        annotationState.autoAnnotating = false;
    }
}

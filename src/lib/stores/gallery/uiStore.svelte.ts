
import type { ProcessedImage } from "$lib/services/gallery/datasetService";

export interface UIState {
    viewMode: "grid" | "column";
    selectedImage: ProcessedImage | null;
    editMode: "modal" | "sidebar";
    showAnnotationModal: boolean;
}

export const uiState: UIState = $state({
    viewMode: "grid",
    selectedImage: null,
    editMode: "modal",
    showAnnotationModal: false,
});

export function resetSelection() {
    uiState.selectedImage = null;
    uiState.showAnnotationModal = false;
}

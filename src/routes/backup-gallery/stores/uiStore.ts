
import { writable } from "svelte/store";
import type { ProcessedImage } from "../services/datasetService";

export interface UIState {
    viewMode: "grid" | "column";
    selectedImage: ProcessedImage | null;
    editMode: "modal" | "sidebar";
    showAnnotationModal: boolean;
}

const initialState: UIState = {
    viewMode: "grid",
    selectedImage: null,
    editMode: "modal",
    showAnnotationModal: false,
};

function createUIStore() {
    const { subscribe, set, update } = writable<UIState>(initialState);

    return {
        subscribe,
        set,
        update,
        resetSelection: () => update(s => ({ ...s, selectedImage: null, showAnnotationModal: false }))
    };
}

export const uiStore = createUIStore();

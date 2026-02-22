import { writable, derived } from 'svelte/store';

export type ImageStatus = 'todo' | 'in_progress' | 'done' | 'needs_review';

export interface ImageStatusEntry {
    path: string;
    status: ImageStatus;
}

function createImageStatusStore() {
    const { subscribe, update, set } = writable<Map<string, ImageStatus>>(new Map());

    return {
        subscribe,
        setStatus(path: string, status: ImageStatus) {
            update(map => {
                const newMap = new Map(map);
                newMap.set(path, status);
                return newMap;
            });
        },
        getStatus(map: Map<string, ImageStatus>, path: string): ImageStatus {
            return map.get(path) || 'todo';
        },
        reset() {
            set(new Map());
        },
    };
}

export const imageStatusStore = createImageStatusStore();

export const STATUS_CONFIG: Record<ImageStatus, { label: string; color: string; icon: string }> = {
    todo: { label: 'To Do', color: '#6b7280', icon: 'radio_button_unchecked' },
    in_progress: { label: 'In Progress', color: '#f59e0b', icon: 'pending' },
    done: { label: 'Done', color: '#22c55e', icon: 'check_circle' },
    needs_review: { label: 'Review', color: '#ef4444', icon: 'error' },
};

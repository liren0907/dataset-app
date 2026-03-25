export type ImageStatus = 'todo' | 'in_progress' | 'done' | 'needs_review';

export interface ImageStatusEntry {
    path: string;
    status: ImageStatus;
}

export const imageStatusMap = $state(new Map<string, ImageStatus>());

export function setImageStatus(path: string, status: ImageStatus) {
    imageStatusMap.set(path, status);
}

export function getImageStatus(path: string): ImageStatus {
    return imageStatusMap.get(path) || 'todo';
}

export function resetImageStatuses() {
    imageStatusMap.clear();
}

export const STATUS_CONFIG: Record<ImageStatus, { label: string; color: string; icon: string }> = {
    todo: { label: 'To Do', color: '#6b7280', icon: 'radio_button_unchecked' },
    in_progress: { label: 'In Progress', color: '#f59e0b', icon: 'pending' },
    done: { label: 'Done', color: '#22c55e', icon: 'check_circle' },
    needs_review: { label: 'Review', color: '#ef4444', icon: 'error' },
};

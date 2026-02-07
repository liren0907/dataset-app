import { writable } from 'svelte/store';

export interface Toast {
    id: string;
    message: string;
    variant: 'success' | 'error' | 'info' | 'warning';
    duration?: number;
}

function createToastStore() {
    const { subscribe, update } = writable<Toast[]>([]);

    return {
        subscribe,

        show: (message: string, variant: Toast['variant'] = 'info', duration: number = 5000) => {
            const id = crypto.randomUUID();
            const toast: Toast = { id, message, variant, duration };

            update(toasts => [...toasts, toast]);

            if (duration > 0) {
                setTimeout(() => {
                    update(toasts => toasts.filter(t => t.id !== id));
                }, duration);
            }

            return id;
        },

        success: (message: string, duration?: number) => {
            return createToastStore().show(message, 'success', duration);
        },

        error: (message: string, duration?: number) => {
            return createToastStore().show(message, 'error', duration);
        },

        dismiss: (id: string) => {
            update(toasts => toasts.filter(t => t.id !== id));
        },

        clear: () => {
            update(() => []);
        }
    };
}

export const toastStore = createToastStore();

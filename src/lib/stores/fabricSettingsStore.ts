import { writable } from 'svelte/store';

export interface FabricSettings {
    baseStrokeWidth: number;
}

const DEFAULT_SETTINGS: FabricSettings = {
    baseStrokeWidth: 2,
};

function createFabricSettingsStore() {
    const { subscribe, set, update } = writable<FabricSettings>(DEFAULT_SETTINGS);

    return {
        subscribe,
        setBaseStrokeWidth(value: number) {
            update(s => ({ ...s, baseStrokeWidth: Math.max(0.5, Math.min(5, value)) }));
        },
        reset() {
            set(DEFAULT_SETTINGS);
        },
    };
}

export const fabricSettings = createFabricSettingsStore();

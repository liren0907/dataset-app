import { writable } from 'svelte/store';

export interface FabricSettings {
    baseStrokeWidth: number;
    showVertexPoints: boolean;
}

const DEFAULT_SETTINGS: FabricSettings = {
    baseStrokeWidth: 2,
    showVertexPoints: true,
};

function createFabricSettingsStore() {
    const { subscribe, set, update } = writable<FabricSettings>(DEFAULT_SETTINGS);

    return {
        subscribe,
        setBaseStrokeWidth(value: number) {
            update(s => ({ ...s, baseStrokeWidth: Math.max(0.5, Math.min(5, value)) }));
        },
        toggleVertexPoints() {
            update(s => ({ ...s, showVertexPoints: !s.showVertexPoints }));
        },
        reset() {
            set(DEFAULT_SETTINGS);
        },
    };
}

export const fabricSettings = createFabricSettingsStore();

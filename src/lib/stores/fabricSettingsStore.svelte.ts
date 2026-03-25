export interface FabricSettings {
    baseStrokeWidth: number;
    showVertexPoints: boolean;
}

const DEFAULT_SETTINGS: FabricSettings = {
    baseStrokeWidth: 2,
    showVertexPoints: true,
};

export const fabricSettings = $state<FabricSettings>({ ...DEFAULT_SETTINGS });

export function setBaseStrokeWidth(value: number) {
    fabricSettings.baseStrokeWidth = Math.max(0.5, Math.min(5, value));
}

export function toggleVertexPoints() {
    fabricSettings.showVertexPoints = !fabricSettings.showVertexPoints;
}

export function resetFabricSettings() {
    fabricSettings.baseStrokeWidth = DEFAULT_SETTINGS.baseStrokeWidth;
    fabricSettings.showVertexPoints = DEFAULT_SETTINGS.showVertexPoints;
}

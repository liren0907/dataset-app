export interface LabelClass {
    name: string;
    color: string;
    shortcut: number | null; // 1-9 for quick assignment
}

const PALETTE = [
    '#ef4444', '#3b82f6', '#22c55e', '#f59e0b', '#a855f7',
    '#ec4899', '#14b8a6', '#f97316', '#6366f1', '#84cc16',
    '#06b6d4', '#e11d48', '#8b5cf6', '#10b981', '#d946ef',
];

export const labelClasses = $state<LabelClass[]>([]);

export function addClass(name: string) {
    if (labelClasses.find(c => c.name === name)) return;
    const colorIndex = labelClasses.length % PALETTE.length;
    const shortcut = labelClasses.length < 9 ? labelClasses.length + 1 : null;
    labelClasses.push({ name, color: PALETTE[colorIndex], shortcut });
}

export function removeClass(name: string) {
    const idx = labelClasses.findIndex(c => c.name === name);
    if (idx !== -1) labelClasses.splice(idx, 1);
}

export function updateClassColor(name: string, color: string) {
    const cls = labelClasses.find(c => c.name === name);
    if (cls) cls.color = color;
}

export function resetLabelTaxonomy() {
    labelClasses.length = 0;
}

/** Get shortcut number (1-9) to class name map */
export function getShortcutMap(): Map<number, string> {
    const map = new Map<number, string>();
    for (const cls of labelClasses) {
        if (cls.shortcut !== null) {
            map.set(cls.shortcut, cls.name);
        }
    }
    return map;
}

/** Get class name to color map */
export function getClassColorMap(): Map<string, string> {
    const map = new Map<string, string>();
    for (const cls of labelClasses) {
        map.set(cls.name, cls.color);
    }
    return map;
}

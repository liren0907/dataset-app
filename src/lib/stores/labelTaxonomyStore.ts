import { writable, derived } from 'svelte/store';

export interface LabelClass {
    name: string;
    color: string;
    shortcut: number | null; // 1-9 for quick assignment
}

const DEFAULT_CLASSES: LabelClass[] = [
    { name: 'car', color: '#ef4444', shortcut: 1 },
    { name: 'person', color: '#3b82f6', shortcut: 2 },
    { name: 'bicycle', color: '#22c55e', shortcut: 3 },
    { name: 'traffic_light', color: '#f59e0b', shortcut: 4 },
    { name: 'sign', color: '#a855f7', shortcut: 5 },
];

const PALETTE = [
    '#ef4444', '#3b82f6', '#22c55e', '#f59e0b', '#a855f7',
    '#ec4899', '#14b8a6', '#f97316', '#6366f1', '#84cc16',
    '#06b6d4', '#e11d48', '#8b5cf6', '#10b981', '#d946ef',
];

function createLabelTaxonomyStore() {
    const { subscribe, set, update } = writable<LabelClass[]>(DEFAULT_CLASSES);

    return {
        subscribe,
        addClass(name: string) {
            update(classes => {
                if (classes.find(c => c.name === name)) return classes;
                const colorIndex = classes.length % PALETTE.length;
                const shortcut = classes.length < 9 ? classes.length + 1 : null;
                return [...classes, { name, color: PALETTE[colorIndex], shortcut }];
            });
        },
        removeClass(name: string) {
            update(classes => classes.filter(c => c.name !== name));
        },
        updateClassColor(name: string, color: string) {
            update(classes =>
                classes.map(c => (c.name === name ? { ...c, color } : c))
            );
        },
        reset() {
            set(DEFAULT_CLASSES);
        },
    };
}

export const labelTaxonomy = createLabelTaxonomyStore();

/** Derived store: map shortcut number (1-9) to class name */
export const shortcutMap = derived(labelTaxonomy, ($taxonomy) => {
    const map = new Map<number, string>();
    for (const cls of $taxonomy) {
        if (cls.shortcut !== null) {
            map.set(cls.shortcut, cls.name);
        }
    }
    return map;
});

/** Derived store: map class name to color */
export const classColorMap = derived(labelTaxonomy, ($taxonomy) => {
    const map = new Map<string, string>();
    for (const cls of $taxonomy) {
        map.set(cls.name, cls.color);
    }
    return map;
});

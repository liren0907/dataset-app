# Svelte 5 Migration — Remaining Work

This document tracks what was **not** completed during the Svelte 4 → 5 migration (2026-03-22).

## Migration Summary

- **Completed**: 100 `.svelte` files migrated to Svelte 5 runes syntax
- **Build**: Passes cleanly (`yarn build`)
- **Runtime testing**: Not yet performed

---

## 1. Stores Not Converted to Runes

All 10 store files still use `writable`/`derived` from `svelte/store`. This is intentional — they work in Svelte 5 without changes. Converting to `$state` runes is optional.

**Files:**

| File | Type | Notes |
|---|---|---|
| `src/store.js` | Root legacy store | Theme, sidebar, GUI state. Barely used. |
| `src/lib/stores/toastStore.ts` | Toast notifications | Factory pattern with `writable` |
| `src/lib/stores/fabricSettingsStore.ts` | Fabric settings | Simple `writable` |
| `src/lib/stores/imageStatusStore.ts` | Image workflow status | Uses `Map` as store value |
| `src/lib/stores/labelTaxonomyStore.ts` | Label classes | Has `derived` stores (`shortcutMap`, `classColorMap`) |
| `src/lib/stores/gallery/imageStore.ts` | Image pagination | Complex store with async methods |
| `src/lib/stores/gallery/uiStore.ts` | Gallery UI state | View mode, selected image |
| `src/lib/stores/gallery/annotationStore.ts` | Annotation ops | Cross-store mutations to `imageStore` |
| `src/lib/stores/gallery/exportStore.ts` | Export/crop ops | Most complex store, Tauri event listeners |
| `src/routes/turbo-export/stores/exportStore.ts` | Turbo export state | 20+ individual `writable` stores |

**Why not converted:**
- Every component using `$storeName` auto-subscription syntax would need refactoring
- `writable`/`derived` are fully supported in Svelte 5
- Risk is high, benefit is low

**If you want to convert later:**
- Replace `writable()` with a module-level `$state()` object
- Replace `derived()` with `$derived()`
- Remove `subscribe`/`set`/`update` methods — consumers access state directly
- All `$storeName` usages in components must change to direct imports

---

## 2. Pre-existing TypeScript Errors

The legacy gallery files (which had ~80+ type errors) have been moved to `deprecated/` and are no longer part of the active codebase.

One remaining type error exists in the active codebase:
- `src/routes/turbo-export/stores/exportStore.ts` — `DetailedStats` missing properties

**To check current errors:**
```bash
npx svelte-kit sync && npx svelte-check --tsconfig ./tsconfig.json
```

---

## 3. `svelte-dnd-action` Event Syntax (Needs Runtime Verification)

In `src/routes/turbo-export/components/LabelManager.svelte`, the drag-and-drop events were changed from Svelte 4 to Svelte 5 syntax:

```svelte
<!-- Before -->
<div use:dndzone={{items}} on:consider={handler} on:finalize={handler}>

<!-- After -->
<div use:dndzone={{items}} onconsider={handler} onfinalize={handler}>
```

`svelte-dnd-action@^0.9.67` declares Svelte 5 support in its peer dependencies, but this specific event syntax change needs runtime testing. If drag-and-drop breaks, revert to `on:consider`/`on:finalize` — Svelte 5 compatibility mode still supports the old syntax for action events.

---

## 4. Runtime Testing Checklist

The build passes but no runtime testing was done. Test these flows with `yarn tauri dev`:

- [ ] **Home page**: Drag-drop file import
- [ ] **Gallery** (`/gallery`): Select directory → browse → paginate → annotate → crop → export
- [ ] **Turbo Export** (`/turbo-export`): Select directory → scan labels → drag-reorder labels → configure → export
- [ ] **Smart Tools** (`/smart-tools`): Upload image → crop → preview → export
- [ ] **Fabric Annotator** (`/fabric-annotator`): Open directory → draw bbox/polygon/polyline → save → navigate images
- [ ] **Settings** (`/settings`): Theme toggle (light/dark)
---

## 5. `src/store.js` — Legacy Root Store

This file at the project root (`src/store.js`) contains several `writable` stores:
- `theme` — light/dark theme
- `darkMode` — backward compat boolean
- `isSidebarOpen` — sidebar toggle
- `guiState` — locale, OS, modal state
- `dataStore` — file mode, source list
- `imageStore` — transform state (scale, rotation, EXIF)

It's mostly unused by the current app (the layout manages theme locally, gallery has its own stores). Consider removing or consolidating if these stores are truly dead code.

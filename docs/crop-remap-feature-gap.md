# Crop & Remap Feature Gap Analysis

> **Document Created:** 2026-01-19  
> **Purpose:** Record missing features in `/gallery/CropRemapTool` compared to standalone pages

---

## Context

The `/gallery` route has a `CropRemapTool.svelte` component that provides basic crop & remap functionality. However, the standalone pages (`/crop-remap` and `/crop-remap-component`) have additional features not yet integrated into the gallery.

---

## ✅ Features Already in Gallery CropRemapTool

| Feature | Location |
|---------|----------|
| Source/Output directory selection | `CropRemapTool.svelte` |
| Dataset analysis with `fetchDatasetSummary` | Auto-triggers on source change |
| Parent label selection with smart suggestion | Uses common parent keywords |
| Child label selection (toggleable LabelBadge chips) | Filters by safety equipment keywords |
| Padding factor slider (0.5x - 2.0x) | Range input with visual feedback |
| Crop & remap processing | Calls `performCropAndRemap` service |
| Success/error alerts | Uses shared `Alert` component |
| Modern SplitPaneModal UI | Uses shared UI component library |

---

## ❌ Missing Features (To Integrate)

### 1. Preview Grid Component

**Source:** `/crop-remap-component/components/PreviewGrid.svelte`

**Description:**  
Displays a grid of 5 random annotated sample images from the dataset after analysis. Helps users visualize their data before processing.

**Key Elements:**
- Grid layout (1-5 columns responsive)
- Thumbnail images with "Preview" badge
- Click to open full-size modal
- Loading state with spinner

**Integration Notes:**
- Add to right panel of SplitPaneModal or as a collapsible section
- Requires `autoPreviewAnnotatedImages` service function

---

### 2. Interactive KonvaJS Preview Modal

**Source:** `/crop-remap-component/components/KonvaPreviewModal.svelte`

**Description:**  
Full-screen modal with interactive annotation viewer using KonvaJS. Allows users to inspect, zoom, and interact with annotations before processing.

**Key Features:**
- **Zoom controls:** Zoom in (+), Zoom out (-), Reset (0), Fit to screen (R)
- **Annotation controls:** Select all (Ctrl+A), Deselect (Esc), Delete selected (Del)
- **Status display:** Current zoom %, total annotations, selected count
- **Keyboard shortcuts:** Full keyboard navigation support
- **KonvaJS canvas:** Interactive panning and zooming

**Integration Notes:**
- Could reuse existing `/gallery/components/konvaService.ts`
- Consider sharing with `ModalAnnotationViewer.svelte` logic

---

### 3. Auto Preview Service Function

**Source:** `/crop-remap-component/services/dataService.ts`

**Function:** `autoPreviewAnnotatedImages(sourceDir, datasetSummary)`

**Description:**  
Automatically fetches 5 random annotated images after dataset analysis for preview purposes.

**Backend Call:**
```typescript
invoke("generate_annotated_previews", {
    sourceDir: sourceDir,
    numPreviews: 5,
    tempDir: tempDir  // Uses appDataDir for temp storage
})
```

**Returns:**
```typescript
PreviewImage[] = {
    id: string,
    path: string,
    previewUrl: string,  // Tauri convertFileSrc
    name: string,
    annotations: any[]   // For KonvaJS drawing
}
```

**Integration Notes:**
- Requires `generate_annotated_previews` Rust command (verify it exists in backend)
- Uses `appDataDir` from `@tauri-apps/api/path`

---

### 4. Dataset Statistics Panel

**Source:** `/crop-remap/components/StatsDisplay.svelte`

**Description:**  
A dedicated panel showing dataset statistics in a visual format separate from the configuration panel.

**Current Status:**  
Gallery's `CropRemapTool` shows basic info in the sidebar but doesn't have a dedicated stats panel.

**Note:** Lower priority - the current implementation may be sufficient.

---

## Implementation Priority

| Priority | Feature | Effort | Impact |
|----------|---------|--------|--------|
| 🔴 High | Preview Grid | Medium | High - Users can visualize data |
| 🔴 High | KonvaJS Preview Modal | Medium | High - Inspect annotations |
| 🟡 Medium | Auto Preview Service | Low | Required for Preview Grid |
| 🟢 Low | Stats Panel | Low | Nice to have |

---

## File References

### Standalone Components to Reference:
- `/src/routes/crop-remap-component/components/PreviewGrid.svelte`
- `/src/routes/crop-remap-component/components/KonvaPreviewModal.svelte`
- `/src/routes/crop-remap-component/services/dataService.ts`
- `/src/routes/crop-remap-component/services/konvaService.ts`

### Gallery Target:
- `/src/routes/gallery/components/CropRemapTool.svelte`
- `/src/routes/gallery/services/datasetService.ts`

### Existing Reusable Components:
- `/src/routes/gallery/components/konvaService.ts` (KonvaManager)
- `/src/routes/gallery/components/ModalAnnotationViewer.svelte` (Has KonvaJS logic)

---

## Notes

- The gallery already has a sophisticated `ModalAnnotationViewer` with KonvaJS - could potentially share/reuse that logic
- The `konvaService.ts` in gallery is more mature than the one in `/crop-remap-component`
- Backend command `generate_annotated_previews` needs verification


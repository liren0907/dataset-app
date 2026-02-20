# Gallery UI Unification Plan

> **Last Updated**: 2026-02-07  
> **Status**: Mostly Complete ✅

## Completed Work

The following phases have been completed:

- ✅ **Shared Component Library** - All 12 UI components created in `src/lib/components/ui/`
- ✅ **Phase 1: Navbar Unification** - `GalleryNavbar.svelte` now uses `IconButton`, `ToggleButtonGroup`, `IconSegmentedControl`
- ✅ **Phase 2: Modal Cleanup** - `ExportModal.svelte` uses `SplitPaneModal`, `Button`, `LabelBadge`, `Alert`, etc.

---

## Remaining Work

### Phase 3: Split Pane Standardization (Partial)

**Targets**:
- `CropRemapTool.svelte` - ✅ Done (Now uses SplitPaneModal layout)
- `AdvancedCropRemapTool.svelte` - ⏳ Pending (Still uses custom layout)
- `HierarchicalCrop.svelte` - ⏳ Pending (Still uses custom card layout)

**Action**: Refactor the pending components to use the shared `SplitPaneModal` component for layout consistency.

**Benefit**: Ensures exact layout consistency (width ratios, padding, scrollbars) across all complex tools.

---

## Component Reference Tree

快速開發參考：元件層級、Props、Events、Store 連結

### Stores (狀態管理)

| Store | 位置 | 主要狀態 |
|-------|------|---------|
| `imageStore` | `stores/imageStore.ts` | `directoryPath`, `images`, `loading`, `error`, `currentPage`, `pageSize`, `totalImages`, `totalPages`, `datasetSummary`, `isMockMode` |
| `uiStore` | `stores/uiStore.ts` | `viewMode` (grid/column), `editMode` (modal/sidebar), `selectedImage`, `showAnnotationModal` |
| `annotationStore` | `stores/annotationStore.ts` | `annotating`, `autoAnnotating`, `annotationType` (bounding_box/polygon), `autoAnnotationEnabled` |
| `exportStore` | `stores/exportStore.ts` | `showActualExportModal`, `pageExportLoading`, `pageExportError`, `pageExportSuccess`, `showCropTool`, `showAdvancedCropTool`, `showHierarchicalCrop`, `croppedDatasets`, `cropModalParentLabel`, `activeCroppedDatasetPath`, `originalDirectoryPath`, `cropProcessing`, `cropProgressMessage`, `cropProgressCurrent`, `cropProgressTotal` |

### Main Page Structure

```
/gallery (+page.svelte)
│
├── GalleryNavbar ─────────────────────────────────────────────────
│   Props: isMockMode, loading, directoryPath, images, annotationType,
│          autoAnnotationEnabled, annotating, showCropTool, 
│          showAdvancedCropTool, viewMode, editMode
│   │
│   ├── on:toggleMockMode ──────→ imageStore.setMockMode()
│   ├── on:selectDirectory ─────→ imageStore.selectDirectory()
│   ├── on:setAnnotationType ───→ $annotationStore.annotationType = value
│   ├── on:toggleAutoAnnotation → $annotationStore.autoAnnotationEnabled = !value
│   ├── on:annotateImages ──────→ annotationStore.annotateImages()
│   ├── on:openExportModal ─────→ $exportStore.showActualExportModal = true
│   ├── on:toggleCropTool ──────→ $exportStore.showCropTool = !value
│   ├── on:toggleAdvancedCropTool → $exportStore.showAdvancedCropTool = !value
│   ├── on:setViewMode ─────────→ $uiStore.viewMode = value
│   └── on:setEditMode ─────────→ $uiStore.editMode = value
│
├── DatasetSummary ────────────────────────────────────────────────
│   Props: datasetSummary
│   │
│   └── on:initiateCrop ────────→ exportStore.openCropModalWithLabel(label)
│       (點擊 label badge 會觸發 AdvancedCropRemapTool 並預選該 label)
│
├── CroppedDatasetCard ────────────────────────────────────────────
│   Props: tempPath, exportedPath, imageCount, parentLabel, childLabels, createdAt
│   │
│   ├── on:preview ─────────────→ openCroppedPreview(tempPath)
│   ├── on:openInGallery ───────→ exportStore.openCroppedDatasetInGallery(tempPath)
│   └── on:remove ──────────────→ exportStore.removeCroppedDataset(tempPath)
│
├── ImageGallery ──────────────────────────────────────────────────
│   Props: images, totalImages, viewMode, currentPage, totalPages, pageSize, selectedImage,
│          annotationType, loading, loadingMore
│   │
│   ├── on:loadPage ────────────→ imageStore.loadImagesPage(page)
│   │                             annotationStore.autoLoadAnnotationsForPage()
│   │                             Note: +page currently listens to `pageChange` and should be aligned.
│   └── on:imageClick ──────────→ if editMode === "modal":
│                                   $uiStore.selectedImage = image
│                                   $uiStore.showAnnotationModal = true
│                                 else:
│                                   $uiStore.selectedImage = image
│
├── ImagePreviewPanel (when editMode === "sidebar" && selectedImage)
│   Props: selectedImage
│   │
│   └── on:close ───────────────→ $uiStore.selectedImage = null
│
├── GalleryEmptyState (when loading || !directoryPath || images.length === 0)
│   Props: loading, directoryPath, images
│   │
│   └── on:selectDirectory ─────→ imageStore.selectDirectory()
│
└── [Modals / Tools] ──────────────────────────────────────────────

    ├── ExportModal (when $exportStore.showActualExportModal)
    │   Props: showModal, currentDirectoryPath, currentDatasetSummary
    │   │
    │   ├── on:closeModal ──────→ $exportStore.showActualExportModal = false
    │   └── on:runExport ───────→ exportStore.runUnifiedExport(details)
    │                             → datasetService.performDatasetExport()
    │
    ├── CropRemapTool (when $exportStore.showCropTool)
    │   Props: isOpen
    │   │
    │   ├── on:cropCompleted ───→ exportStore.handleCropCompleted(outputDir)
    │   └── on:close ───────────→ $exportStore.showCropTool = false
    │
    ├── AdvancedCropRemapTool (when $exportStore.showAdvancedCropTool)
    │   Props: currentDirectory, cropToolOpen, preSelectedParentLabel
    │   │
    │   └── on:cropCompleted ───→ exportStore.handleCropCompleted(outputDir, details)
    │       (details: parentLabel, childLabels, imageCount)
    │
    ├── ModalAnnotationViewer (when $uiStore.showAnnotationModal && $uiStore.selectedImage)
    │   Props: showModal, selectedImage, autoAnnotationEnabled, isMockMode
    │   │
    │   ├── on:close ───────────→ $uiStore.showAnnotationModal = false
    │   │                         $uiStore.selectedImage = null
    │   └── on:save ────────────→ (annotation data preserved)
    │
    ├── CroppedDatasetPreviewModal (when showCroppedPreviewModal)
    │   Props: isOpen, loading, error, outputPath, images, sampleCount
    │   │
    │   ├── on:close ───────────→ closeCroppedPreview()
    │   └── on:selectImage ─────→ handleSelectPreviewImage(image)
    │                             → Opens KonvaViewer
    │
    └── KonvaViewer (when selectedPreviewImage !== null)
        Props: showModal, imageData
        │
        └── on:close ───────────→ handlePreviewViewerClose()
```

### Services (後端調用)

| Service | 位置 | 主要功能 |
|---------|------|---------|
| `datasetService.ts` | `services/datasetService.ts` | 圖片列表、標註、匯出 |
| `konvaService.ts` | `services/konvaService.ts` | Konva.js 渲染 |

**datasetService 常用函數：**
- `fetchPaginatedImages(path, page, pageSize)` → Tauri: `get_paginated_images`
- `fetchDatasetSummary(path)` → Tauri: `get_labelme_summary`
- `performAutoAnnotation(path, page, pageSize)` → Tauri: `auto_annotate_images`
- `performDatasetExport(params)` → Tauri: `export_to_yolo_new` / `extract_labels`
- `performCropAndRemap(...)` → Tauri: `crop_and_remap_annotations`
- `generateAnnotatedPreviews(...)` → Tauri: `generate_annotated_previews`

### UI Components (共用元件)

位置: `src/lib/components/ui/`

```
Button, IconButton, BrowseInput, SectionLabel,
SelectionCard, SplitPaneModal, SimpleModal,
Alert, LabelBadge, ToggleButtonGroup, IconSegmentedControl
```

**Import**: `import { Button, Alert, ... } from "$lib/components/ui";`

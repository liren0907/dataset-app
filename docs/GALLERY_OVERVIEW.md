# Existing Gallery Implementations Documentation

This document provides a comprehensive technical review of the three existing gallery implementations in the codebase.

## 1. Dataset Gallery (`src/routes/dataset-gallery`)
**Type**: Legacy / Baseline Implementation

### Overview
The original gallery implementation designed for basic viewing and navigation of dataset images. It relies heavily on global shared components.

### Architecture
*   **Controller**: `+page.svelte` (Monolithic controller pattern)
*   **Dependencies**: Imports shared services from `$lib/services/datasetService`.
*   **Components**:
    *   `$lib/ImageGallery.svelte`: Core grid display.
    *   `$lib/CropRemapTool.svelte`: Embedded directly in the page flow (older implementation).
    *   `$lib/ImageViewerModal.svelte`: Read-only image viewer.
    *   `components/GalleryControls.svelte`: Local component for top bar controls.

### Key Features
*   **Browsing**: Standard grid view with pagination.
*   **Annotation Display**: Can load and visualize bounding boxes (read-only).
*   **Crop Tool**: Includes a legacy version of the crop/remap tool embedded above the grid.
*   **Export**: Basic dataset export functionality via modal.

---

## 2. Advanced Gallery (`src/routes/dataset-gallery-advanced`)
**Type**: Editor / Experimental Implementation

### Overview
A specialized version focused on **Annotation Editing**. It introduces `Konva.js` for interactive canvas manipulation, allowing users to draw and modify annotations.

### Architecture
*   **Controller**: `+page.svelte`
*   **Service Isolation**: Uses a *local* `datasetService.ts` instead of the global one, indicating a fork/divergence in logic.
*   **Key Components**:
    *   **`ModalAnnotationViewer.svelte`**: A complex component integrating `Konva.js`. This is the core differentiator, enabling interactive editing (drawing, resizing, deleting shapes).
    *   `konvaService.ts`: A dedicated TypeScript service for managing the Konva stage and shapes.
    *   `components/ImageGallery.svelte`: Local copy of the grid component.

### Key Features
*   **Interactive Editing**:
    *   **Bounding Boxes & Polygons**: Users can draw new shapes and drag/edit existing ones.
    *   **Keyboard Shortcuts**: Supports Delete, Select All, Save (Ctrl+S).
    *   **Zoom/Pan**: Canvas-based zooming and panning.
*   **Auto-Annotation**: Enhanced logic to "Auto-load All" annotations (boxes + polygons) in a single batch.
*   **Backend Integration**: Specific calls to generate annotation previews.

---

## 3. Image Viewer 3 (`src/routes/imageViewer3`)
**Type**: Modern / Modular Implementation

### Overview
The cleanest and most architecturally sound implementation. It refactors the features into small, single-purpose components located within its own directory. It serves as a dashboard for "Viewing" and "Batch Tools".

### Architecture
*   **Controller**: `+page.svelte` (Clean, acts as an elaborator).
*   **Modular Structure**: All dependencies are strictly local to `src/routes/imageViewer3/components/`.
    *   `components/ViewerControls.svelte`: Clean toolbar.
    *   `components/ImageGrid.svelte`: Refactored grid logic.
    *   `components/CropTool.svelte`: A standalone, accordion-style Crop Tool (cleaner than the legacy embedded one).
    *   `components/ExportModal.svelte` & `components/ExtractLabelsModal.svelte`: Dedicated tool modals.

### Key Features
*   **Tool Dashboard**:
    *   **Crop Tool**: Integrated as a collapsible accordion.
    *   **YOLO Export**: Dedicated modal for exporting datasets to YOLO format.
    *   **Label Extraction**: Unique feature to "Extract" specific labels (e.g., "person") into a new dataset.
*   **Viewing**: Optimized grid view with "Column" mode support.
*   **Annotation Support**: Read-only visualization (similar to basic gallery, lacks the Konva editor).

---

## Summary of Differences

| Feature | Dataset Gallery | Advanced Gallery | Image Viewer 3 |
| :--- | :--- | :--- | :--- |
| **Primary Focus** | General Viewing | **Editing (Draw/Modify)** | **Tools & Clean Code** |
| **Architecture** | Component Mix (Global/Local) | Complex (Canvas/Konva) | **Highly Modular** |
| **Editing** | No (Read-only) | **Yes (Konva)** | No (Read-only) |
| **Crop Tool** | Legacy Embedded | None/Hidden | **Modern Accordion** |
| **Export/Extras** | Basic | Unified Export | **Export + Extract Labels** |

### Conclusion
*   **`imageViewer3`** provides the best **User Interface and Code Structure**.
*   **`dataset-gallery-advanced`** provides the critical **Editing Capability**.
*   **`dataset-gallery`** is effectively obsolete.

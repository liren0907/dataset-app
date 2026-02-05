# Frontend Optimization TODO

## Phase 3: Global Adoption (Pending)

The primary objective is to replace manual HTML/CSS styling with the new shared UI components (`Button`, `SplitPaneModal`, etc.) across the entire application to ensure consistency and maintainability.

- [ ] **Landing Page (`src/routes/+page.svelte`)**:
    - Replace manual button classes (e.g., `btn btn-primary`) with `<Button>` components.
    - Standardize layout containers where applicable.

- [ ] **Gallery Navigation (`src/routes/gallery/components/GalleryNavbar.svelte`)**:
    - Update action buttons and inputs to use the new library components.

- [ ] **Other Views**:
    - Audit remaining Svelte files for raw Tailwind usage and refactor as needed.

## Phase 4: Crop Tool Feature Gaps (Pending)

Integrate missing features into the Gallery's `CropRemapTool` to match standalone functionality.

- [ ] **Preview Grid (`PreviewGrid.svelte`)**:
    - Add grid layout to display 5 random annotated samples before processing.
    - Visualize data quality immediately.

- [ ] **Interactive Preview (`KonvaPreviewModal.svelte`)**:
    - Implement full-screen modal with KonvaJS for deep inspection.
    - Support zoom, pan, and annotation checking.

- [ ] **Auto Preview Service**:
    - Implement `autoPreviewAnnotatedImages` in `datasetService.ts`.
    - Connect to backend `generate_annotated_previews` command.

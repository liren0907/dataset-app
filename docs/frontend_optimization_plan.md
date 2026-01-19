# Frontend Optimization Plan: Componentization & Standardization

## 1. Executive Summary
This document outlines the roadmap for the next development stage of the dataset application's frontend. The primary objective is to transition from ad-hoc styling to a **Component-Based Architecture** using Svelte. This will eliminate code duplication, ensure strict UI consistency, and significantly reduce maintenance overhead.

## 2. Current State Analysis
*   **Consistency**: Recent updates standardized the `ExportModal` and `CropRemapTool` to a "Flat Gray" design. However, this consistency relies on duplicating long Tailwind class strings (`btn btn-sm bg-base-200...`) across multiple files.
*   **Maintainability**: Any future design change (e.g., adding hover shadows back) would require finding and replacing code in every single file, which is error-prone.
*   **Scalability**: As the app grows, maintaining visual unity without a shared library will become increasingly difficult.

## 3. Proposed Architecture: Shared UI Library
We will establish a dedicated directory for atomic UI components: `src/lib/components/ui/`.

### 3.1. Core Components

#### `Button.svelte`
A universal button component to handle all interaction styles.
*   **Props**: `variant` (neutral, primary, ghost, danger), `size` (sm, md, lg), `fullWidth` (boolean).
*   **Usage**: `<Button variant="neutral" on:click={...}>Cancel</Button>`
*   **Standard**: Enforces the "Flat Gray" design by default.

#### `SplitPaneModal.svelte`
A layout wrapper for all pop-out tools.
*   **Features**: Fixed 800x600px size, built-in sidebar/content grid, standardized scrolling, and close button logic.
*   **Usage**:
    ```svelte
    <SplitPaneModal title="Export Dataset">
        <div slot="sidebar">...</div>
        <div slot="content">...</div>
    </SplitPaneModal>
    ```

#### `BrowseInput.svelte`
Combines the read-only directory path display with the "Browse" button.
*   **Features**: auto-truncation of long paths, consistent padding/margins for the Browse button.

## 4. Implementation Roadmap

### Phase 1: Infrastructure Setup
*   [ ] Create directory structure `src/lib/components/ui/`.
*   [ ] Implement the atomic `Button.svelte` component.
*   [ ] Implement `BrowseInput.svelte`.
*   [ ] Implement `SplitPaneModal.svelte`.

### Phase 2: Refactoring Existing Tools
*   [ ] **Refactor ExportModal**: Replace raw HTML/Tailwind with `<SplitPaneModal>`, `<BrowseInput>`, and `<Button>`.
*   [ ] **Refactor CropRemapTool**: Apply the same components.
*   [ ] **Verification**: Ensure no visual regression compared to the current hardcoded version.

### Phase 3: Global Adoption (Future)
*   [ ] Audit `+page.svelte` (Landing Page) to replace manual buttons with `<Button>`.
*   [ ] Audit `GalleryNavbar` and other views.

## 5. Benefits
1.  **Single Source of Truth**: Change the button style in *one file*, and the entire app updates instantly.
2.  **Cleaner Code**: Reduces template size by ~30-40% by removing repetitive utility classes.
3.  **Faster Development**: Developers can drag-and-drop components without worrying about pixel-perfect CSS.

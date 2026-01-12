# Gallery Layout Refactor Analysis

## 1. Design Concept Overview
The proposed design shifts the application from a **Grid-First + Modal Editor** workflow to a **Master-Detail (Split View)** workflow.

### Core Layout Changes
*   **Current**: Single column flow. Header -> Controls -> Stats -> Grid. Clicking an image opens a modal.
*   **Proposed**:
    *   **Header Area**: Contains global actions (Change Dir, Export, Stats).
    *   **Main Workspace (Split View)**:
        *   **Left Pane (Gallery List)**: A scrollable sidebar displaying the list of images.
        *   **Right Pane (Editor Workspace)**: A large, persistent canvas for the active image.
    *   **State Persistence**: A "Pending Modifications" bar indicating unsaved changes.

## 2. Component Refactoring Requirements

To achieve this without "inventing" new features, we simply rearrange existing logic:

### A. The Editor (`src/routes/gallery/components/AnnotationEditorModal.svelte`)
*   **Target**: Convert to `AnnotationEditorPane.svelte`.
*   **Change**: Remove the "Modal" wrapper (overlay, fixed positioning, z-index).
*   **Logic**: Keep the `Konva` integration exactly as is. Instead of mounting/unmounting on click, it handles prop changes (`selectedImage`) to update the canvas.

### B. The Gallery (`src/routes/gallery/components/ImageGrid.svelte`)
*   **Target**: Adapt to `GallerySidebar.svelte` or use props to switch styling.
*   **Change**:
    *   Needs to fit in a narrower column (e.g., `w-1/4` or fixed pixel width).
    *   Grid items (thumbnails) might need to be smaller or switch to a "List" style (Image + Details on right) depending on width.
    *   "Selected" state needs to be visually highlighted (border or background color) to show which image is active in the right pane.

### C. The Controller (`src/routes/gallery/+page.svelte`)
*   **Change**:
    *   Replace the vertical flex layout with a **CSS Grid** or **Flex Row** layout.
    *   **State**: `selectedImage` no longer toggles a modal. It determines what renders in the Right Pane.
    *   **Default State**: If no image is selected, the Right Pane should show a "Placeholder / Empty State" (e.g., "Select an image to edit").

### D. The Stats (`src/routes/gallery/components/DatasetSummary.svelte`)
*   **Change**: The design shows this strictly at the top. The current implementation is already good, just needs style tweaks to match the "Card" look in the mockup (cleaner, less background color, more whitespace).

## 3. Implementation Plan
1.  **Extract Editor**: Strip the modal logic from `AnnotationEditorModal` and make it a pure div-based component.
2.  **Reshape Page**: Update `+page.svelte` to use a 2-column layout (Left: Gallery, Right: Editor).
3.  **Refine List**: Style the grid to work well as a sidebar (maybe reduced columns: 1 or 2 wide).
4.  **Connect**: Ensure clicking an item in the Left Pane instantly updates the Right Pane without reloading the page context.

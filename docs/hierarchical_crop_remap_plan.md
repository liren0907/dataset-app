# Hierarchical Crop & Remap Integration Plan

## Objective
To implement a seamless, hierarchical workflow for creating cropped datasets directly from the Gallery's summary view. This allows users to quickly isolate objects (e.g., "persons") based on existing annotations and inspect the results immediately.

## 1. User Experience Flow

1.  **Entry Point**: In the `DatasetSummaryCard` on the Gallery page, the user sees a "Label Hierarchy" or "Class Distribution" section.
2.  **Trigger**: The user clicks on a specific Class Label (e.g., "Person") that they want to process.
3.  **Configuration (Modal)**: A **Pop-out Modal** (based on `AdvancedCropRemapTool`) opens.
    *   The clicked label is auto-selected as the **Parent Label**.
    *   The user selects **Child Labels** (e.g., "Helmet", "Vest") and configures padding.
    *   The user clicks "Start Processing".
4.  **Feedback**: The modal closes (or shows progress), and the main Gallery view updates.
5.  **Result Display**: A new **"Cropped Dataset Card"** appears in a dedicated section below the main summary.
    *   It shows statistics for the newly created dataset (e.g., "150 images created").
    *   **Preview**: A "Preview Outcome" button allows the user to view the cropped images (using `KonvaViewer` or similar) without initiating a full gallery reload.

## 2. Technical Implementation

### Phase 1: Component Refactoring (Advanced Tool)
*   **Goal**: Convert `AdvancedCropRemapTool.svelte` into a reusable, modular component (e.g., `CropRemapModal.svelte`) that allows feature parity with the existing simple tool but includes the **Preview** capabilities.
*   **Features**:
    *   **Konva Preview**: Must integrate `generate_annotated_previews` to show what the crop will look like.
    *   **Input Props**: Accepts `sourceDir` and `preSelectedParentLabel`.
    *   **Events**: Emits `cropCompleted` with details of the new dataset.

### Phase 2: DatasetSummaryCard Enhancements
*   **Target File**: `src/routes/gallery/components/DatasetSummaryCard.svelte`
*   **Changes**:
    *   Enhance the "Label Distribution" section.
    *   Make label badges clickable (or add a specific "Crop" action icon next to them).
    *   Dispatch an event (e.g., `initiateCrop`) when clicked, passing the label name.

### Phase 3: Cropped Results Management
*   **New Component**: `CroppedDatasetCard.svelte`
    *   Similar visual style to `DatasetSummaryCard` but compact.
    *   Displays: Output Path, Image Count, Class Count.
    *   Action: "Preview" button (opens a modal with random samples from this new directory).
*   **Page State**:
    *   `+page.svelte` (or a store) will maintain a list of `croppedDatasets`.
    *   When the Crop Modal emits `cropCompleted`, add the result to this list.

### Phase 4: Integration
*   Update `+page.svelte` to:
    *   Listen to `initiateCrop` from `DatasetSummaryCard`.
    *   Open `CropRemapModal`.
    *   Handle the success result and append to the "Cropped Results" section.

## 3. Visual Reference (Concept)

```
[ Main Dataset Summary Card ]
| Total: 1000 | Annotated: 800 |
| Labels: [Person (Crop)] [Car (Crop)] [Dog (Crop)] |  <-- Click to Trigger
```

*... User runs crop on "Person" ...*

```
[ Cropped Datasets Section ]
+--------------------------------------------------+
| Reference: /exports/person_crop_20231024         |
| Images: 150 | Classes: 2 (Helmet, Vest)          |
| [ PREVIEW COMPLETED DATASET ] [ OPEN IN GALLERY ]|
+--------------------------------------------------+
```

## 4. Risks & Considerations
*   **Performance**: Generating previews for large cropped datasets might be slow. We should limit the preview to ~5-10 random images.
*   **State Persistence**: If the user refreshes `+page.svelte`, the "Cropped Results" list might disappear unless stored in a persistent store or local storage. For now, ephemeral state is acceptable.

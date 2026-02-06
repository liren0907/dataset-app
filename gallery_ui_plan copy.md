# Gallery UI Unification Plan

## Goal
Standardize the UI components within the `/gallery` route to ensure design consistency, reduce code duplication, and improve maintainability. This plan builds upon the "Flat Gray" aesthetic established in the `ExportModal` and `CropRemapTool` refactoring.

## 1. Existing Shared Components (Ready)
The following components exist in `src/lib/components/ui/` and are ready for wider adoption:
- `Button` (Flat Gray style)
- `BrowseInput` (Directory selection with icon)
- `SectionLabel` (Uppercase header labels)
- `SelectionCard` (Selectable option cards)
- `SplitPaneModal` (35/65 split layout)

## 2. New Components to Create
Identify and extract repeated patterns in `GalleryNavbar` and other modals into shared components.

### 2.1 IconButton
**Purpose**: Replace the frequent `btn btn-ghost btn-sm btn-square` pattern in the navbar.
**Features**:
- Icon slot or prop
- Tooltip integration
- Active/Inactive states (e.g., Mock Mode toggle)
- Loading state

### 2.2 ToggleButtonGroup
**Purpose**: Standardize segmented controls like "View Mode" (Grid/List) and "Annotation Type" (Box/Polygon).
**Features**:
- List of options
- Active state styling (`bg-base-200 shadow-inner`)
- Icons and Labels

### 2.3 Alert
**Purpose**: Standardize error and success messages found in modals.
**Features**:
- Variants: `error`, `success`, `info`, `warning`
- Icon integration
- Close functionality (optional)

### 2.4 LabelBadge
**Purpose**: Standardize label tags used in `ExtractLabelsModal` and filter chips.
**Features**:
- Clickable/Toggleable
- Active/Inactive styles (strikethrough for excluded)
- Count display

### 2.5 SimpleModal
**Purpose**: A generic modal wrapper for non-split layouts (like `ExtractLabelsModal`).
**Features**:
- Title
- Close button
- Click-outside-to-close

## 3. Refactoring Targets

### Phase 1: Navbar Unification
**Target**: `src/routes/gallery/components/GalleryNavbar.svelte`
- Replace manual icon buttons with `IconButton`
- Replace manual toggle groups with `ToggleButtonGroup`
- **Benefit**: drastically reduces template boilerplate in the navbar.

### Phase 2: Modal Cleanup
**Target**: `src/routes/gallery/components/ExtractLabelsModal.svelte`
- Adopt `SimpleModal` wrapper
- Use `Button` for actions
- Use `LabelBadge` for label selection list
- Use `Alert` for error messages

### Phase 3: Split Pane Standardization
**Target**: `ExportModal.svelte` and `CropRemapTool.svelte`
- Currently, these implement their own split layout structure manually.
- **Action**: Refactor them to wrap their content in the shared `SplitPaneModal` component.
- **Benefit**: Ensures exact layout consistency (width ratios, padding, scrollbars) across all complex tools.

## 4. Execution Roadmap

1.  **Create New Components**: Implement `IconButton`, `ToggleButtonGroup`, `Alert`, `LabelBadge`, `SimpleModal` in `src/lib/components/ui/`.
2.  **Refactor Navbar**: detailed pass on `GalleryNavbar.svelte`.
3.  **Refactor ExtractLabelsModal**: detailed pass on `ExtractLabelsModal.svelte`.
4.  **Adopt SplitPaneModal**: Update `ExportModal` and `CropRemapTool` to use the shared layout wrapper.

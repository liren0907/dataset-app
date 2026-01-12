# Dataset Gallery Layout Documentation

## Overview
The **Dataset Gallery** (`src/routes/gallery/+page.svelte`) implements a **Master-Detail** interface designed to maximize screen real estate for image viewing and annotation while maintaining easy access to navigation controls.

## Layout Architecture

### 1. The Container Strategy
The page does **not** rely on the default document flow. Instead, it uses a fixed-height strategy to ensure the gallery has its own internal scrolling, separate from the main application window.

- **Critical CSS Constraint**: `h-[calc(100vh-6rem)]`
    - **Why?**: The global application Layout (`+layout.svelte`) adds padding/margins equivalent to approximately `6rem` (header offset + padding).
    - **Issue with `h-screen`**: Setting `h-screen` (100vh) forces the container to be 100% of the viewport height. When combined with the Layout's padding, this pushes the bottom of the container *off-screen*, creating double scrollbars and potential UI blocking (z-index issues).
    - **Solution**: We calculate the exact remaining height so the container fits perfectly within the viewport without overflowing.

### 2. Component Hierarchy

The layout is divided into two main vertical sections:

#### A. Header (Fixed Height)
- **Role**: Contains global controls (Directory Selection, View Settings) and the `DatasetSummary` dashboard.
- **Behavior**: `flex-none`. It takes up as much height as its content needs, but never shrinks.

#### B. Workspace (Flexible Height)
- **Role**: The main working area.
- **Behavior**: `flex-1` + `overflow-hidden`. This section expands to fill all remaining vertical space after the header. It creates a "boundary" for the internal scrollable areas.

### 3. Split-Pane Workspace

Inside the **Workspace**, the layout splits horizontally:

#### Left: Sidebar (Gallery List)
- **Width**: Fixed (`w-[320px]` or `lg:w-[400px]`).
- **Scrolling**: `overflow-y-auto`. Only this specific column scrolls, preserving the user's scroll position in the list even while editing an image.
- **Content**: The `ImageGrid` component.

#### Right: Editor Pane (Detail View)
- **Width**: `flex-1`. Takes up all remaining horizontal width.
- **Behavior**: `min-w-0` (Critical fix for Flexbox nested overflow issues).
- **Content**: The `AnnotationEditorPane`, which contains the KonvaJS canvas.

## CSS Reference
```html
<!-- Main Container -->
<div class="h-[calc(100vh-6rem)] flex flex-col ...">

    <!-- Header -->
    <div class="flex-none ...">...</div>

    <!-- Workspace -->
    <div class="flex-1 flex overflow-hidden">
        
        <!-- Left Sidebar -->
        <div class="w-[400px] flex-none overflow-y-auto ...">...</div>

        <!-- Right Editor -->
        <div class="flex-1 relative ...">...</div>

    </div>
</div>
```

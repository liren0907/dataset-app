# Unified Gallery Plan & Conflict Report

> [!IMPORTANT]
> This document consolidates information from `GALLERY_LAYOUT_DOCS.md` and `GALLERY_OVERVIEW.md`, highlighting current discrepancies between the vision and the code.

## 🎯 The Vision: "Unified Gallery"
The goal is to establish `src/routes/gallery` as the single, authoritative interface for dataset management, combining the best features of previous implementations into a professional, desktop-class experience.

### Core Architecture (Target State)
We aim to refactor the current page into a **Fixed-Height Master-Detail Layout**:
-   **No Global Scroll**: The window itself never scrolls.
-   **Split Interface**:
    -   **Left Sidebar (List)**: Independently scrollable grid of images.
    -   **Right Pane (Editor)**: Large, static area for viewing/editing the selected image.
    -   **Top Header**: Fixed toolbars for filtering, mode switching, and mock data.

---

## 🚨 Current Conflicts & Status

### 1. The Layout Conflict
| Documented Vision | Actual Code (`src/routes/gallery`) | Status |
| :--- | :--- | :--- |
| **Fixed Height** (`h-screen`) | **Scrolling Page** (`min-h-screen`) | ❌ **Conflict** |
| **Split Pane** (Sidebar/Editor) | **Single View** (Grid OR Modal) | ❌ **Conflict** |
| **Internal Scrolling** | **Window Scrolling** | ❌ **Conflict** |

**Impact**: The current layout feels like a "webpage" rather than a "desktop app". Large datasets cause the toolbar to disappear when scrolling down.

### 2. The Identity Conflict
| Old Docs Claim... | Reality | Status |
| :--- | :--- | :--- |
| `src/routes/gallery` is "Legacy" | It is the **Active Development** branch | ✅ **Evolved** |
| `src/routes/gallery` is "Read-only" | It now has **Crop, Export, & Auto-Label** | ✅ **Evolved** |
| `imageViewer3` is the "Future" | Features are being migrated *from* it to `gallery` | 🔄 **Merging** |

---

## 🛠 Feature Roadmap

### Completed ✅
-   **Modern Toolbar**: New standard navbar with breadcrumbs and tool toggles.
-   **Mock Data System**: Toggle between real filesystem and mock JSON.
-   **Smart Tools Integration**: Auto-labeling and LabelMe summary generation.
-   **Export Tools**: "Turbo Export" and "Extract Labels" modals are integrated.

### Missing / To-Do 🚧
1.  **Layout Refactor**: Implement the `h-[calc(100vh-6rem)]` fixed container strategy.
2.  **Split Pane Implementation**: Move from "Modal View" to "Side-by-Side View" for faster annotation workflow.
3.  **Konva Integration**: The "Advanced Gallery" editing (drawing boxes/polygons) is still missing from this unified route.

## CSS Strategy for Refactor
To fix the layout conflict, we will apply this structure:
```html
<!-- Wrapper: Fixes height to viewport minus header -->
<div class="h-[calc(100vh-6rem)] flex flex-col overflow-hidden">
    
    <!-- 1. Toolbar (Fixed) -->
    <div class="flex-none z-10">...</div>

    <!-- 2. Workspace (Expands) -->
    <div class="flex-1 flex overflow-hidden">
        
        <!-- Left: Image Grid (Scrolls independently) -->
        <div class="w-96 flex-none overflow-y-auto">...</div>

        <!-- Right: Editor/Viewer (Static) -->
        <div class="flex-1 relative bg-base-200">...</div>

    </div>
</div>
```

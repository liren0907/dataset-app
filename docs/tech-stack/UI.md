# Project UI Tech Stack

This document outlines the standard UI libraries and icon sets used in the `dataset-app` project. Please adhere to these standards when adding new features to maintain consistency.

## 1. UI Component Library
**[DaisyUI](https://daisyui.com/)**
*   **Framework**: Based on **Tailwind CSS**.
*   **Usage**: We use DaisyUI classes for all core components (Buttons, Cards, Inputs, Modals, Badges).
*   **Theme**: The application uses a custom Tailwind theme configuration (see `tailwind.config.js`).

> [!NOTE]
> Ensure consistent use of DaisyUI components to maintain a unified look and feel. Avoid mixing custom CSS styles with DaisyUI classes unless absolutely necessary for specific custom behaviors.

**Example Usage:**
```html
<!-- Primary Button -->
<button class="btn btn-primary">Click Me</button>

<!-- Card -->
<div class="card bg-base-100 shadow-xl">
  <div class="card-body">
    <h2 class="card-title">Card Title</h2>
    <p>Content goes here.</p>
  </div>
</div>
```

## 2. Icon System
**[Google Material Symbols](https://fonts.google.com/icons) (Rounded)**
*   **Variant**: `Material Symbols Rounded`.
*   **Implementation**: Loaded via Google Fonts CDN in `src/app.html`.
*   **Usage**: Use the `material-symbols-rounded` class on a `span` element.

> [!IMPORTANT]
> Always use the **Rounded** variant of Material Symbols to match the application's soft UI aesthetic. Do not use the Sharp or Outlined variants.

**Example Usage:**
```html
<!-- Settings Icon -->
<span class="material-symbols-rounded">settings</span>

<!-- Edit Icon with specific size -->
<span class="material-symbols-rounded text-lg">edit</span>
```

## 3. Other Dependencies
*   **Flowbite Svelte**: Installed but primarily used as a fallback or for specific headless interactive components not covered by DaisyUI.
*   **Konva.js**: Used for canvas-based annotation rendering.
*   **Svelte Plotly**: Used for statistical charts in dashboards.

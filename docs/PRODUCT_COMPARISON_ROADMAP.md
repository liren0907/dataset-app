# Product Comparison & Future Roadmap: Fabric Annotator vs. Industry Standards

## 1. Why is there a difference? (The "Gap" Analysis)

From a frontend perspective, the differences between `fabric-annotator` and tools like Label Studio, CVAT, or Roboflow stem from three main pillars: **Layout Architecture**, **Object Management Complexity**, and **AI-First Design**.

### A. Layout Architecture (Fixed vs. Dynamic)
*   **Current Tool**: Uses a classic "Dashboard" layout (fixed sidebar, fixed toolbar). This is easy to implement but lacks flexibility for complex tasks.
*   **Industry Standards**:
    *   **Label Studio**: Uses a **Slot/XML-based UI**. The interface itself is a data structure, allowing users to move panels, collapse them, and even define custom layouts for different tasks (Audio vs. Video).
    *   **CVAT**: Optimized for "High-Density Data". It uses floating toolbars and highly specialized property panels that don't take up permanent screen space unless needed.

### B. Object Management (List vs. Outliner)
*   **Current Tool**: A flat list of annotations in the sidebar. This works for ~10 objects but breaks when an image has 200+ tiny objects (like a drone shot or satellite image).
*   **Industry Standards**: They use an **"Outliner" or "Tree View"**.
    *   Supports grouping, locking, and visibility toggling.
    *   Supports **Attributes** (e.g., OCcluded: True, Truncated: 20%).
    *   Supports hierarchy (e.g., "Car" as parent, "Wheel" as child keypoint).

### C. The "Zero-Interaction" Goal (AI-Assisted)
*   **Industry Standards**: They are moving away from manual drawing.
    *   **Roboflow/SAM**: You don't "draw" a polygon anymore; you click 3 points and the AI snaps the contour.
    *   **Interpolation**: In CVAT, you draw once in frame 1 and once in frame 10; the UI handles the 8 frames in between.

---

## 2. Recommended Future Roadmap

To bring `fabric-annotator` closer to professional standards, I suggest focusing on these four areas:

### Phase 1: Interactive Workspace (UI/UX Foundation)
- [ ] **Collapsible Sidebars**: Allow the gallery and annotation list to collapse to maximize drawing space.
- [ ] **Property Inspector**: Instead of just a dropdown for label, show a "Properties" panel when an object is clicked (Manual X/Y/W/H entry, Attributes like 'Occluded').
- [ ] **Hot-swap Themes**: High-contrast modes for medical or satellite imagery.

### Phase 2: Advanced Annotation Management
- [ ] **The "Outliner"**: Implement a virtualized list to handle hundreds of annotations efficiently.
- [ ] **Object Hierarchies**: Support for "Groups". For example: Grouping 5 keypoints as a "Person" object.
- [ ] **Batch Labeling**: Select multiple boxes and change their labels all at once.

### Phase 3: AI-Assisted "Magic" Tools
- [ ] **Segment Anything (SAM) Integration**: Use the `AI_INFERENCE_PLAN.md` to implement a "Magic Wand" tool. This is the single biggest "Wow" factor in modern tools.
- [ ] **Auto-Tracking**: For temporal data (video), use simple algorithms like CSRT or KCF to move boxes automatically as the object moves.

### Phase 4: Data Insights & Curation
- [ ] **Dataset Stats**: A dedicated "Analysis" tab showing class distribution (e.g., "You have 500 Dogs but only 5 Cats").
- [ ] **Quality Control (Review Mode)**: A specialized UI for a "Reviewer" to approve/reject annotations with comments.

---

## 3. Technical Suggestions for your Stack

Since we are using **Svelte + Fabric.js + Tauri**:

1.  **Canvas Performance**: If you start handling 1000+ objects, Fabric's "object-based" nature might slow down. Look into `requestRenderAll` and potentially custom caching for non-active objects.
2.  **Tauri OS Integration**: Use Tauri to open a "Magnifier" window or a "Mini Map" (Oerview) that uses native window scaling for high-performance zooming.
3.  **SAM on Rust side**: Keep the SAM model inference in Rust (via `ort`) to keep the UI thread buttery smooth.

# Dataset App

A desktop application for exploring and processing image datasets annotated in LabelMe, with tools for visualization, conversion to YOLO, label extraction, basic auto-annotation, and crop/remap utilities. Built with SvelteKit (frontend) and Tauri v2 (Rust backend).

## Status and Caution
- This project is under rapid development.
- APIs, commands, and UI may change frequently and introduce breaking changes.

## Features
- Dataset browsing with pagination and lazy previews
- Image details and on-demand metadata loading
- LabelMe dataset summary (label counts, annotation types, totals)
- Export to YOLO (train/val/test splits, shape type, specific labels)
- Extract specific labels from LabelMe datasets
- Convert YOLO → LabelMe
- Draw bounding boxes or polygons into rendered images
- Visualization-only scan without saving outputs
- Auto-annotate skeleton (backend hook for extensions)
- Crop and remap annotations around a parent label with padding

## Getting Started


### Install dependencies and build
```bash
yarn install
yarn tauri dev
```
This will:
- Start SvelteKit dev server on `http://localhost:1420`
- Launch the Tauri shell pointing at the dev server

### Build Production Bundle
```bash
yarn tauri build
```
This will:
- Build SvelteKit to `build/`
- Produce platform bundles via Tauri (see `src-tauri/tauri.conf.json`)
- For more details on building and bundling, see the Tauri 2 docs: [https://v2.tauri.app/](https://v2.tauri.app/)



## Project Structure
```
.
├─ build/                       # SvelteKit output
├─ docs/                        # Design and analysis docs
├─ src/                         # Frontend (SvelteKit)
│  ├─ app.html, app.css
│  ├─ lib/                      # Components and services
│  │  ├─ services/datasetService.ts
│  │  └─ ... Svelte components
│  ├─ routes/                   # Pages (+page.svelte / +page.ts)
│  └─ funcs/                    # Plain JS helpers
├─ src-tauri/                   # Backend (Rust, Tauri v2)
│  ├─ src/main.rs               # Tauri commands and wiring
│  ├─ src/*.rs                  # Modules: handlers, converters, drawers
│  ├─ Cargo.toml                # Rust dependencies
│  └─ tauri.conf.json           # Tauri app config
├─ static/                      # Static assets
└─ stored/                      # Archived iterations (not active)
```

## Key Commands (Rust Backend)
Exposed via Tauri and consumed by the frontend:
- Dataset I/O
  - `get_directory_images`, `get_paginated_images`, `get_image_details`
  - `get_labelme_summary`
- Drawing & Visualization
  - `draw_bounding_boxes`, `draw_polygons`, `visualize_dataset`
- Conversions & Export
  - `export_to_yolo`, `labelme2yolo::export_to_yolo_new`
  - `convert_to_labelme`
  - `extract_labels`
- Maintenance
  - `change_label_name`, `clear_image_data`
- Media helpers
  - `get_video_info`, `read_video_file`, `read_image_file`
- Tools
  - `auto_annotate_images`
  - `crop_and_remap_annotations`

See `src-tauri/src/main.rs` for the full list and signatures.


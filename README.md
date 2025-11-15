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
```

## Key Commands (Rust Backend)
**If you're interested in the Rust backend architecture and Tauri command details, please refer to [`docs/RUST_BACKEND.md`](docs/RUST_BACKEND.md) for comprehensive documentation including hierarchical code structure, function signatures, and implementation details.**

See `src-tauri/src/main.rs` for the full list and signatures.


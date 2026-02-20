# Changelog

## [0.1.0] - 2026-02-21
### Added
- **Gallery Refactor**: New unified gallery UI (`/gallery`) with lazy previews and integrated metadata.
- **LabelMe Engine**: High-performance, asynchronous Rust engine for COCO/YOLO conversion.
- **Smart Remapping**: New tools for hierarchical cropping and parent/child boundary adjustments.

### Changed
- **Backend Renamed**: Tauri Rust backend library renamed to `dataset_app_lib`.
- **Rust Edition upgraded**: Updated to Rust Edition 2024 for improved pattern matching.
- **Routes structure**: Migrated older tools to `/legacy-gallery/*` to keep the main view clean.

# Frontend Routes to Backend Commands Mapping

This document provides a comprehensive mapping between the SvelteKit frontend routes (and their core library services) and the Rust Tauri backend commands they invoke.

## Route Map

| Route / Module | Description | Tauri Backend Commands Invoked |
| :--- | :--- | :--- |
| `/gallery` | The main, unified dataset viewing & processing hub (powered by `src/lib/services/gallery/datasetService.ts` and `src/lib/stores/gallery/exportStore.ts`). | `get_paginated_images`, `get_image_details`, `get_labelme_summary`, `auto_annotate_images`, `generate_annotated_previews`, `generate_single_annotated_preview`, `crop_and_remap_annotations`, `export_to_yolo_new`, `extract_labels`, `path_exists`, `copy_directory` |
| `/turbo-export` | Dedicated asynchronous route for high-performance LabelMe dataset structure discovery and YOLO/COCO format exporting. | `scan_labelme_labels`, `scan_labelme_labels_with_counts`, `analyze_labelme_dataset`, `convert_labelme`, `scan_labelme_labels_async`, `scan_labelme_labels_with_counts_async`, `analyze_labelme_dataset_async` |
| `/legacy-gallery/dataset-gallery` | Legacy read-only gallery view for datasets. | `get_paginated_images`, `get_image_details`, `get_labelme_summary` |
| `/legacy-gallery/crop-remap` | Legacy UI for dataset subset extraction and label remapping. | `get_labelme_summary`, `crop_and_remap_annotations`, `generate_annotated_previews` |
| `/legacy-gallery/imageViewer3` | Legacy image viewer with inline annotation tooling. | `get_dataset_summary`, `load_images_page`, `load_image_annotations`, `crop_and_remap_annotations`, `export_dataset_to_yolo`, `extract_labels_from_dataset` |
| `/legacy-gallery/dataset-gallery-advanced` | Legacy advanced processing gallery. | Similar invocations mapped via legacy `datasetService.ts` (`get_paginated_images`, `get_image_details`, `auto_annotate_images`, `crop_and_remap_annotations`, `export_to_yolo_new`, `extract_labels`, `get_labelme_summary`) |
| `/smart-tools/*` | Various automated annotation utilities. | *(Present in directory structure, mostly client-side logic or depends on `load_image_annotations` patterns)* |

## Architecture Notes

*   **Service Layer Extraction:** As part of recent refactoring, most backend invocations for the primary `/gallery` route have been abstracted away from direct Svelte components and moved into `src/lib/services/gallery/` and `src/lib/stores/gallery/`. 
*   **LabelMe Conversion Engine:** The `/turbo-export` route acts as the exclusive consumer of the newly introduced `labelme_convert` Rust module, leveraging both its synchronous metadata commands and its asynchronous (`*_async`) progress-emitting conversion utilities.

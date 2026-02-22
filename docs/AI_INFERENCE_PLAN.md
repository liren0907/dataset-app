# AI-Assisted Annotation: Development Plan

> **Goal**: Integrate real AI model inference into the Rust backend so users can auto-detect objects or segment with a single click, directly from the `fabric-annotator` UI.

## Current State

The infrastructure is already in place:

- **Rust**: `src-tauri/src/commands/smart_tools.rs` contains a placeholder `auto_detect_objects` command with request/response types defined.
- **Frontend**: `FabricAnnotator.svelte` already imports and can call Tauri commands.
- **Dependencies**: The project already uses `tokio` (async), `rayon` (parallelism), `image` crate, and `opencv`.

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│  Frontend (Svelte + Fabric.js)                               │
│                                                              │
│  [User clicks "Auto Detect" or Ctrl+D]                       │
│       │                                                      │
│       ▼                                                      │
│  invoke("auto_detect_objects", { image_path, threshold })    │
│       │                                                      │
└───────┼──────────────────────────────────────────────────────┘
        │ Tauri IPC
┌───────▼──────────────────────────────────────────────────────┐
│  Rust Backend                                                │
│                                                              │
│  smart_tools::auto_detect_objects()                           │
│       │                                                      │
│       ├─► ModelManager::get_or_load("yolov8n.onnx")          │
│       │       (lazy_static, loads once, reuses)              │
│       │                                                      │
│       ├─► preprocess(image_path) → input tensor              │
│       │       (resize to 640x640, normalize, CHW layout)     │
│       │                                                      │
│       ├─► session.run(input) → raw output                    │
│       │                                                      │
│       ├─► postprocess(output, threshold)                     │
│       │       (decode boxes, apply NMS, map class IDs)       │
│       │                                                      │
│       └─► Return Vec<DetectedObject> as JSON                 │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## Phase A: Object Detection (YOLOv8)

### A1. Add ONNX Runtime to Rust

**File**: `src-tauri/Cargo.toml`

```toml
[dependencies]
ort = { version = "2", features = ["download-binaries"] }  # ONNX Runtime wrapper
ndarray = "0.16"  # For tensor manipulation
```

> **Why `ort`?** It's the most mature Rust ONNX Runtime binding. The `download-binaries` feature auto-downloads the correct ONNX Runtime shared library for the target platform at build time.

### A2. Obtain a Pre-trained ONNX Model

- Download `yolov8n.onnx` (nano, ~6MB) from Ultralytics or export via:
  ```bash
  pip install ultralytics
  yolo export model=yolov8n.pt format=onnx imgsz=640
  ```
- Store the model in: `src-tauri/models/yolov8n.onnx`
- Add to `.gitignore` if too large; provide a download script instead.

### A3. Implement ModelManager (Rust)

**File**: `src-tauri/src/core/model_manager.rs`

```rust
use lazy_static::lazy_static;
use ort::{Session, SessionBuilder, GraphOptimizationLevel};
use std::sync::Mutex;

lazy_static! {
    static ref YOLO_SESSION: Mutex<Option<Session>> = Mutex::new(None);
}

pub fn get_or_load_yolo(model_path: &str) -> Result<(), String> {
    let mut session = YOLO_SESSION.lock().map_err(|e| e.to_string())?;
    if session.is_none() {
        let s = SessionBuilder::new()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .with_model_from_file(model_path)
            .map_err(|e| format!("Failed to load model: {}", e))?;
        *session = Some(s);
    }
    Ok(())
}
```

### A4. Implement Preprocessing

**File**: `src-tauri/src/core/inference.rs`

Key steps:
1. Read image via `image` crate
2. Resize to 640×640 (letterbox with padding)
3. Convert to `f32`, normalize to `[0, 1]`
4. Transpose to CHW layout → `ndarray::Array4<f32>` with shape `[1, 3, 640, 640]`
5. Track scale factors and padding offsets for coordinate mapping back

### A5. Implement Postprocessing

Key steps:
1. Parse YOLOv8 output tensor (shape: `[1, 84, 8400]` for COCO 80 classes)
2. Transpose to `[8400, 84]` — each row = `[cx, cy, w, h, class_scores...]`
3. Filter by confidence threshold
4. Apply Non-Maximum Suppression (NMS) with IoU threshold ~0.45
5. Map box coordinates back to original image dimensions
6. Map class IDs to label names via a class list

### A6. Update `smart_tools.rs`

Replace the placeholder in `auto_detect_objects` with:
```rust
pub async fn auto_detect_objects(
    image_path: String,
    confidence_threshold: Option<f32>,
) -> Result<String, String> {
    let start = std::time::Instant::now();
    let threshold = confidence_threshold.unwrap_or(0.5);

    // 1. Ensure model is loaded
    let model_path = get_model_path("yolov8n.onnx");
    model_manager::get_or_load_yolo(&model_path)?;

    // 2. Preprocess
    let input = inference::preprocess(&image_path)?;

    // 3. Run inference (on blocking thread to avoid blocking Tauri)
    let output = tokio::task::spawn_blocking(move || {
        inference::run_yolo(&input)
    }).await.map_err(|e| e.to_string())??;

    // 4. Postprocess
    let detections = inference::postprocess(&output, threshold, &image_path)?;

    let response = AutoDetectResponse {
        objects: detections,
        model: "yolov8n".into(),
        processing_time_ms: start.elapsed().as_millis() as u64,
    };

    serde_json::to_string(&response).map_err(|e| e.to_string())
}
```

### A7. Frontend Integration

**File**: `FabricAnnotator.svelte`

Add an "Auto Detect" button to the toolbar:
```typescript
async function autoDetect() {
    if (!currentImagePath) return;
    const result = await invoke("auto_detect_objects", {
        imagePath: currentImagePath,
        confidenceThreshold: 0.5,
    });
    const parsed = JSON.parse(result as string);
    // Convert detected objects to BBox format and add to canvas
    for (const obj of parsed.objects) {
        fabricManager?.addBBox(obj.x1, obj.y1, obj.x2, obj.y2, obj.label);
    }
}
```

---

## Phase B: Interactive Segmentation (SAM)

### B1. Model Selection

| Model | Size | Speed | Use Case |
|-------|------|-------|----------|
| `mobile_sam.onnx` | ~40MB | ~50ms | Recommended for desktop |
| `sam_vit_b.onnx` | ~375MB | ~200ms | Higher quality |
| `sam2_tiny.onnx` | ~38MB | ~30ms | Latest, best tradeoff |

### B2. Two-Stage Architecture

SAM uses a two-stage approach:
1. **Image Encoder** (run once per image): produces image embeddings (~256×64×64)
2. **Prompt Decoder** (run per click): takes point/box prompts + embeddings → mask

This means:
- Cache the image embedding when an image is loaded
- Each user click only runs the lightweight decoder (~10ms)

### B3. Frontend Interaction Flow

```
1. User enables "Magic Wand" tool (keyboard: W)
2. User clicks on an object → sends (x, y) point to Rust
3. Rust runs SAM decoder with the cached embedding
4. Rust returns polygon points (contour of the mask)
5. Frontend renders as a Fabric.js polygon
6. User can refine with additional positive/negative clicks
```

### B4. Implementation Files

| File | Purpose |
|------|---------|
| `core/sam_manager.rs` | SAM model loading, embedding cache |
| `core/sam_inference.rs` | Encode image, decode with prompts |
| `commands/smart_tools.rs` | New command: `sam_segment` |
| `SmartToolbar.svelte` | Magic wand UI + click handler |

---

## Phase C: Model Management UI

### C1. Model Download Manager

- Check `~/.dataset-app/models/` for available models
- Provide a UI in Settings to download/delete models
- Show download progress via Tauri events

### C2. Confidence Threshold Slider

- Add a slider to the toolbar (default: 0.5)
- Adjustable per-use, persisted in app settings

### C3. Class Mapping

- Map COCO class IDs to the user's custom taxonomy
- Allow users to configure which detected classes to keep

---

## Dependencies Summary

```toml
# Add to Cargo.toml
ort = { version = "2", features = ["download-binaries"] }
ndarray = "0.16"
```

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| ONNX Runtime binary size (~50MB) | Accept for desktop app; use `download-binaries` feature |
| Cross-platform build complexity | `ort` handles this via auto-download; test on macOS/Windows/Linux |
| Model file distribution | Don't bundle in git; provide first-run download or manual placement |
| GPU acceleration | Start with CPU; `ort` supports CoreML (macOS) and CUDA via features |
| Memory usage with large models | Use `mobile_sam` / `yolov8n` as defaults; allow advanced users to swap |

## Recommended Execution Order

1. **A1–A3**: Add `ort` + `ndarray`, implement `ModelManager` — verify ONNX loads
2. **A4–A5**: Preprocessing + postprocessing — verify with a test image via CLI
3. **A6–A7**: Wire to Tauri command + frontend button — end-to-end test
4. **B1–B4**: SAM integration (after YOLO is proven)
5. **C1–C3**: Polish (model download UI, settings)

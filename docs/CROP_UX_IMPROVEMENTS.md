# Hierarchical Crop UX Improvements

## Problem
When user clicks **Run Crop**, the backend processes images which takes time. Current UX only shows a loading spinner with no indication of progress or estimated completion time.

---

## Proposed Improvements

### Phase 1: Quick Wins (Low Complexity)

#### 1.1 Descriptive Processing Message
Show what's being processed instead of just a spinner.

**Current:**
- Button shows loading spinner only

**Proposed:**
```
🔄 Cropping 45 "person" instances with 3 child labels...
```

**Implementation:**
- Add `processingMessage` state variable
- Update message when `runCrop` starts
- Display below the button during processing

---

#### 1.2 Time Estimate
Show estimated processing time based on image count.

**Example:**
```
⏱️ Estimated time: ~30 seconds (50 images)
```

**Implementation:**
- Calculate estimate: `imageCount * 0.5 seconds` (adjust based on testing)
- Show estimate when user selects parent label
- Update during processing with elapsed time

---

#### 1.3 Toast Notification on Complete
Show a toast when crop completes successfully.

**Example:**
```
✅ Cropped 45 images successfully!
   Saved to temporary folder.
   [View Results]
```

**Implementation:**
- Use existing Alert component or add toast library
- Auto-dismiss after 5 seconds
- Include action button to scroll to results

---

### Phase 2: Interaction Safety (Medium Complexity)

#### 2.1 Prevent Accidental Close
Disable close/collapse of HierarchicalCrop panel during processing.

**Implementation:**
- Add `isProcessing` prop to parent component
- Disable toggle button while processing
- Show tooltip: "Cannot close while processing"

---

#### 2.2 Prevent Dataset Switch
Block switching to original/other datasets during crop.

**Implementation:**
- Disable "Back to Original" button during processing
- Disable dataset selection in navbar
- Show warning if user tries to navigate away

---

### Phase 3: Real-time Progress (High Complexity)

#### 3.1 Progress Bar with Percentage
Show actual progress as backend processes each image.

**Example:**
```
[████████░░░░░░░░] 45% (23/50 images)
```

**Implementation:**
- **Backend:** Emit Tauri events for progress updates
- **Frontend:** Listen to events, update progress state
- Add progress bar component

**Backend Changes Required:**
```rust
// In annotation_processor.rs
app_handle.emit_all("crop_progress", payload)?;
```

---

## Priority Matrix

| Improvement | Impact | Effort | Priority |
|-------------|--------|--------|----------|
| 1.1 Descriptive Message | High | Low | ⭐⭐⭐ |
| 1.2 Time Estimate | Medium | Low | ⭐⭐⭐ |
| 1.3 Toast Notification | High | Low | ⭐⭐⭐ |
| 2.1 Prevent Close | Medium | Medium | ⭐⭐ |
| 2.2 Prevent Switch | Medium | Medium | ⭐⭐ |
| 3.1 Progress Bar | High | High | ⭐ |

---

## Recommended Implementation Order

1. **Phase 1.1 + 1.2** - Descriptive message + time estimate (same PR)
2. **Phase 1.3** - Toast notification
3. **Phase 2** - Safety features (if time permits)
4. **Phase 3** - Progress bar (future enhancement)

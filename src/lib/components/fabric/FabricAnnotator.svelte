<script lang="ts">
    import { onMount } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import { invoke, convertFileSrc } from "@tauri-apps/api/core";
    import {
        FabricManager,
        type Mode,
        type BBox,
        type Polygon,
        type Polyline,
        type Keypoint,
        type LabelMeJSON,
        type ImageEntry,
    } from "$lib/logic/FabricManager";
    import AnnotationSidebar from "$lib/components/fabric/AnnotationSidebar.svelte";
    import PropertyInspector from "$lib/components/fabric/PropertyInspector.svelte";
    import DatasetStats from "$lib/components/fabric/DatasetStats.svelte";
    import FabricToolbar from "$lib/components/fabric/FabricToolbar.svelte";
    import ImageGallery from "$lib/components/fabric/ImageGallery.svelte";
    import { shortcutMap } from "$lib/stores/labelTaxonomyStore";
    import { imageStatusStore } from "$lib/stores/imageStatusStore";
    import { fly } from "svelte/transition";

    let parentElement: HTMLDivElement;
    let canvasElement: HTMLCanvasElement;
    let fabricManager: FabricManager | null = null;

    let mode: Mode = "select";
    let bBoxes: BBox[] = [];
    let polygons: Polygon[] = [];
    let polylines: Polyline[] = [];
    let keypoints: Keypoint[] = [];
    let isPolygonDrawing = false;
    let isPolylineDrawing = false;
    let imageLoaded = false;

    // Directory mode state
    let directoryPath = "";
    let imageList: ImageEntry[] = [];
    let currentImageIndex = -1;
    $: directoryMode = imageList.length > 0;

    // Layout state
    let showGallery = true;
    let showSidebar = true;
    let showStats = false;
    let selectedObject: any = null;
    let selectedObjects: any[] = [];

    // Autosave
    let autosaveTimer: ReturnType<typeof setTimeout> | null = null;
    let currentImagePath = "";
    const AUTOSAVE_DELAY_MS = 1500;

    function syncState() {
        if (!fabricManager) return;
        bBoxes = fabricManager.getBBoxes();
        polygons = fabricManager.getPolygons();
        polylines = fabricManager.getPolylines();
        keypoints = fabricManager.getKeypoints();
        mode = fabricManager.getMode();
        isPolygonDrawing = fabricManager.isPolygonDrawing();
        isPolylineDrawing = mode === "polyline" && polylines.length > 0;
        selectedObject = fabricManager.getSelectedObject();
        selectedObjects = fabricManager.getSelectedObjects();
        scheduleAutosave();

        // Auto-track image status
        const totalAnns =
            bBoxes.length +
            polygons.length +
            polylines.length +
            keypoints.length;
        if (currentImagePath && totalAnns > 0) {
            imageStatusStore.setStatus(currentImagePath, "in_progress");
        }
    }

    function scheduleAutosave() {
        if (!currentImagePath) return;
        if (autosaveTimer) clearTimeout(autosaveTimer);
        autosaveTimer = setTimeout(() => performAutosave(), AUTOSAVE_DELAY_MS);
    }

    async function performAutosave() {
        if (!currentImagePath || !fabricManager) return;
        const annotations = [
            ...bBoxes.map((b) => ({
                label: b.label || "",
                shape_type: "rectangle",
                points: [
                    [b.x1, b.y1],
                    [b.x2, b.y2],
                ],
            })),
            ...polygons.map((p) => ({
                label: p.label || "",
                shape_type: "polygon",
                points: p.points.map((pt) => [pt.x, pt.y]),
            })),
            ...polylines.map((l) => ({
                label: l.label || "",
                shape_type: "linestrip",
                points: l.points.map((pt) => [pt.x, pt.y]),
            })),
            ...keypoints.map((k) => ({
                label: k.label || "",
                shape_type: "point",
                points: [[k.x, k.y]],
            })),
        ];
        try {
            await invoke("save_annotation", {
                imagePath: currentImagePath,
                annotations,
            });
            console.log(
                "[Autosave] Saved",
                annotations.length,
                "annotations for",
                currentImagePath,
            );
        } catch (err) {
            console.warn("[Autosave] Failed:", err);
        }
    }

    // Image Preloader
    function preloadAdjacentImages() {
        if (imageList.length === 0 || currentImageIndex < 0) return;
        const indicesToPreload = [currentImageIndex - 1, currentImageIndex + 1];
        for (const idx of indicesToPreload) {
            if (idx >= 0 && idx < imageList.length) {
                const img = new Image();
                img.src = convertFileSrc(imageList[idx].path);
            }
        }
    }

    onMount(() => {
        fabricManager = new FabricManager(canvasElement);

        const unsubscribeUpdate = fabricManager.on("update", syncState);
        const unsubscribeMode = fabricManager.on("modeChange", syncState);

        const resizeObserver = new ResizeObserver((entries) => {
            for (const entry of entries) {
                const { width, height } = entry.contentRect;
                fabricManager?.resize(width, height);
            }
        });

        if (parentElement) {
            resizeObserver.observe(parentElement);
        }

        return () => {
            unsubscribeUpdate();
            unsubscribeMode();
            resizeObserver.disconnect();
            fabricManager?.dispose();
        };
    });

    function setModeFromEvent(event: CustomEvent<Mode>) {
        fabricManager?.setMode(event.detail);
    }

    function setMode(newMode: Mode) {
        fabricManager?.setMode(newMode);
    }

    /** Derive the LabelMe JSON path from an image path */
    function getJsonPath(imagePath: string): string {
        const lastDot = imagePath.lastIndexOf(".");
        return (
            (lastDot > 0 ? imagePath.substring(0, lastDot) : imagePath) +
            ".json"
        );
    }

    /** Load an image onto the canvas and auto-load its LabelMe JSON if present */
    async function loadImageWithAnnotations(imagePath: string) {
        if (!fabricManager) return;

        const assetUrl = convertFileSrc(imagePath);
        console.log("[FabricAnnotator] Loading image:", imagePath);
        console.log("[FabricAnnotator] Asset URL:", assetUrl);
        try {
            await fabricManager.loadImage(assetUrl);
            console.log("[FabricAnnotator] Image loaded successfully");
        } catch (err) {
            console.error("[FabricAnnotator] Failed to load image:", err);
            return;
        }
        imageLoaded = true;
        currentImagePath = imagePath;

        // Auto-load LabelMe JSON
        const jsonPath = getJsonPath(imagePath);
        console.log("[FabricAnnotator] Checking JSON at:", jsonPath);
        try {
            const jsonText = (await invoke("read_file_content", {
                filePath: jsonPath,
            })) as string;
            const labelmeData: LabelMeJSON = JSON.parse(jsonText);
            console.log(
                "[FabricAnnotator] Parsed LabelMe JSON, shapes:",
                labelmeData.shapes?.length,
                "types:",
                labelmeData.shapes?.map((s: any) => s.shape_type),
            );
            if (labelmeData.shapes && Array.isArray(labelmeData.shapes)) {
                fabricManager.loadAnnotations(labelmeData.shapes);
            }
        } catch (err) {
            console.log(
                "[FabricAnnotator] No LabelMe JSON found or failed to load.",
                err,
            );
        }
    }

    async function triggerFileInput() {
        try {
            const selected = await open({
                multiple: false,
                filters: [
                    {
                        name: "Images",
                        extensions: ["png", "jpg", "jpeg", "webp", "gif"],
                    },
                ],
            });

            if (selected) {
                // Clear directory mode when loading single image
                imageList = [];
                currentImageIndex = -1;
                directoryPath = "";

                await loadImageWithAnnotations(selected as string);
            }
        } catch (err) {
            console.error("Failed to open file:", err);
        }
    }

    async function loadDirectory(path: string) {
        try {
            directoryPath = path;
            const resultStr = (await invoke("auto_annotate_images", {
                path: directoryPath,
                page: 1,
                pageSize: 10000,
            })) as string;

            const data = JSON.parse(resultStr);
            if (data && Array.isArray(data.annotated_images)) {
                const entries: ImageEntry[] = [];
                for (const img of data.annotated_images) {
                    entries.push({
                        name: img.path.split(/[/\\]/).pop() || img.path,
                        path: img.path,
                        hasJson: img.has_json,
                    });
                }
                imageList = entries;

                // Auto-load first image
                if (imageList.length > 0) {
                    currentImageIndex = 0;
                    await loadImageWithAnnotations(imageList[0].path);
                }
            }
        } catch (err) {
            console.error("Failed to load directory:", err);
        }
    }

    async function triggerDirectoryInput() {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
            });

            if (selected) {
                await loadDirectory(selected as string);
            }
        } catch (err) {
            console.error("Failed to open directory:", err);
        }
    }

    async function triggerMockLoad() {
        const mockPath =
            "/Users/admin/Desktop/003-open-source/dataset-app/mock_data/labelme";
        console.log(
            "[FabricAnnotator] Triggering mock load for path:",
            mockPath,
        );
        await loadDirectory(mockPath);
    }

    async function selectImage(event: CustomEvent<number>) {
        const index = event.detail;
        if (index >= 0 && index < imageList.length) {
            currentImageIndex = index;
            await loadImageWithAnnotations(imageList[index].path);
            preloadAdjacentImages();
        }
    }

    function finishPolygon() {
        fabricManager?.finishPolygon();
    }

    function resetPolygon() {
        fabricManager?.resetPolygonDrawing();
    }

    function onUpdateBBoxLabel(id: number, label: string) {
        fabricManager?.updateBBoxLabel(id, label);
    }

    function onUpdatePolygonLabel(id: number, label: string) {
        fabricManager?.updatePolygonLabel(id, label);
    }

    function onDeleteBBox(id: number) {
        fabricManager?.deleteBBox(id);
    }

    function onDeletePolygon(id: number) {
        fabricManager?.deletePolygon(id);
    }

    function onUpdatePolylineLabel(id: number, label: string) {
        fabricManager?.updatePolylineLabel(id, label);
    }

    function onUpdateKeypointLabel(id: number, label: string) {
        fabricManager?.updateKeypointLabel(id, label);
    }

    function onDeletePolyline(id: number) {
        fabricManager?.deletePolyline(id);
    }

    function onDeleteKeypoint(id: number) {
        fabricManager?.deleteKeypoint(id);
    }

    function finishPolyline() {
        fabricManager?.finishPolyline();
    }

    function resetPolyline() {
        fabricManager?.resetPolylineDrawing();
    }

    function onUpdateProperty(
        id: number,
        type: string,
        prop: string,
        value: any,
    ) {
        fabricManager?.updateProperty(id, type, prop, value);
    }

    function onToggleLock(id: number, type: string) {
        fabricManager?.toggleLock(id, type);
    }

    function onToggleVisibility(id: number, type: string) {
        fabricManager?.toggleVisibility(id, type);
    }

    function batchUpdateLabel(label: string) {
        fabricManager?.batchUpdateLabel(label);
    }

    function handleKeyDown(event: KeyboardEvent) {
        // Ignore if user is typing in an input field (like the label editor)
        if (
            event.target instanceof HTMLInputElement ||
            event.target instanceof HTMLTextAreaElement
        ) {
            return;
        }

        // Undo / Redo
        if ((event.ctrlKey || event.metaKey) && event.key === "z") {
            event.preventDefault();
            if (event.shiftKey) {
                fabricManager?.redo();
            } else {
                fabricManager?.undo();
            }
            return;
        }

        // Quick labeling with number keys (1-9)
        const num = parseInt(event.key);
        if (num >= 1 && num <= 9) {
            const className = $shortcutMap.get(num);
            if (className && fabricManager) {
                // Apply to the last drawn annotation
                const bboxes = fabricManager.getBBoxes();
                const polygons = fabricManager.getPolygons();
                if (bboxes.length > 0) {
                    const last = bboxes[bboxes.length - 1];
                    if (!last.label) {
                        fabricManager.updateBBoxLabel(last.id, className);
                        return;
                    }
                }
                if (polygons.length > 0) {
                    const last = polygons[polygons.length - 1];
                    if (!last.label) {
                        fabricManager.updatePolygonLabel(last.id, className);
                        return;
                    }
                }
            }
        }

        // Tool switching
        switch (event.key.toLowerCase()) {
            case "v":
                setMode("select");
                break;
            case "r":
                setMode("bbox");
                break;
            case "p":
                setMode("polygon");
                break;
            case "l":
                setMode("polyline");
                break;
            case "k":
                setMode("keypoint");
                break;
            case "h":
                setMode("pan");
                break;
        }
    }
</script>

<svelte:window on:keydown={handleKeyDown} />

<div
    class="flex flex-col h-full w-full overflow-hidden bg-base-100 text-base-content"
>
    <!-- Top Toolbar -->
    <FabricToolbar
        {mode}
        {isPolygonDrawing}
        {isPolylineDrawing}
        on:setMode={setModeFromEvent}
        on:triggerFileInput={triggerFileInput}
        on:triggerDirectoryInput={triggerDirectoryInput}
        on:triggerMockLoad={triggerMockLoad}
        on:finishPolygon={finishPolygon}
        on:resetPolygon={resetPolygon}
        on:finishPolyline={finishPolyline}
        on:resetPolyline={resetPolyline}
    />

    <!-- Stats Dashboard Overlay -->
    {#if showStats}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
            class="absolute inset-0 z-50 bg-base-300/60 backdrop-blur-sm flex items-center justify-center p-8"
            on:click|self={() => (showStats = false)}
        >
            <div class="max-w-md w-full relative">
                <button
                    class="absolute -top-2 -right-2 btn btn-circle btn-xs btn-error z-10"
                    on:click={() => (showStats = false)}>✕</button
                >
                <DatasetStats
                    images={imageList}
                    allBBoxes={bBoxes}
                    allPolygons={polygons}
                    allPolylines={polylines}
                    allKeypoints={keypoints}
                />
            </div>
        </div>
    {/if}

    <button
        class="fixed bottom-4 right-80 z-40 btn btn-circle btn-primary shadow-lg"
        class:right-4={!showSidebar}
        on:click={() => (showStats = !showStats)}
        title="View Dataset Stats"
    >
        <span class="material-symbols-rounded">bar_chart</span>
    </button>

    <!-- Main Area -->
    <div class="flex flex-1 overflow-hidden relative">
        <!-- Left: Image Gallery (directory mode only) -->
        {#if directoryMode && showGallery}
            <div transition:fly={{ x: -200, duration: 200 }} class="z-10">
                <ImageGallery
                    images={imageList}
                    selectedIndex={currentImageIndex}
                    on:select={selectImage}
                />
            </div>
        {/if}

        <!-- Toggle Gallery Button -->
        {#if directoryMode}
            <button
                class="absolute left-0 top-1/2 -translate-y-1/2 z-20 btn btn-circle btn-xs btn-ghost hover:bg-base-300"
                style="left: {showGallery ? '224px' : '0'}"
                on:click={() => (showGallery = !showGallery)}
            >
                <span class="material-symbols-rounded text-sm">
                    {showGallery ? "chevron_left" : "chevron_right"}
                </span>
            </button>
        {/if}

        <!-- Canvas Area -->
        <div
            bind:this={parentElement}
            class="flex-1 bg-base-200/30 relative flex items-center justify-center overflow-hidden"
        >
            {#if !imageLoaded}
                <div
                    class="text-center text-base-content/40 pointer-events-none select-none space-y-2"
                >
                    <span class="material-symbols-rounded text-[48px]"
                        >image</span
                    >
                    <p class="text-sm">
                        Load an image or directory to start annotating
                    </p>
                </div>
            {/if}
            <canvas bind:this={canvasElement} class="block"></canvas>
        </div>

        <!-- Toggle Sidebar Button -->
        <button
            class="absolute right-0 top-1/2 -translate-y-1/2 z-20 btn btn-circle btn-xs btn-ghost hover:bg-base-300"
            style="right: {showSidebar ? '288px' : '0'}"
            on:click={() => (showSidebar = !showSidebar)}
        >
            <span class="material-symbols-rounded text-sm">
                {showSidebar ? "chevron_right" : "chevron_left"}
            </span>
        </button>

        <!-- Right Sidebar -->
        {#if showSidebar}
            <div
                transition:fly={{ x: 200, duration: 200 }}
                class="flex w-72 flex-col border-l border-base-200 bg-base-100 shrink-0 z-10"
            >
                <AnnotationSidebar
                    {bBoxes}
                    {polygons}
                    {polylines}
                    {keypoints}
                    {onUpdateBBoxLabel}
                    {onUpdatePolygonLabel}
                    {onUpdatePolylineLabel}
                    {onUpdateKeypointLabel}
                    {onDeleteBBox}
                    {onDeletePolygon}
                    {onDeletePolyline}
                    {onDeleteKeypoint}
                    {onToggleLock}
                    {onToggleVisibility}
                    {selectedObjects}
                />

                <PropertyInspector
                    {selectedObject}
                    {selectedObjects}
                    onBatchUpdateLabel={batchUpdateLabel}
                    {onUpdateProperty}
                />
            </div>
        {/if}
    </div>
</div>

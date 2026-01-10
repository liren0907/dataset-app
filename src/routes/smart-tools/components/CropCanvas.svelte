<script lang="ts">
    import { onMount, createEventDispatcher } from "svelte";

    export let originalImage: HTMLImageElement | null;
    export let zoom: number;
    export let rotation: number;
    export let aspectRatio: string;
    export let cropArea: {
        x: number;
        y: number;
        width: number;
        height: number;
    };

    // We export the canvas element binding so the parent (and PreviewModal) can access it if needed for calculations
    export let canvasElement: HTMLCanvasElement;

    const dispatch = createEventDispatcher();

    let ctx: CanvasRenderingContext2D | null = null;
    let container: HTMLElement;

    // Interaction state
    let isDragging = false;
    let isResizing = false;
    let dragStart = { x: 0, y: 0 };
    let resizeHandle = "";

    // Hover states
    let isHoveringCrop = false;
    let isHoveringHandle = false;
    let currentHandle = "";
    let isPanning = false;
    let lastMousePos = { x: 0, y: 0 };

    // Watch for prop changes to redraw
    $: if (originalImage && canvasElement) {
        // Debounce slightly or just draw
        requestAnimationFrame(drawImage);
    }
    $: if (zoom || rotation) {
        requestAnimationFrame(drawImage);
    }

    onMount(() => {
        if (originalImage) {
            initCanvas();
            drawImage();
        }
    });

    // We need to re-init canvas if the container resizes or image changes significantly
    export function initCanvas() {
        if (!canvasElement || !originalImage || !container) return;

        const containerRect = container.getBoundingClientRect();
        canvasElement.width = containerRect.width;
        canvasElement.height = containerRect.height;

        ctx = canvasElement.getContext("2d");

        // Initialize crop area to center of image if not set (or we can trust parent to set initial crop)
        // For refactoring, let's assume parent sets initial logic or we check if 0
        if (cropArea.width === 0) {
            const centerX = canvasElement.width / 2;
            const centerY = canvasElement.height / 2;
            cropArea.x = centerX - 100;
            cropArea.y = centerY - 100;
            cropArea.width = 200;
            cropArea.height = 200;
        }
    }

    export function drawImage() {
        if (!ctx || !originalImage || !canvasElement) return;

        ctx.clearRect(0, 0, canvasElement.width, canvasElement.height);

        // Save context for rotation
        ctx.save();

        // Apply zoom and rotation
        ctx.translate(canvasElement.width / 2, canvasElement.height / 2);
        ctx.scale(zoom, zoom);
        ctx.rotate((rotation * Math.PI) / 180);
        ctx.translate(-canvasElement.width / 2, -canvasElement.height / 2);

        // Draw the image (fitted to canvas)
        const canvasAspect = canvasElement.width / canvasElement.height;
        const imageAspect = originalImage.width / originalImage.height;

        let scale, scaledWidth, scaledHeight, x, y;

        if (canvasAspect > imageAspect) {
            // Canvas is wider, fit by height
            scale = canvasElement.height / originalImage.height;
            scaledWidth = originalImage.width * scale;
            scaledHeight = canvasElement.height;
            x = (canvasElement.width - scaledWidth) / 2;
            y = 0;
        } else {
            // Canvas is taller, fit by width
            scale = canvasElement.width / originalImage.width;
            scaledWidth = canvasElement.width;
            scaledHeight = originalImage.height * scale;
            x = 0;
            y = (canvasElement.height - scaledHeight) / 2;
        }

        ctx.drawImage(originalImage, x, y, scaledWidth, scaledHeight);

        // Restore context
        ctx.restore();

        // Draw crop area
        drawCropArea();
    }

    function drawCropArea() {
        if (!ctx || !canvasElement) return;

        const isActive = isHoveringCrop || isDragging;
        const isResizingNow = isResizing || isHoveringHandle;

        // Styling
        if (isResizingNow) {
            ctx.strokeStyle = "#ef4444";
            ctx.lineWidth = 3;
        } else if (isActive) {
            ctx.strokeStyle = "#10b981";
            ctx.lineWidth = 2;
        } else {
            ctx.strokeStyle = "#3b82f6";
            ctx.lineWidth = 2;
        }

        // Draw React
        ctx.setLineDash(isActive ? [] : [5, 5]);
        ctx.strokeRect(cropArea.x, cropArea.y, cropArea.width, cropArea.height);
        ctx.setLineDash([]);

        // Glow
        if (isActive) {
            ctx.shadowColor = ctx.strokeStyle as string;
            ctx.shadowBlur = 8;
            ctx.strokeRect(
                cropArea.x,
                cropArea.y,
                cropArea.width,
                cropArea.height,
            );
            ctx.shadowBlur = 0;
        }

        // Handles
        const handleSize = isHoveringHandle ? 12 : 8;
        const handles = [
            { x: cropArea.x, y: cropArea.y, type: "nw" },
            { x: cropArea.x + cropArea.width, y: cropArea.y, type: "ne" },
            { x: cropArea.x, y: cropArea.y + cropArea.height, type: "sw" },
            {
                x: cropArea.x + cropArea.width,
                y: cropArea.y + cropArea.height,
                type: "se",
            },
        ];

        handles.forEach((handle) => {
            const isHovered = currentHandle === handle.type;
            if (isHovered) {
                ctx!.fillStyle = "#ef4444";
                ctx!.strokeStyle = "#ffffff";
                ctx!.lineWidth = 2;
            } else if (isHoveringHandle) {
                ctx!.fillStyle = "#10b981";
                ctx!.strokeStyle = "#ffffff";
                ctx!.lineWidth = 1;
            } else {
                ctx!.fillStyle = "#3b82f6";
                ctx!.strokeStyle = "#ffffff";
                ctx!.lineWidth = 1;
            }

            ctx!.fillRect(
                handle.x - handleSize / 2,
                handle.y - handleSize / 2,
                handleSize,
                handleSize,
            );
            ctx!.strokeRect(
                handle.x - handleSize / 2,
                handle.y - handleSize / 2,
                handleSize,
                handleSize,
            );
        });

        // Dimensions Info
        if (isActive) {
            ctx.fillStyle = "#ffffff";
            ctx.strokeStyle = "#000000";
            ctx.lineWidth = 3;
            ctx.font = "12px Arial";
            const dimensionsText = `${Math.round(cropArea.width)} × ${Math.round(cropArea.height)}`;

            const textX = cropArea.x + cropArea.width - 10;
            const textY = cropArea.y + 20;

            const metrics = ctx.measureText(dimensionsText);
            ctx.fillStyle = "rgba(0, 0, 0, 0.7)";
            ctx.fillRect(
                textX - metrics.width - 8,
                textY - 12,
                metrics.width + 16,
                20,
            );

            ctx.fillStyle = "#ffffff";
            ctx.strokeStyle = "#000000";
            // strokeText needs to be called on ctx, we know it's not null here due to guard
            ctx.lineWidth = 2;
            ctx.strokeText(dimensionsText, textX - metrics.width - 4, textY);
            ctx.fillText(dimensionsText, textX - metrics.width - 4, textY);
        }
    }

    function getMousePos(event: MouseEvent) {
        const rect = canvasElement.getBoundingClientRect();
        return {
            x: event.clientX - rect.left,
            y: event.clientY - rect.top,
        };
    }

    function isInCropArea(x: number, y: number) {
        return (
            x >= cropArea.x &&
            x <= cropArea.x + cropArea.width &&
            y >= cropArea.y &&
            y <= cropArea.y + cropArea.height
        );
    }

    function getResizeHandle(x: number, y: number) {
        const handleSize = 12;
        const handles = [
            { pos: "nw", x: cropArea.x, y: cropArea.y },
            { pos: "ne", x: cropArea.x + cropArea.width, y: cropArea.y },
            { pos: "sw", x: cropArea.x, y: cropArea.y + cropArea.height },
            {
                pos: "se",
                x: cropArea.x + cropArea.width,
                y: cropArea.y + cropArea.height,
            },
        ];

        for (const handle of handles) {
            if (
                Math.abs(x - handle.x) < handleSize &&
                Math.abs(y - handle.y) < handleSize
            ) {
                return handle.pos;
            }
        }
        return "";
    }

    // --- Interaction Handlers ---

    function handleMouseDown(event: MouseEvent) {
        if (!canvasElement) return;
        const pos = getMousePos(event);
        resizeHandle = getResizeHandle(pos.x, pos.y);

        if (isPanning) {
            // Already in panning mode via Spacebar
        } else if (resizeHandle) {
            isResizing = true;
        } else if (isInCropArea(pos.x, pos.y)) {
            isDragging = true;
        } else {
            // New crop area
            cropArea.x = pos.x - 50;
            cropArea.y = pos.y - 50;
            cropArea.width = 100;
            cropArea.height = 100;
            isDragging = true;
            drawImage();
        }
        dragStart = pos;
    }

    function handleMouseMove(event: MouseEvent) {
        if (!canvasElement) return;
        const pos = getMousePos(event);
        lastMousePos = { x: event.clientX, y: event.clientY };

        updateHoverStates();

        const dx = pos.x - dragStart.x;
        const dy = pos.y - dragStart.y;

        if (isResizing) {
            resizeCropArea(dx, dy);
        } else if (isDragging) {
            moveCropArea(dx, dy);
        } else if (isPanning) {
            // panning local var, handled by moving crop area just like original
            const panSpeed = 0.5;
            cropArea.x += dx * panSpeed;
            cropArea.y += dy * panSpeed;
            drawImage();
        }

        dragStart = pos;
    }

    function handleMouseUp() {
        isDragging = false;
        isResizing = false;
        resizeHandle = "";
        isPanning = false;
    }

    function handleMouseLeave() {
        handleMouseUp();
        isHoveringCrop = false;
        isHoveringHandle = false;
        currentHandle = "";
    }

    function handleMouseEnter() {
        updateHoverStates();
    }

    function updateHoverStates() {
        if (!canvasElement) return;

        const rect = canvasElement.getBoundingClientRect();
        const mouseX = lastMousePos.x - rect.left;
        const mouseY = lastMousePos.y - rect.top;

        currentHandle = getResizeHandle(mouseX, mouseY);
        isHoveringHandle = currentHandle !== "";
        isHoveringCrop = !isHoveringHandle && isInCropArea(mouseX, mouseY);
    }

    function getCanvasCursor() {
        if (isDragging || isPanning) {
            return isResizing ? "grabbing" : "grabbing";
        }
        if (isHoveringHandle) {
            const cursorMap: any = {
                nw: "nw-resize",
                ne: "ne-resize",
                sw: "sw-resize",
                se: "se-resize",
            };
            return cursorMap[currentHandle] || "pointer";
        }
        if (isHoveringCrop) return "grab";
        return "crosshair";
    }

    function resizeCropArea(dx: number, dy: number) {
        switch (resizeHandle) {
            case "nw":
                cropArea.x += dx;
                cropArea.y += dy;
                cropArea.width -= dx;
                cropArea.height -= dy;
                break;
            case "ne":
                cropArea.y += dy;
                cropArea.width += dx;
                cropArea.height -= dy;
                break;
            case "sw":
                cropArea.x += dx;
                cropArea.width -= dx;
                cropArea.height += dy;
                break;
            case "se":
                cropArea.width += dx;
                cropArea.height += dy;
                break;
        }

        if (cropArea.width < 20) cropArea.width = 20;
        if (cropArea.height < 20) cropArea.height = 20;

        if (aspectRatio !== "free") {
            applyAspectRatio();
        }
        drawImage();
    }

    function moveCropArea(dx: number, dy: number) {
        cropArea.x += dx;
        cropArea.y += dy;

        // Boundary check
        if (cropArea.x < 0) cropArea.x = 0;
        if (cropArea.y < 0) cropArea.y = 0;
        if (cropArea.x + cropArea.width > canvasElement.width)
            cropArea.x = canvasElement.width - cropArea.width;
        if (cropArea.y + cropArea.height > canvasElement.height)
            cropArea.y = canvasElement.height - cropArea.height;

        drawImage();
    }

    function applyAspectRatio() {
        const ratios: any = {
            square: 1,
            "4:3": 4 / 3,
            "16:9": 16 / 9,
            "3:2": 3 / 2,
            "2:3": 2 / 3,
            "9:16": 9 / 16,
        };
        const ratio = ratios[aspectRatio];
        if (ratio) {
            cropArea.height = cropArea.width / ratio;
        }
    }

    // Handlers exposed/called from parent via props change or methods?
    // Since we put interaction on canvas, we just need to handle keyboard events passed down or handle them globally in parent?
    // Parent handles global keys (shortcuts).

    // Make drawing/panning accessible for key events which modify state
    export function setPanning(panning: boolean) {
        isPanning = panning;
        if (panning && canvasElement) canvasElement.style.cursor = "grab";
        else updateHoverStates(); // reset cursor
    }
</script>

<div
    bind:this={container}
    class="relative bg-gradient-to-br from-gray-50 to-gray-100 rounded-xl overflow-hidden shadow-inner border-2 border-gray-200"
    style="height: 600px;"
>
    <!-- Grid Background -->
    <div class="absolute inset-0 opacity-10">
        <svg width="100%" height="100%" xmlns="http://www.w3.org/2000/svg">
            <defs>
                <pattern
                    id="grid"
                    width="20"
                    height="20"
                    patternUnits="userSpaceOnUse"
                >
                    <path
                        d="M 20 0 L 0 0 0 20"
                        fill="none"
                        stroke="#6b7280"
                        stroke-width="1"
                    />
                </pattern>
            </defs>
            <rect width="100%" height="100%" fill="url(#grid)" />
        </svg>
    </div>

    <!-- Main Canvas -->
    <canvas
        bind:this={canvasElement}
        on:mousedown={handleMouseDown}
        on:mousemove={handleMouseMove}
        on:mouseup={handleMouseUp}
        on:mouseleave={handleMouseLeave}
        on:mouseenter={handleMouseEnter}
        class="relative z-10 w-full h-full transition-all duration-200"
        style="cursor: {getCanvasCursor()};"
    ></canvas>

    <!-- Info Overlay -->
    <div
        class="absolute top-4 left-4 bg-black bg-opacity-50 text-white px-3 py-1 rounded-lg text-sm z-20"
    >
        Zoom: {Math.round(zoom * 100)}% | {rotation}°
    </div>
</div>

<div class="mt-3 text-xs text-gray-500 text-center">
    <span class="inline-flex items-center">
        <svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
            <path
                fill-rule="evenodd"
                d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                clip-rule="evenodd"
            />
        </svg>
        Drag handles to resize • Drag area to move • Click empty space for new crop
    </span>
</div>

<style>
    canvas {
        cursor: move;
    }
    canvas:hover {
        cursor: move;
    }
</style>

<script lang="ts">
    let {
        isOpen,
        originalImage,
        cropArea,
        rotation,
        zoom,
        canvas,
        onclose,
    }: {
        isOpen: boolean;
        originalImage: HTMLImageElement | null;
        cropArea: { x: number; y: number; width: number; height: number };
        rotation: number;
        zoom: number;
        canvas: HTMLCanvasElement | undefined;
        onclose?: () => void;
    } = $props();

    let previewCanvas: HTMLCanvasElement;
    let previewCtx: CanvasRenderingContext2D | null = null;

    $effect(() => {
        if (isOpen && previewCanvas && originalImage && canvas) {
            renderCropPreview();
        }
    });

    function renderCropPreview() {
        if (!originalImage || !canvas || !previewCanvas) return;

        try {
            // Calculate how the original image is scaled and positioned on the canvas
            // This mirrors the logic in drawImage()
            const canvasAspect = canvas.width / canvas.height;
            const imageAspect = originalImage.width / originalImage.height;

            let scale, scaledWidth, scaledHeight, imageX, imageY;

            if (canvasAspect > imageAspect) {
                // Canvas is wider, fit by height
                scale = canvas.height / originalImage.height;
                scaledWidth = originalImage.width * scale;
                scaledHeight = canvas.height;
                imageX = (canvas.width - scaledWidth) / 2;
                imageY = 0;
            } else {
                // Canvas is taller, fit by width
                scale = canvas.width / originalImage.width;
                scaledWidth = canvas.width;
                scaledHeight = originalImage.height * scale;
                imageX = 0;
                imageY = (canvas.height - scaledHeight) / 2;
            }

            // Apply zoom to scale
            scale *= zoom;

            // Transform crop area from canvas coordinates to original image coordinates
            const cropX = Math.max(0, (cropArea.x - imageX) / scale);
            const cropY = Math.max(0, (cropArea.y - imageY) / scale);
            let cropWidth = cropArea.width / scale;
            let cropHeight = cropArea.height / scale;

            // Ensure crop doesn't exceed image boundaries
            const maxCropX = originalImage.width - cropWidth;
            const maxCropY = originalImage.height - cropHeight;

            const finalCropX = Math.min(cropX, maxCropX);
            const finalCropY = Math.min(cropY, maxCropY);
            const finalCropWidth = Math.min(
                cropWidth,
                originalImage.width - finalCropX,
            );
            const finalCropHeight = Math.min(
                cropHeight,
                originalImage.height - finalCropY,
            );

            // Set preview canvas size (maintain crop aspect ratio)
            const maxPreviewSize = 300;
            const cropAspect = finalCropWidth / finalCropHeight;

            let previewWidth, previewHeight;
            if (cropAspect > 1) {
                previewWidth = maxPreviewSize;
                previewHeight = maxPreviewSize / cropAspect;
            } else {
                previewHeight = maxPreviewSize;
                previewWidth = maxPreviewSize * cropAspect;
            }

            // Ensure minimum size
            previewWidth = Math.max(previewWidth, 50);
            previewHeight = Math.max(previewHeight, 50);

            previewCanvas.width = previewWidth;
            previewCanvas.height = previewHeight;

            // Get canvas context
            previewCtx = previewCanvas.getContext("2d");
            if (!previewCtx) {
                console.error("Could not get preview canvas context");
                return;
            }

            // Clear canvas
            previewCtx.clearRect(
                0,
                0,
                previewCanvas.width,
                previewCanvas.height,
            );

            // Save context for transformations
            previewCtx.save();

            // Apply rotation if needed
            if (rotation !== 0) {
                previewCtx.translate(
                    previewCanvas.width / 2,
                    previewCanvas.height / 2,
                );
                previewCtx.rotate((rotation * Math.PI) / 180);
                previewCtx.translate(
                    -previewCanvas.width / 2,
                    -previewCanvas.height / 2,
                );
            }

            // Draw the cropped portion of the original image
            previewCtx.drawImage(
                originalImage,
                finalCropX,
                finalCropY,
                finalCropWidth,
                finalCropHeight, // Source rectangle on original image
                0,
                0,
                previewCanvas.width,
                previewCanvas.height, // Destination rectangle on preview canvas
            );

            previewCtx.restore();
        } catch (error) {
            console.error("Error in renderCropPreview:", error);
        }
    }

    function downloadCroppedImage() {
        if (!previewCanvas) return;
        const link = document.createElement("a");
        link.download = "cropped-image.png";
        link.href = previewCanvas.toDataURL();
        link.click();
    }

    function close() {
        onclose?.();
    }
</script>

<dialog class="modal" class:modal-open={isOpen}>
    <div class="modal-box max-w-3xl">
        <!-- Header -->
        <div class="flex items-center justify-between mb-4">
            <h3 class="font-bold text-lg flex items-center gap-2">
                <span class="material-symbols-rounded text-primary"
                    >visibility</span
                >
                Crop Preview
            </h3>
            <button onclick={close} class="btn btn-sm btn-circle btn-ghost">
                <span class="material-symbols-rounded">close</span>
            </button>
        </div>

        <!-- Preview Content -->
        <div class="flex justify-center bg-base-200 rounded-lg p-8">
            <div class="relative">
                <canvas
                    bind:this={previewCanvas}
                    class="border-4 border-base-100 shadow-lg rounded-lg"
                ></canvas>
                <div class="badge badge-success absolute -top-2 -right-2 gap-1">
                    <span class="material-symbols-rounded icon-sm">check</span>
                    Cropped
                </div>
            </div>
        </div>

        <!-- Footer -->
        <div class="modal-action">
            <div class="flex-1 text-sm opacity-60 flex items-center gap-1">
                <span class="material-symbols-rounded icon-sm">info</span>
                High quality preview
            </div>
            <button onclick={close} class="btn btn-ghost">Cancel</button>
            <button onclick={downloadCroppedImage} class="btn btn-success">
                <span class="material-symbols-rounded">download</span>
                Download
            </button>
        </div>
    </div>
    <form method="dialog" class="modal-backdrop">
        <button onclick={close}>close</button>
    </form>
</dialog>

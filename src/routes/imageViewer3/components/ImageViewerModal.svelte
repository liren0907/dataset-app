<script lang="ts">
    import { createEventDispatcher, onMount } from "svelte";

    export let image: any; // ProcessedImage with annotations
    export let annotationType: string;

    const dispatch = createEventDispatcher();

    function formatFileSize(bytes: number) {
        if (!bytes) return "";
        if (bytes < 1024) return bytes + " B";
        if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
        return (bytes / 1048576).toFixed(1) + " MB";
    }

    // Canvas drawing logic
    function drawAnnotations() {
        if (!image || !image.annotations) return;

        const imageElement = document.getElementById(
            "selected-image",
        ) as HTMLImageElement;
        const canvasElement = document.getElementById(
            "annotation-canvas",
        ) as HTMLCanvasElement;

        if (!imageElement || !canvasElement) return;

        // Wait for image to be loaded
        if (!imageElement.complete) {
            imageElement.onload = () => drawAnnotations();
            return;
        }

        const ctx = canvasElement.getContext("2d");
        if (!ctx) return;

        // Set canvas size to match the actual displayed image size
        canvasElement.width = imageElement.clientWidth;
        canvasElement.height = imageElement.clientHeight;

        // Clear previous drawings
        ctx.clearRect(0, 0, canvasElement.width, canvasElement.height);

        // Calculate scale factors for the image
        const naturalWidth = imageElement.naturalWidth;
        const naturalHeight = imageElement.naturalHeight;
        const scaleX = canvasElement.width / naturalWidth;
        const scaleY = canvasElement.height / naturalHeight;

        // Draw each annotation
        image.annotations.forEach((annotation: any, index: number) => {
            if (!annotation.points) return;

            // Assign a color based on the index
            const hue = (index * 30) % 360;
            ctx.strokeStyle = `hsl(${hue}, 100%, 50%)`;
            ctx.fillStyle = `hsla(${hue}, 100%, 50%, 0.2)`;
            ctx.lineWidth = 3;

            // Start drawing
            ctx.beginPath();

            // Draw based on annotation type
            if (annotation.shape_type === "rectangle") {
                // For rectangles, we expect 2 points (top-left and bottom-right)
                if (annotation.points.length >= 2) {
                    const topLeft = annotation.points[0];
                    const bottomRight = annotation.points[1];

                    const x = topLeft[0] * scaleX;
                    const y = topLeft[1] * scaleY;
                    const width = (bottomRight[0] - topLeft[0]) * scaleX;
                    const height = (bottomRight[1] - topLeft[1]) * scaleY;

                    ctx.rect(x, y, width, height);
                }
            } else if (annotation.shape_type === "polygon") {
                // For polygons, we need at least 3 points
                if (annotation.points.length >= 3) {
                    // Move to the first point
                    ctx.moveTo(
                        annotation.points[0][0] * scaleX,
                        annotation.points[0][1] * scaleY,
                    );

                    // Draw lines to each subsequent point
                    for (let i = 1; i < annotation.points.length; i++) {
                        ctx.lineTo(
                            annotation.points[i][0] * scaleX,
                            annotation.points[i][1] * scaleY,
                        );
                    }

                    // Close the polygon
                    ctx.closePath();
                }
            }

            // Fill and stroke the path
            ctx.fill();
            ctx.stroke();

            // Add a label
            const firstPoint = annotation.points[0];
            const labelX = firstPoint[0] * scaleX;
            const labelY = firstPoint[1] * scaleY - 5;

            ctx.font = "14px Arial";
            ctx.fillStyle = `hsl(${hue}, 100%, 35%)`;
            ctx.fillText(annotation.label, labelX, labelY);
        });
    }

    onMount(() => {
        // Wait for DOM to update
        setTimeout(() => {
            drawAnnotations();
        }, 100);

        // Add resize listener to redraw on window resize
        window.addEventListener("resize", drawAnnotations);

        return () => {
            window.removeEventListener("resize", drawAnnotations);
        };
    });
</script>

<div
    class="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50 p-4"
>
    <div
        class="relative max-w-6xl w-full bg-white rounded-lg shadow-xl overflow-hidden"
    >
        <div class="flex justify-between items-center p-4 border-b">
            <h3 class="text-lg font-medium text-gray-800 truncate">
                {image.name}
            </h3>
            <button
                on:click={() => dispatch("close")}
                class="text-gray-400 hover:text-gray-500"
            >
                ✕
            </button>
        </div>
        <div
            class="p-4 flex flex-col items-center justify-center max-h-[calc(100vh-8rem)] overflow-auto"
        >
            <div class="relative">
                <!-- Main image -->
                <img
                    id="selected-image"
                    src={image.previewUrl}
                    alt={image.name}
                    class="max-w-full max-h-[calc(100vh-12rem)] object-contain"
                />

                <!-- Annotation canvas layer -->
                {#if image.annotations && image.annotations.length > 0}
                    <canvas
                        id="annotation-canvas"
                        class="absolute top-0 left-0 w-full h-full pointer-events-none"
                    ></canvas>
                {/if}
            </div>

            <!-- Display annotations in modal if available -->
            {#if image.annotations && image.annotations.length > 0}
                <div
                    class="mt-4 p-3 bg-green-50 rounded-md border border-green-100 max-w-xl w-full"
                >
                    <h4 class="text-md font-medium text-green-800 mb-2">
                        {annotationType === "bounding_box"
                            ? "Bounding Box"
                            : "Polygon"} Annotations
                    </h4>
                    <ul class="space-y-1">
                        {#each image.annotations as annotation}
                            <li class="flex justify-between text-sm">
                                <span class="text-gray-700"
                                    >{annotation.label}</span
                                >
                                <span class="text-gray-500 font-medium"
                                    >{annotation.shape_type}</span
                                >
                            </li>
                        {/each}
                    </ul>
                </div>
            {/if}
        </div>
        <div class="p-4 border-t bg-gray-50">
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                {#if image.size}
                    <div>
                        <span class="text-sm text-gray-500">Size:</span>
                        <span class="text-sm font-medium ml-1"
                            >{formatFileSize(image.size)}</span
                        >
                    </div>
                {/if}
                {#if image.dimensions}
                    <div>
                        <span class="text-sm text-gray-500">Dimensions:</span>
                        <span class="text-sm font-medium ml-1"
                            >{image.dimensions.width} × {image.dimensions
                                .height}</span
                        >
                    </div>
                {/if}
                {#if image.created}
                    <div>
                        <span class="text-sm text-gray-500">Created:</span>
                        <span class="text-sm font-medium ml-1"
                            >{new Date(image.created).toLocaleString()}</span
                        >
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>

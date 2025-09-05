<script lang="ts">
    import { onMount, afterUpdate, createEventDispatcher } from 'svelte';

    export let selectedImage: any | null = null; // The image object to display
    export let annotationType: string = 'bounding_box'; // To display correct annotation info

    const dispatch = createEventDispatcher();

    function close() {
        dispatch('close');
    }

    // Helper function to format file size
    function formatFileSize(bytes) {
        if (bytes === null || bytes === undefined) return '';
        if (bytes < 1024) return bytes + " B";
        if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
        return (bytes / 1048576).toFixed(1) + " MB";
    }

    // Function to draw annotations on the selected image
    function drawAnnotationsOnModal() {
        if (!selectedImage || !selectedImage.annotations) return;

        const imageElement = document.getElementById("selected-image-modal") as HTMLImageElement;
        const canvasElement = document.getElementById("annotation-canvas-modal") as HTMLCanvasElement;

        if (!imageElement || !canvasElement) return;

        if (!imageElement.complete || imageElement.naturalHeight === 0) {
            const attemptDraw = () => {
                if (imageElement.complete && imageElement.naturalHeight > 0) {
                    drawAnnotationsOnModal();
                } else {
                    console.warn("Image in modal could not be loaded for annotation drawing.");
                }
            };
            imageElement.onload = attemptDraw;
            // If onload already fired and it failed, avoid getting stuck in a loop if called from afterUpdate repeatedly.
            // However, `on:load` on the img tag itself is a more direct way to handle initial draw.
            return;
        }

        const canvas = canvasElement;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        canvas.width = imageElement.clientWidth;
        canvas.height = imageElement.clientHeight;
        ctx.clearRect(0, 0, canvas.width, canvas.height);

        const naturalWidth = imageElement.naturalWidth;
        const naturalHeight = imageElement.naturalHeight;
        const scaleX = canvas.width / naturalWidth;
        const scaleY = canvas.height / naturalHeight;

        selectedImage.annotations.forEach((annotation, index) => {
            if (!annotation.points || annotation.points.length === 0) return;

            const hue = (index * 30) % 360;
            ctx.strokeStyle = `hsl(${hue}, 100%, 50%)`;
            ctx.fillStyle = `hsla(${hue}, 100%, 50%, 0.2)`;
            ctx.lineWidth = 3;

            ctx.beginPath();

            if (annotation.shape_type === "rectangle") {
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
                if (annotation.points.length >= 1) {
                    ctx.moveTo(annotation.points[0][0] * scaleX, annotation.points[0][1] * scaleY);
                    for (let i = 1; i < annotation.points.length; i++) {
                        ctx.lineTo(annotation.points[i][0] * scaleX, annotation.points[i][1] * scaleY);
                    }
                    if (annotation.points.length >= 3) {
                        ctx.closePath();
                    }
                }
            }

            const isClosedPolygon = annotation.shape_type === "polygon" && annotation.points.length >= 3;
            const isRectangle = annotation.shape_type === "rectangle" && annotation.points.length >=2;

            if (isClosedPolygon || isRectangle) {
                ctx.fill();
            }
            ctx.stroke();

            if (annotation.label && annotation.points[0]) {
                const firstPoint = annotation.points[0];
                const labelX = firstPoint[0] * scaleX;
                let labelY = firstPoint[1] * scaleY - 5;
                if (labelY < 15 && firstPoint[1] * scaleY + 15 < canvas.height) labelY = firstPoint[1] * scaleY + 15;
                 else if (labelY < 15) labelY = 15;


                ctx.font = "14px Arial";
                ctx.fillStyle = `hsl(${hue}, 100%, 35%)`;
                ctx.fillText(annotation.label, labelX, labelY);
            }
        });
    }

    afterUpdate(() => {
        if (selectedImage && selectedImage.annotations && selectedImage.annotations.length > 0) {
            // Defer to allow DOM to fully update and image to potentially load.
            // The on:load on the image is the primary trigger for drawing.
            // This afterUpdate serves as a fallback if props change after initial load.
            setTimeout(() => {
                 if (document.getElementById("selected-image-modal")?.complete) {
                    drawAnnotationsOnModal();
                }
            }, 50);
        }
    });

    onMount(() => {
         if (selectedImage && selectedImage.annotations && selectedImage.annotations.length > 0) {
             // Defer to ensure image is loaded if component mounts with an image already selected
            setTimeout(() => {
                if (document.getElementById("selected-image-modal")?.complete) {
                    drawAnnotationsOnModal();
                }
            }, 100); // A slightly longer delay onMount just in case.
        }
    });

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'Escape') {
            close();
        }
    }

</script>

<svelte:window on:keydown={handleKeydown} />

{#if selectedImage}
    <div
        class="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50 p-4"
        role="dialog"
        aria-modal="true"
        aria-labelledby="modal-title"
        on:click|self={close}
    >
        <div
            class="relative max-w-6xl w-full bg-white rounded-lg shadow-xl overflow-hidden flex flex-col max-h-[calc(100vh-2rem)]"
            on:click|stopPropagation
        >
            <div class="flex justify-between items-center p-4 border-b">
                <h3 id="modal-title" class="text-lg font-medium text-gray-800 truncate">
                    {selectedImage.name || 'Image Details'}
                </h3>
                <button
                    on:click={close}
                    class="text-gray-400 hover:text-gray-600 p-2 -mr-2 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    aria-label="Close image viewer"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>
            <div
                class="p-4 flex-grow flex flex-col items-center justify-center overflow-auto min-h-[200px]" 
            >
                <div class="relative w-full h-full flex items-center justify-center">
                    <img
                        id="selected-image-modal"
                        src={selectedImage.previewUrl}
                        alt={selectedImage.name}
                        class="max-w-full max-h-full object-contain block"
                        on:load={drawAnnotationsOnModal}
                    />
                    {#if selectedImage.annotations && selectedImage.annotations.length > 0}
                        <canvas
                            id="annotation-canvas-modal"
                            class="absolute top-0 left-0 w-full h-full pointer-events-none"
                        ></canvas>
                    {/if}
                </div>
            </div>
            <div class="p-4 border-t bg-gray-50 text-xs">
                {#if selectedImage.annotations && selectedImage.annotations.length > 0}
                    <div
                        class="mb-3 p-3 bg-green-50 rounded-md border border-green-200 max-h-32 overflow-y-auto"
                    >
                        <h4 class="text-xs font-semibold text-green-800 mb-2">
                            {annotationType === "bounding_box" ? "Bounding Box" : "Polygon"} Annotations ({selectedImage.annotations.length})
                        </h4>
                        <ul class="space-y-1">
                            {#each selectedImage.annotations as annotation, i (i)}
                                <li class="flex justify-between items-center">
                                    <span class="text-gray-700 truncate pr-2">{annotation.label || 'Unlabelled'}</span>
                                    <span class="text-gray-500 font-medium bg-gray-100 px-1.5 py-0.5 rounded text-xs">{annotation.shape_type}</span>
                                </li>
                            {/each}
                        </ul>
                    </div>
                {/if}
                <div class="grid grid-cols-1 sm:grid-cols-3 gap-x-4 gap-y-2">
                    {#if selectedImage.size != null}
                        <div>
                            <span class="text-gray-500">Size:</span>
                            <span class="text-gray-800 font-medium ml-1">{formatFileSize(selectedImage.size)}</span>
                        </div>
                    {/if}
                    {#if selectedImage.dimensions}
                        <div>
                            <span class="text-gray-500">Dimensions:</span>
                            <span class="text-gray-800 font-medium ml-1">{selectedImage.dimensions.width} × {selectedImage.dimensions.height}</span>
                        </div>
                    {/if}
                    {#if selectedImage.path}
                         <div class="sm:col-span-3">
                            <span class="text-gray-500">Path:</span>
                            <span class="text-gray-800 font-medium ml-1 break-all">{selectedImage.path}</span>
                        </div>
                    {/if}
                    {#if selectedImage.created}
                        <div class="sm:col-span-3 mt-1">
                            <span class="text-gray-500">Created:</span>
                            <span class="text-gray-800 font-medium ml-1">{new Date(selectedImage.created).toLocaleString()}</span>
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    </div>
{/if} 
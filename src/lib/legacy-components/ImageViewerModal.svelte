<script lang="ts">
    import { onMount, afterUpdate, createEventDispatcher } from "svelte";

    export let selectedImage: any | null = null;
    export let annotationType: string = "bounding_box";

    const dispatch = createEventDispatcher();

    function close() {
        dispatch("close");
    }

    function formatFileSize(bytes) {
        if (bytes === null || bytes === undefined) return "";
        if (bytes < 1024) return bytes + " B";
        if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
        return (bytes / 1048576).toFixed(1) + " MB";
    }

    function drawAnnotationsOnModal() {
        if (!selectedImage || !selectedImage.annotations) return;

        const imageElement = document.getElementById(
            "selected-image-modal",
        ) as HTMLImageElement;
        const canvasElement = document.getElementById(
            "annotation-canvas-modal",
        ) as HTMLCanvasElement;

        if (!imageElement || !canvasElement) return;

        if (!imageElement.complete || imageElement.naturalHeight === 0) {
            const attemptDraw = () => {
                if (imageElement.complete && imageElement.naturalHeight > 0) {
                    drawAnnotationsOnModal();
                }
            };
            imageElement.onload = attemptDraw;
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
                    ctx.moveTo(
                        annotation.points[0][0] * scaleX,
                        annotation.points[0][1] * scaleY,
                    );
                    for (let i = 1; i < annotation.points.length; i++) {
                        ctx.lineTo(
                            annotation.points[i][0] * scaleX,
                            annotation.points[i][1] * scaleY,
                        );
                    }
                    if (annotation.points.length >= 3) {
                        ctx.closePath();
                    }
                }
            }

            const isClosedPolygon =
                annotation.shape_type === "polygon" &&
                annotation.points.length >= 3;
            const isRectangle =
                annotation.shape_type === "rectangle" &&
                annotation.points.length >= 2;

            if (isClosedPolygon || isRectangle) {
                ctx.fill();
            }
            ctx.stroke();

            if (annotation.label && annotation.points[0]) {
                const firstPoint = annotation.points[0];
                const labelX = firstPoint[0] * scaleX;
                let labelY = firstPoint[1] * scaleY - 5;
                if (labelY < 15 && firstPoint[1] * scaleY + 15 < canvas.height)
                    labelY = firstPoint[1] * scaleY + 15;
                else if (labelY < 15) labelY = 15;

                ctx.font = "14px Arial";
                ctx.fillStyle = `hsl(${hue}, 100%, 35%)`;
                ctx.fillText(annotation.label, labelX, labelY);
            }
        });
    }

    afterUpdate(() => {
        if (
            selectedImage &&
            selectedImage.annotations &&
            selectedImage.annotations.length > 0
        ) {
            setTimeout(() => {
                if (document.getElementById("selected-image-modal")?.complete) {
                    drawAnnotationsOnModal();
                }
            }, 50);
        }
    });

    onMount(() => {
        if (
            selectedImage &&
            selectedImage.annotations &&
            selectedImage.annotations.length > 0
        ) {
            setTimeout(() => {
                if (document.getElementById("selected-image-modal")?.complete) {
                    drawAnnotationsOnModal();
                }
            }, 100);
        }
    });

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === "Escape") {
            close();
        }
    }
</script>

<svelte:window on:keydown={handleKeydown} />

<dialog class="modal" class:modal-open={selectedImage !== null}>
    <div class="modal-box max-w-6xl max-h-[calc(100vh-2rem)]">
        <!-- Header -->
        <div class="flex items-center justify-between mb-4">
            <h3 class="font-bold text-lg flex items-center gap-2 truncate">
                <span class="material-symbols-rounded text-primary">image</span>
                {selectedImage?.name || "Image Details"}
            </h3>
            <button on:click={close} class="btn btn-sm btn-circle btn-ghost">
                <span class="material-symbols-rounded">close</span>
            </button>
        </div>

        <!-- Image Content -->
        <div
            class="flex items-center justify-center min-h-[300px] bg-base-200 rounded-lg p-4"
        >
            <div class="relative">
                {#if selectedImage}
                    <img
                        id="selected-image-modal"
                        src={selectedImage.previewUrl}
                        alt={selectedImage.name}
                        class="max-w-full max-h-[60vh] object-contain rounded-lg"
                        on:load={drawAnnotationsOnModal}
                    />
                    {#if selectedImage.annotations && selectedImage.annotations.length > 0}
                        <canvas
                            id="annotation-canvas-modal"
                            class="absolute top-0 left-0 w-full h-full pointer-events-none"
                        ></canvas>
                    {/if}
                {/if}
            </div>
        </div>

        <!-- Details Section -->
        <div class="mt-4 space-y-3">
            {#if selectedImage?.annotations && selectedImage.annotations.length > 0}
                <div class="collapse collapse-arrow bg-success/10 rounded-lg">
                    <input type="checkbox" checked />
                    <div
                        class="collapse-title text-sm font-medium flex items-center gap-2"
                    >
                        <span class="material-symbols-rounded text-success"
                            >label</span
                        >
                        {annotationType === "bounding_box"
                            ? "Bounding Box"
                            : "Polygon"} Annotations ({selectedImage.annotations
                            .length})
                    </div>
                    <div class="collapse-content">
                        <ul class="space-y-1">
                            {#each selectedImage.annotations as annotation, i (i)}
                                <li
                                    class="flex justify-between items-center text-sm"
                                >
                                    <span class="truncate"
                                        >{annotation.label ||
                                            "Unlabelled"}</span
                                    >
                                    <span class="badge badge-ghost badge-sm"
                                        >{annotation.shape_type}</span
                                    >
                                </li>
                            {/each}
                        </ul>
                    </div>
                </div>
            {/if}

            <div
                class="stats stats-horizontal bg-base-200 shadow w-full text-sm"
            >
                {#if selectedImage?.size != null}
                    <div class="stat py-2 px-4">
                        <div class="stat-title text-xs">Size</div>
                        <div class="stat-value text-sm">
                            {formatFileSize(selectedImage.size)}
                        </div>
                    </div>
                {/if}
                {#if selectedImage?.dimensions}
                    <div class="stat py-2 px-4">
                        <div class="stat-title text-xs">Dimensions</div>
                        <div class="stat-value text-sm">
                            {selectedImage.dimensions.width} × {selectedImage
                                .dimensions.height}
                        </div>
                    </div>
                {/if}
                {#if selectedImage?.created}
                    <div class="stat py-2 px-4">
                        <div class="stat-title text-xs">Created</div>
                        <div class="stat-value text-xs">
                            {new Date(
                                selectedImage.created,
                            ).toLocaleDateString()}
                        </div>
                    </div>
                {/if}
            </div>

            {#if selectedImage?.path}
                <div class="text-xs opacity-60 break-all">
                    <span class="font-medium">Path:</span>
                    {selectedImage.path}
                </div>
            {/if}
        </div>

        <!-- Footer -->
        <div class="modal-action">
            <button class="btn btn-ghost" on:click={close}>Close</button>
        </div>
    </div>
    <form method="dialog" class="modal-backdrop">
        <button on:click={close}>close</button>
    </form>
</dialog>

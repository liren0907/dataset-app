<script lang="ts">
	import { onMount, createEventDispatcher } from "svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";

	let isDragHover: boolean = false;
	let uploadedImage: string | null = null;
	let originalImage: HTMLImageElement | null = null;
	let canvas: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null = null;
	let container: HTMLElement;

	// Crop area properties
	let cropArea = {
		x: 50,
		y: 50,
		width: 200,
		height: 200,
	};

	let isDragging = false;
	let isResizing = false;
	let dragStart = { x: 0, y: 0 };
	let resizeHandle = "";
	let aspectRatio: string = "free";
	let zoom: number = 1;
	let rotation: number = 0;

	// Enhanced interaction states
	let isHoveringCrop = false;
	let isHoveringHandle = false;
	let currentHandle = "";
	let isPanning = false;
	let lastMousePos = { x: 0, y: 0 };

	// Preview properties
	let showPreview = false;
	let previewCanvas: HTMLCanvasElement;
	let previewCtx: CanvasRenderingContext2D | null = null;

	// Reactive statement to handle preview rendering after canvas is available
	$: if (showPreview && previewCanvas && originalImage && canvas) {
		console.log("Preview canvas available, rendering crop preview...");
		renderCropPreview();
	}

	onMount(async () => {
		// Listen for file drop events
		await listen("tauri://file-drop", async (event) => {
			isDragHover = false;
			const files = event.payload as string[];
			if (files.length > 0) {
				await handleFileUpload(files[0]);
			}
		});

		await listen("tauri://file-drop-hover", () => {
			isDragHover = true;
		});

		await listen("tauri://file-drop-cancelled", () => {
			isDragHover = false;
		});

		// Add keyboard shortcuts
		document.addEventListener("keydown", handleKeyDown);
		document.addEventListener("keyup", handleKeyUp);
	});

	function handleKeyDown(event: KeyboardEvent) {
		// Space bar for panning mode
		if (event.code === "Space") {
			event.preventDefault();
			isPanning = true;
			canvas.style.cursor = "grab";
		}

		// Other shortcuts
		switch (event.key) {
			case "Enter":
				if (showPreview) {
					downloadCroppedImage();
				} else {
					showCropPreview();
				}
				break;
			case "Escape":
				if (showPreview) {
					closePreview();
				}
				break;
			case "+":
			case "=":
				event.preventDefault();
				zoomIn();
				break;
			case "-":
				event.preventDefault();
				zoomOut();
				break;
			case "r":
				event.preventDefault();
				resetCrop();
				break;
		}
	}

	function handleKeyUp(event: KeyboardEvent) {
		if (event.code === "Space") {
			isPanning = false;
			updateHoverStates();
		}
	}

	async function selectFile() {
		try {
			const selected = await open({
				multiple: false,
				filters: [
					{
						name: "Images",
						extensions: [
							"png",
							"jpg",
							"jpeg",
							"webp",
							"bmp",
							"gif",
						],
					},
				],
			});

			if (selected) {
				await handleFileUpload(selected as string);
			}
		} catch (error) {
			console.error("Error selecting file:", error);
		}
	}

	async function handleFileUpload(filePath: string) {
		try {
			uploadedImage = convertFileSrc(filePath);
			originalImage = new Image();
			originalImage.onload = () => {
				initCanvas();
				drawImage();
			};
			originalImage.src = uploadedImage;
		} catch (error) {
			console.error("Error loading image:", error);
		}
	}

	function initCanvas() {
		if (!canvas || !originalImage) return;

		const containerRect = container.getBoundingClientRect();
		canvas.width = containerRect.width;
		canvas.height = containerRect.height;

		ctx = canvas.getContext("2d");

		// Initialize crop area to center of image
		const centerX = canvas.width / 2;
		const centerY = canvas.height / 2;
		cropArea.x = centerX - 100;
		cropArea.y = centerY - 100;
		cropArea.width = 200;
		cropArea.height = 200;
	}

	function drawImage() {
		if (!ctx || !originalImage || !canvas) {
			console.error("Missing required elements for drawImage:", {
				ctx: !!ctx,
				originalImage: !!originalImage,
				canvas: !!canvas,
			});
			return;
		}

		ctx.clearRect(0, 0, canvas.width, canvas.height);

		// Save context for rotation
		ctx.save();

		// Apply zoom and rotation
		ctx.translate(canvas.width / 2, canvas.height / 2);
		ctx.scale(zoom, zoom);
		ctx.rotate((rotation * Math.PI) / 180);
		ctx.translate(-canvas.width / 2, -canvas.height / 2);

		// Draw the image (fitted to canvas)
		const canvasAspect = canvas.width / canvas.height;
		const imageAspect = originalImage.width / originalImage.height;

		let scale, scaledWidth, scaledHeight, x, y;

		if (canvasAspect > imageAspect) {
			// Canvas is wider, fit by height
			scale = canvas.height / originalImage.height;
			scaledWidth = originalImage.width * scale;
			scaledHeight = canvas.height;
			x = (canvas.width - scaledWidth) / 2;
			y = 0;
		} else {
			// Canvas is taller, fit by width
			scale = canvas.width / originalImage.width;
			scaledWidth = canvas.width;
			scaledHeight = originalImage.height * scale;
			x = 0;
			y = (canvas.height - scaledHeight) / 2;
		}

		console.log("Drawing image with scale:", scale, "at position:", x, y);

		ctx.drawImage(originalImage, x, y, scaledWidth, scaledHeight);

		// Restore context
		ctx.restore();

		// Draw crop area
		drawCropArea();
	}

	function drawCropArea() {
		if (!ctx || !canvas) return;

		// Enhanced visual feedback based on interaction state
		const isActive = isHoveringCrop || isDragging;
		const isResizingNow = isResizing || isHoveringHandle;

		// Dynamic styling based on state
		if (isResizingNow) {
			ctx.strokeStyle = "#ef4444"; // Red for resizing
			ctx.lineWidth = 3;
		} else if (isActive) {
			ctx.strokeStyle = "#10b981"; // Green for active/hover
			ctx.lineWidth = 2;
		} else {
			ctx.strokeStyle = "#3b82f6"; // Blue for normal
			ctx.lineWidth = 2;
		}

		// Draw crop rectangle with enhanced visual feedback
		ctx.setLineDash(isActive ? [] : [5, 5]);
		ctx.strokeRect(cropArea.x, cropArea.y, cropArea.width, cropArea.height);
		ctx.setLineDash([]);

		// Add glow effect for active state
		if (isActive) {
			ctx.shadowColor = ctx.strokeStyle;
			ctx.shadowBlur = 8;
			ctx.strokeRect(
				cropArea.x,
				cropArea.y,
				cropArea.width,
				cropArea.height,
			);
			ctx.shadowBlur = 0;
		}

		// Draw resize handles with enhanced styling
		const handleSize = isHoveringHandle ? 12 : 8;
		const handles = [
			{ x: cropArea.x, y: cropArea.y, type: "nw" }, // top-left
			{ x: cropArea.x + cropArea.width, y: cropArea.y, type: "ne" }, // top-right
			{ x: cropArea.x, y: cropArea.y + cropArea.height, type: "sw" }, // bottom-left
			{
				x: cropArea.x + cropArea.width,
				y: cropArea.y + cropArea.height,
				type: "se",
			}, // bottom-right
		];

		handles.forEach((handle) => {
			const isHovered = currentHandle === handle.type;

			if (isHovered) {
				ctx.fillStyle = "#ef4444"; // Red for hovered handle
				ctx.strokeStyle = "#ffffff";
				ctx.lineWidth = 2;
			} else if (isHoveringHandle) {
				ctx.fillStyle = "#10b981"; // Green for other handles when hovering
				ctx.strokeStyle = "#ffffff";
				ctx.lineWidth = 1;
			} else {
				ctx.fillStyle = "#3b82f6"; // Blue for normal handles
				ctx.strokeStyle = "#ffffff";
				ctx.lineWidth = 1;
			}

			// Draw handle with stroke
			ctx.fillRect(
				handle.x - handleSize / 2,
				handle.y - handleSize / 2,
				handleSize,
				handleSize,
			);
			ctx.strokeRect(
				handle.x - handleSize / 2,
				handle.y - handleSize / 2,
				handleSize,
				handleSize,
			);
		});

		// Draw crop dimensions info when active
		if (isActive) {
			ctx.fillStyle = "#ffffff";
			ctx.strokeStyle = "#000000";
			ctx.lineWidth = 3;
			ctx.font = "12px Arial";
			const dimensionsText = `${Math.round(cropArea.width)} × ${Math.round(cropArea.height)}`;

			// Position text in top-right of crop area
			const textX = cropArea.x + cropArea.width - 10;
			const textY = cropArea.y + 20;

			// Text background
			const metrics = ctx.measureText(dimensionsText);
			ctx.fillStyle = "rgba(0, 0, 0, 0.7)";
			ctx.fillRect(
				textX - metrics.width - 8,
				textY - 12,
				metrics.width + 16,
				20,
			);

			// Text
			ctx.fillStyle = "#ffffff";
			ctx.strokeStyle = "#000000";
			ctx.lineWidth = 2;
			ctx.strokeText(dimensionsText, textX - metrics.width - 4, textY);
			ctx.fillText(dimensionsText, textX - metrics.width - 4, textY);
		}
	}

	function getMousePos(event: MouseEvent) {
		const rect = canvas.getBoundingClientRect();
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

	function handleMouseDown(event: MouseEvent) {
		if (!canvas) return;

		const pos = getMousePos(event);
		resizeHandle = getResizeHandle(pos.x, pos.y);

		// Check for spacebar + drag (panning mode)
		if (event.code === "Space" || event.key === " ") {
			isPanning = true;
		} else if (resizeHandle) {
			isResizing = true;
		} else if (isInCropArea(pos.x, pos.y)) {
			isDragging = true;
		} else {
			// Click in empty space - start new crop area
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
		if (!canvas) return;

		const pos = getMousePos(event);
		lastMousePos = { x: event.clientX, y: event.clientY };

		// Update hover states for dynamic cursor
		updateHoverStates();

		const dx = pos.x - dragStart.x;
		const dy = pos.y - dragStart.y;

		if (isResizing) {
			resizeCropArea(dx, dy);
		} else if (isDragging) {
			moveCropArea(dx, dy);
		} else if (isPanning) {
			// Handle panning with spacebar + drag
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
		// Reset hover states when entering canvas
		updateHoverStates();
	}

	function updateHoverStates() {
		if (!canvas) return;

		const rect = canvas.getBoundingClientRect();
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
			const cursorMap = {
				nw: "nw-resize",
				ne: "ne-resize",
				sw: "sw-resize",
				se: "se-resize",
			};
			return (
				cursorMap[currentHandle as keyof typeof cursorMap] || "pointer"
			);
		}

		if (isHoveringCrop) {
			return "grab";
		}

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

		// Maintain minimum size
		if (cropArea.width < 20) cropArea.width = 20;
		if (cropArea.height < 20) cropArea.height = 20;

		// Apply aspect ratio if set
		if (aspectRatio !== "free") {
			applyAspectRatio();
		}

		drawImage();
	}

	function moveCropArea(dx: number, dy: number) {
		cropArea.x += dx;
		cropArea.y += dy;

		// Keep crop area within canvas bounds
		if (cropArea.x < 0) cropArea.x = 0;
		if (cropArea.y < 0) cropArea.y = 0;
		if (cropArea.x + cropArea.width > canvas.width)
			cropArea.x = canvas.width - cropArea.width;
		if (cropArea.y + cropArea.height > canvas.height)
			cropArea.y = canvas.height - cropArea.height;

		drawImage();
	}

	function applyAspectRatio() {
		const ratios = {
			square: 1,
			"4:3": 4 / 3,
			"16:9": 16 / 9,
			"3:2": 3 / 2,
			"2:3": 2 / 3,
			"9:16": 9 / 16,
		};

		const ratio = ratios[aspectRatio as keyof typeof ratios];
		if (ratio) {
			cropArea.height = cropArea.width / ratio;
		}
	}

	function setAspectRatio(ratio: string) {
		aspectRatio = ratio;
		if (ratio !== "free") {
			applyAspectRatio();
			drawImage();
		}
	}

	function zoomIn() {
		zoom = Math.min(zoom * 1.2, 5);
		drawImage();
	}

	function zoomOut() {
		zoom = Math.max(zoom / 1.2, 0.1);
		drawImage();
	}

	function resetZoom() {
		zoom = 1;
		drawImage();
	}

	function rotateClockwise() {
		rotation = (rotation + 90) % 360;
		drawImage();
	}

	function rotateCounterClockwise() {
		rotation = (rotation - 90 + 360) % 360;
		drawImage();
	}

	function resetCrop() {
		if (!canvas) return;
		cropArea.x = canvas.width / 2 - 100;
		cropArea.y = canvas.height / 2 - 100;
		cropArea.width = 200;
		cropArea.height = 200;
		zoom = 1;
		rotation = 0;
		aspectRatio = "free";
		drawImage();
	}

	function showCropPreview() {
		// Just open the modal - the actual rendering will happen via reactive statement
		showPreview = true;
	}

	function renderCropPreview() {
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

			console.log("Crop transformation:", {
				canvasCrop: {
					x: cropArea.x,
					y: cropArea.y,
					w: cropArea.width,
					h: cropArea.height,
				},
				imagePosition: { x: imageX, y: imageY, scale: scale },
				originalImage: {
					w: originalImage.width,
					h: originalImage.height,
				},
				finalCrop: {
					x: finalCropX,
					y: finalCropY,
					w: finalCropWidth,
					h: finalCropHeight,
				},
			});

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

			console.log("Crop preview rendered successfully");
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

	function closePreview() {
		showPreview = false;
	}
</script>

<svelte:head>
	<title>Smart Tools - Interactive Crop Tool</title>
	<meta
		name="description"
		content="Advanced image cropping tool with interactive controls"
	/>
</svelte:head>

<div class="container mx-auto px-4 py-8">
	<div class="max-w-7xl mx-auto">
		<div class="mb-8">
			<div class="flex items-center justify-between">
				<div>
					<h1 class="text-3xl font-bold mb-2 flex items-center gap-3">
						<span
							class="material-symbols-rounded text-primary text-4xl"
							>construction</span
						>
						Smart Tools
					</h1>
					<p class="opacity-70">
						Advanced interactive image cropping and processing tools
					</p>
				</div>
				<div class="badge badge-success gap-1">
					<span class="material-symbols-rounded icon-sm"
						>verified</span
					>
					Pro Tools
				</div>
			</div>
		</div>

		<!-- Keyboard Shortcuts Help -->
		<div class="alert bg-base-200 mb-6">
			<span class="material-symbols-rounded">keyboard</span>
			<div class="flex flex-wrap items-center gap-4 text-sm">
				<span><kbd class="kbd kbd-sm">Space</kbd> + Drag to pan</span>
				<span><kbd class="kbd kbd-sm">Enter</kbd> Preview/Download</span
				>
				<span><kbd class="kbd kbd-sm">Esc</kbd> Close preview</span>
				<span><kbd class="kbd kbd-sm">+/-</kbd> Zoom</span>
				<span><kbd class="kbd kbd-sm">R</kbd> Reset</span>
			</div>
			<div class="text-xs opacity-60">
				<span class="material-symbols-rounded icon-sm">lightbulb</span>
				Click empty space to create new crop area
			</div>
		</div>

		<!-- Crop Tool Section -->
		<div class="card bg-base-100 shadow-xl mb-8">
			<div class="card-body">
				<h2 class="card-title text-2xl flex items-center gap-2">
					<span class="material-symbols-rounded text-primary"
						>crop</span
					>
					Interactive Crop Tool
				</h2>

				{#if !uploadedImage}
					<!-- Upload Area -->
					<div
						class="border-2 border-dashed rounded-xl p-12 text-center transition-all duration-200
						{isDragHover
							? 'border-primary bg-primary/10'
							: 'border-base-300 hover:border-primary/50'}"
					>
						<div class="space-y-4">
							{#if isDragHover}
								<span
									class="material-symbols-rounded text-6xl text-primary"
									>upload_file</span
								>
								<p class="text-lg font-medium">
									Drop your image here
								</p>
							{:else}
								<span
									class="material-symbols-rounded text-6xl opacity-50"
									>image</span
								>
								<p class="text-lg">
									Drag and drop an image here
								</p>
								<p class="text-sm opacity-60">or</p>
							{/if}
							<button
								on:click={selectFile}
								class="btn btn-primary"
							>
								<span class="material-symbols-rounded"
									>cloud_upload</span
								>
								Choose Image
							</button>
							<p class="text-xs opacity-50">
								Supported formats: PNG, JPG, JPEG, WEBP, BMP,
								GIF
							</p>
						</div>
					</div>
				{:else}
					<!-- Crop Interface -->
					<div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
						<!-- Canvas Area -->
						<div class="lg:col-span-3">
							<div
								bind:this={container}
								class="relative bg-gradient-to-br from-gray-50 to-gray-100 rounded-xl overflow-hidden shadow-inner border-2 border-gray-200"
								style="height: 600px;"
							>
								<!-- Canvas Grid Background -->
								<div class="absolute inset-0 opacity-10">
									<svg
										width="100%"
										height="100%"
										xmlns="http://www.w3.org/2000/svg"
									>
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
										<rect
											width="100%"
											height="100%"
											fill="url(#grid)"
										/>
									</svg>
								</div>

								<!-- Main Canvas -->
								<canvas
									bind:this={canvas}
									on:mousedown={handleMouseDown}
									on:mousemove={handleMouseMove}
									on:mouseup={handleMouseUp}
									on:mouseleave={handleMouseLeave}
									on:mouseenter={handleMouseEnter}
									class="relative z-10 w-full h-full transition-all duration-200"
									style="cursor: {getCanvasCursor()};"
								></canvas>

								<!-- Canvas Info Overlay -->
								<div
									class="absolute top-4 left-4 bg-black bg-opacity-50 text-white px-3 py-1 rounded-lg text-sm z-20"
								>
									Zoom: {Math.round(zoom * 100)}% | {rotation}°
								</div>
							</div>

							<!-- Canvas Instructions -->
							<div class="mt-3 text-xs text-gray-500 text-center">
								<span class="inline-flex items-center">
									<svg
										class="w-3 h-3 mr-1"
										fill="currentColor"
										viewBox="0 0 20 20"
									>
										<path
											fill-rule="evenodd"
											d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
											clip-rule="evenodd"
										/>
									</svg>
									Drag handles to resize • Drag area to move •
									Click empty space for new crop
								</span>
							</div>
						</div>

						<!-- Controls Panel -->
						<div class="space-y-4">
							<!-- Status Indicator -->
							<div class="alert alert-success">
								<span class="loading loading-ring loading-sm"
								></span>
								<div>
									<h3 class="font-bold">
										Interactive Crop Tool
									</h3>
									<div class="text-xs">
										Ready for cropping
									</div>
								</div>
							</div>

							<!-- Crop Controls Card -->
							<div class="card bg-base-200 shadow-sm">
								<div class="card-body p-4">
									<h3
										class="card-title text-base flex items-center gap-2"
									>
										<span
											class="material-symbols-rounded text-primary"
											>tune</span
										>
										Crop Controls
									</h3>

									<!-- Aspect Ratio -->
									<div class="form-control">
										<label class="label"
											><span class="label-text"
												>Aspect Ratio</span
											></label
										>
										<select
											bind:value={aspectRatio}
											on:change={(e) =>
												setAspectRatio(e.target.value)}
											class="select select-bordered select-sm w-full"
										>
											<option value="free"
												>Free Form</option
											>
											<option value="square"
												>Square (1:1)</option
											>
											<option value="4:3"
												>Landscape (4:3)</option
											>
											<option value="16:9"
												>Wide (16:9)</option
											>
											<option value="3:2"
												>Photo (3:2)</option
											>
											<option value="2:3"
												>Portrait (2:3)</option
											>
											<option value="9:16"
												>Tall (9:16)</option
											>
										</select>
									</div>

									<!-- Zoom Controls -->
									<div class="form-control">
										<label class="label"
											><span class="label-text">Zoom</span
											></label
										>
										<div class="join">
											<button
												on:click={zoomOut}
												class="btn btn-sm join-item"
												>-</button
											>
											<span
												class="btn btn-sm join-item btn-disabled"
												>{Math.round(zoom * 100)}%</span
											>
											<button
												on:click={zoomIn}
												class="btn btn-sm join-item"
												>+</button
											>
											<button
												on:click={resetZoom}
												class="btn btn-sm join-item"
												>Reset</button
											>
										</div>
									</div>

									<!-- Rotation Controls -->
									<div class="form-control">
										<label class="label"
											><span class="label-text"
												>Rotation</span
											></label
										>
										<div class="join">
											<button
												on:click={rotateCounterClockwise}
												class="btn btn-sm join-item"
											>
												<span
													class="material-symbols-rounded icon-sm"
													>rotate_left</span
												>
											</button>
											<span
												class="btn btn-sm join-item btn-disabled"
												>{rotation}°</span
											>
											<button
												on:click={rotateClockwise}
												class="btn btn-sm join-item"
											>
												<span
													class="material-symbols-rounded icon-sm"
													>rotate_right</span
												>
											</button>
										</div>
									</div>

									<!-- Action Buttons -->
									<div class="space-y-2 mt-4">
										<button
											on:click={showCropPreview}
											class="btn btn-primary btn-block"
										>
											<span
												class="material-symbols-rounded"
												>visibility</span
											>
											Preview Crop
										</button>

										<div class="grid grid-cols-2 gap-2">
											<button
												on:click={resetCrop}
												class="btn btn-ghost btn-sm"
											>
												<span
													class="material-symbols-rounded icon-sm"
													>restart_alt</span
												>
												Reset
											</button>
											<button
												on:click={() => {
													uploadedImage = null;
													originalImage = null;
												}}
												class="btn btn-error btn-sm"
											>
												<span
													class="material-symbols-rounded icon-sm"
													>delete</span
												>
												Clear
											</button>
										</div>
									</div>
								</div>
							</div>

							<!-- Crop Info -->
							<div class="stats stats-vertical shadow w-full">
								<div class="stat py-2">
									<div class="stat-title text-xs">
										Position
									</div>
									<div class="stat-value text-sm">
										{Math.round(cropArea.x)}, {Math.round(
											cropArea.y,
										)}
									</div>
								</div>
								<div class="stat py-2">
									<div class="stat-title text-xs">Size</div>
									<div class="stat-value text-sm">
										{Math.round(cropArea.width)} × {Math.round(
											cropArea.height,
										)}
									</div>
								</div>
								<div class="stat py-2">
									<div class="stat-title text-xs">Ratio</div>
									<div class="stat-value text-sm">
										{(
											cropArea.width / cropArea.height
										).toFixed(2)}
									</div>
								</div>
							</div>
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
</div>

<!-- Crop Preview Modal -->
<dialog class="modal" class:modal-open={showPreview}>
	<div class="modal-box max-w-3xl">
		<!-- Header -->
		<div class="flex items-center justify-between mb-4">
			<h3 class="font-bold text-lg flex items-center gap-2">
				<span class="material-symbols-rounded text-primary"
					>visibility</span
				>
				Crop Preview
			</h3>
			<button
				on:click={closePreview}
				class="btn btn-sm btn-circle btn-ghost"
			>
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
			<button on:click={closePreview} class="btn btn-ghost">Cancel</button
			>
			<button on:click={downloadCroppedImage} class="btn btn-success">
				<span class="material-symbols-rounded">download</span>
				Download
			</button>
		</div>
	</div>
	<form method="dialog" class="modal-backdrop">
		<button on:click={closePreview}>close</button>
	</form>
</dialog>

<style>
	canvas {
		cursor: move;
	}

	/* Custom cursor for resize handles */
	canvas:hover {
		cursor: move;
	}
</style>

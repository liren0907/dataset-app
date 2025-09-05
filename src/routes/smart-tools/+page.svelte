<script lang="ts">
	import { onMount, createEventDispatcher } from 'svelte';
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
		height: 200
	};

	let isDragging = false;
	let isResizing = false;
	let dragStart = { x: 0, y: 0 };
	let resizeHandle = '';
	let aspectRatio: string = 'free';
	let zoom: number = 1;
	let rotation: number = 0;

	// Enhanced interaction states
	let isHoveringCrop = false;
	let isHoveringHandle = false;
	let currentHandle = '';
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
		document.addEventListener('keydown', handleKeyDown);
		document.addEventListener('keyup', handleKeyUp);
	});

	function handleKeyDown(event: KeyboardEvent) {
		// Space bar for panning mode
		if (event.code === 'Space') {
			event.preventDefault();
			isPanning = true;
			canvas.style.cursor = 'grab';
		}

		// Other shortcuts
		switch (event.key) {
			case 'Enter':
				if (showPreview) {
					downloadCroppedImage();
				} else {
					showCropPreview();
				}
				break;
			case 'Escape':
				if (showPreview) {
					closePreview();
				}
				break;
			case '+':
			case '=':
				event.preventDefault();
				zoomIn();
				break;
			case '-':
				event.preventDefault();
				zoomOut();
				break;
			case 'r':
				event.preventDefault();
				resetCrop();
				break;
		}
	}

	function handleKeyUp(event: KeyboardEvent) {
		if (event.code === 'Space') {
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
						extensions: ["png", "jpg", "jpeg", "webp", "bmp", "gif"]
					}
				]
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

		ctx = canvas.getContext('2d');

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
			console.error("Missing required elements for drawImage:", { ctx: !!ctx, originalImage: !!originalImage, canvas: !!canvas });
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
			ctx.strokeStyle = '#ef4444'; // Red for resizing
			ctx.lineWidth = 3;
		} else if (isActive) {
			ctx.strokeStyle = '#10b981'; // Green for active/hover
			ctx.lineWidth = 2;
		} else {
			ctx.strokeStyle = '#3b82f6'; // Blue for normal
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
			ctx.strokeRect(cropArea.x, cropArea.y, cropArea.width, cropArea.height);
			ctx.shadowBlur = 0;
		}

		// Draw resize handles with enhanced styling
		const handleSize = isHoveringHandle ? 12 : 8;
		const handles = [
			{ x: cropArea.x, y: cropArea.y, type: 'nw' }, // top-left
			{ x: cropArea.x + cropArea.width, y: cropArea.y, type: 'ne' }, // top-right
			{ x: cropArea.x, y: cropArea.y + cropArea.height, type: 'sw' }, // bottom-left
			{ x: cropArea.x + cropArea.width, y: cropArea.y + cropArea.height, type: 'se' } // bottom-right
		];

		handles.forEach(handle => {
			const isHovered = currentHandle === handle.type;

			if (isHovered) {
				ctx.fillStyle = '#ef4444'; // Red for hovered handle
				ctx.strokeStyle = '#ffffff';
				ctx.lineWidth = 2;
			} else if (isHoveringHandle) {
				ctx.fillStyle = '#10b981'; // Green for other handles when hovering
				ctx.strokeStyle = '#ffffff';
				ctx.lineWidth = 1;
			} else {
				ctx.fillStyle = '#3b82f6'; // Blue for normal handles
				ctx.strokeStyle = '#ffffff';
				ctx.lineWidth = 1;
			}

			// Draw handle with stroke
			ctx.fillRect(handle.x - handleSize/2, handle.y - handleSize/2, handleSize, handleSize);
			ctx.strokeRect(handle.x - handleSize/2, handle.y - handleSize/2, handleSize, handleSize);
		});

		// Draw crop dimensions info when active
		if (isActive) {
			ctx.fillStyle = '#ffffff';
			ctx.strokeStyle = '#000000';
			ctx.lineWidth = 3;
			ctx.font = '12px Arial';
			const dimensionsText = `${Math.round(cropArea.width)} × ${Math.round(cropArea.height)}`;

			// Position text in top-right of crop area
			const textX = cropArea.x + cropArea.width - 10;
			const textY = cropArea.y + 20;

			// Text background
			const metrics = ctx.measureText(dimensionsText);
			ctx.fillStyle = 'rgba(0, 0, 0, 0.7)';
			ctx.fillRect(textX - metrics.width - 8, textY - 12, metrics.width + 16, 20);

			// Text
			ctx.fillStyle = '#ffffff';
			ctx.strokeStyle = '#000000';
			ctx.lineWidth = 2;
			ctx.strokeText(dimensionsText, textX - metrics.width - 4, textY);
			ctx.fillText(dimensionsText, textX - metrics.width - 4, textY);
		}
	}

	function getMousePos(event: MouseEvent) {
		const rect = canvas.getBoundingClientRect();
		return {
			x: event.clientX - rect.left,
			y: event.clientY - rect.top
		};
	}

	function isInCropArea(x: number, y: number) {
		return x >= cropArea.x && x <= cropArea.x + cropArea.width &&
			   y >= cropArea.y && y <= cropArea.y + cropArea.height;
	}

	function getResizeHandle(x: number, y: number) {
		const handleSize = 12;
		const handles = [
			{ pos: 'nw', x: cropArea.x, y: cropArea.y },
			{ pos: 'ne', x: cropArea.x + cropArea.width, y: cropArea.y },
			{ pos: 'sw', x: cropArea.x, y: cropArea.y + cropArea.height },
			{ pos: 'se', x: cropArea.x + cropArea.width, y: cropArea.y + cropArea.height }
		];

		for (const handle of handles) {
			if (Math.abs(x - handle.x) < handleSize && Math.abs(y - handle.y) < handleSize) {
				return handle.pos;
			}
		}
		return '';
	}

	function handleMouseDown(event: MouseEvent) {
		if (!canvas) return;

		const pos = getMousePos(event);
		resizeHandle = getResizeHandle(pos.x, pos.y);

		// Check for spacebar + drag (panning mode)
		if (event.code === 'Space' || event.key === ' ') {
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
		resizeHandle = '';
		isPanning = false;
	}

	function handleMouseLeave() {
		handleMouseUp();
		isHoveringCrop = false;
		isHoveringHandle = false;
		currentHandle = '';
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
		isHoveringHandle = currentHandle !== '';
		isHoveringCrop = !isHoveringHandle && isInCropArea(mouseX, mouseY);
	}

	function getCanvasCursor() {
		if (isDragging || isPanning) {
			return isResizing ? 'grabbing' : 'grabbing';
		}

		if (isHoveringHandle) {
			const cursorMap = {
				'nw': 'nw-resize',
				'ne': 'ne-resize',
				'sw': 'sw-resize',
				'se': 'se-resize'
			};
			return cursorMap[currentHandle as keyof typeof cursorMap] || 'pointer';
		}

		if (isHoveringCrop) {
			return 'grab';
		}

		return 'crosshair';
	}

	function resizeCropArea(dx: number, dy: number) {
		switch (resizeHandle) {
			case 'nw':
				cropArea.x += dx;
				cropArea.y += dy;
				cropArea.width -= dx;
				cropArea.height -= dy;
				break;
			case 'ne':
				cropArea.y += dy;
				cropArea.width += dx;
				cropArea.height -= dy;
				break;
			case 'sw':
				cropArea.x += dx;
				cropArea.width -= dx;
				cropArea.height += dy;
				break;
			case 'se':
				cropArea.width += dx;
				cropArea.height += dy;
				break;
		}

		// Maintain minimum size
		if (cropArea.width < 20) cropArea.width = 20;
		if (cropArea.height < 20) cropArea.height = 20;

		// Apply aspect ratio if set
		if (aspectRatio !== 'free') {
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
		if (cropArea.x + cropArea.width > canvas.width) cropArea.x = canvas.width - cropArea.width;
		if (cropArea.y + cropArea.height > canvas.height) cropArea.y = canvas.height - cropArea.height;

		drawImage();
	}

	function applyAspectRatio() {
		const ratios = {
			'square': 1,
			'4:3': 4/3,
			'16:9': 16/9,
			'3:2': 3/2,
			'2:3': 2/3,
			'9:16': 9/16
		};

		const ratio = ratios[aspectRatio as keyof typeof ratios];
		if (ratio) {
			cropArea.height = cropArea.width / ratio;
		}
	}

	function setAspectRatio(ratio: string) {
		aspectRatio = ratio;
		if (ratio !== 'free') {
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
		aspectRatio = 'free';
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
			const finalCropWidth = Math.min(cropWidth, originalImage.width - finalCropX);
			const finalCropHeight = Math.min(cropHeight, originalImage.height - finalCropY);

			console.log("Crop transformation:", {
				canvasCrop: { x: cropArea.x, y: cropArea.y, w: cropArea.width, h: cropArea.height },
				imagePosition: { x: imageX, y: imageY, scale: scale },
				originalImage: { w: originalImage.width, h: originalImage.height },
				finalCrop: { x: finalCropX, y: finalCropY, w: finalCropWidth, h: finalCropHeight }
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
			previewCtx = previewCanvas.getContext('2d');
			if (!previewCtx) {
				console.error("Could not get preview canvas context");
				return;
			}

			// Clear canvas
			previewCtx.clearRect(0, 0, previewCanvas.width, previewCanvas.height);

			// Save context for transformations
			previewCtx.save();

			// Apply rotation if needed
			if (rotation !== 0) {
				previewCtx.translate(previewCanvas.width / 2, previewCanvas.height / 2);
				previewCtx.rotate((rotation * Math.PI) / 180);
				previewCtx.translate(-previewCanvas.width / 2, -previewCanvas.height / 2);
			}

			// Draw the cropped portion of the original image
			previewCtx.drawImage(
				originalImage,
				finalCropX, finalCropY, finalCropWidth, finalCropHeight,  // Source rectangle on original image
				0, 0, previewCanvas.width, previewCanvas.height  // Destination rectangle on preview canvas
			);

			previewCtx.restore();

			console.log("Crop preview rendered successfully");

		} catch (error) {
			console.error("Error in renderCropPreview:", error);
		}
	}

	function downloadCroppedImage() {
		if (!previewCanvas) return;

		const link = document.createElement('a');
		link.download = 'cropped-image.png';
		link.href = previewCanvas.toDataURL();
		link.click();
	}

	function closePreview() {
		showPreview = false;
	}
</script>

<svelte:head>
	<title>Smart Tools - Interactive Crop Tool</title>
	<meta name="description" content="Advanced image cropping tool with interactive controls" />
</svelte:head>

<div class="container mx-auto px-4 py-8">
			<div class="max-w-7xl mx-auto">
			<div class="mb-8">
				<div class="flex items-center justify-between">
					<div>
						<h1 class="text-3xl font-bold text-gray-800 mb-2">Smart Tools</h1>
						<p class="text-gray-600">Advanced interactive image cropping and processing tools</p>
					</div>
					<div class="flex items-center space-x-2">
						<span class="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
							<svg class="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
								<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
							</svg>
							Pro Tools
						</span>
					</div>
				</div>
			</div>

			<!-- Keyboard Shortcuts Help -->
			<div class="bg-gradient-to-r from-blue-50 to-indigo-50 border border-blue-200 rounded-lg p-4 mb-6">
				<div class="flex items-center justify-between">
					<div class="flex items-center space-x-4 text-sm text-blue-700">
						<div class="flex items-center space-x-1">
							<kbd class="px-2 py-1 bg-blue-200 rounded text-xs">Space</kbd>
							<span>+ Drag to pan</span>
						</div>
						<div class="flex items-center space-x-1">
							<kbd class="px-2 py-1 bg-blue-200 rounded text-xs">Enter</kbd>
							<span>Preview/Download</span>
						</div>
						<div class="flex items-center space-x-1">
							<kbd class="px-2 py-1 bg-blue-200 rounded text-xs">Esc</kbd>
							<span>Close preview</span>
						</div>
						<div class="flex items-center space-x-1">
							<kbd class="px-2 py-1 bg-blue-200 rounded text-xs">+/-</kbd>
							<span>Zoom</span>
						</div>
						<div class="flex items-center space-x-1">
							<kbd class="px-2 py-1 bg-blue-200 rounded text-xs">R</kbd>
							<span>Reset</span>
						</div>
					</div>
					<div class="text-xs text-blue-600">
						Pro tip: Click empty space to create new crop area
					</div>
				</div>
			</div>

		<!-- Crop Tool Section -->
		<div class="bg-white rounded-lg shadow-lg p-6 mb-8">
			<h2 class="text-2xl font-semibold text-gray-800 mb-6">Interactive Crop Tool</h2>

			{#if !uploadedImage}
				<!-- Upload Area -->
				<div
					class="border-2 border-dashed border-gray-300 rounded-lg p-12 text-center transition-colors duration-200
						{isDragHover ? 'border-blue-500 bg-blue-50' : 'hover:border-gray-400'}"
				>
					<div class="space-y-4">
						<div class="text-gray-500">
							{#if isDragHover}
								<p class="text-lg">Drop your image here</p>
							{:else}
								<p class="text-lg">Drag and drop an image here</p>
								<p class="text-sm mt-2">or</p>
							{/if}
						</div>
						<button
							on:click={selectFile}
							class="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="-ml-1 mr-3 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
							</svg>
							Choose Image
						</button>
						<p class="text-xs text-gray-400 mt-2">Supported formats: PNG, JPG, JPEG, WEBP, BMP, GIF</p>
					</div>
				</div>
			{:else}
				<!-- Crop Interface -->
				<div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
					<!-- Canvas Area -->
					<div class="lg:col-span-3">
						<div bind:this={container} class="relative bg-gradient-to-br from-gray-50 to-gray-100 rounded-xl overflow-hidden shadow-inner border-2 border-gray-200" style="height: 600px;">
							<!-- Canvas Grid Background -->
							<div class="absolute inset-0 opacity-10">
								<svg width="100%" height="100%" xmlns="http://www.w3.org/2000/svg">
									<defs>
										<pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
											<path d="M 20 0 L 0 0 0 20" fill="none" stroke="#6b7280" stroke-width="1"/>
										</pattern>
									</defs>
									<rect width="100%" height="100%" fill="url(#grid)" />
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
							<div class="absolute top-4 left-4 bg-black bg-opacity-50 text-white px-3 py-1 rounded-lg text-sm z-20">
								Zoom: {Math.round(zoom * 100)}% | {rotation}°
							</div>
						</div>

						<!-- Canvas Instructions -->
						<div class="mt-3 text-xs text-gray-500 text-center">
							<span class="inline-flex items-center">
								<svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
									<path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"/>
								</svg>
								Drag handles to resize • Drag area to move • Click empty space for new crop
							</span>
						</div>
					</div>

					<!-- Controls Panel -->
					<div class="space-y-6">
						<!-- Status Indicator -->
						<div class="bg-gradient-to-r from-green-50 to-emerald-50 border border-green-200 rounded-lg p-4">
							<div class="flex items-center space-x-3">
								<div class="flex-shrink-0">
									<div class="w-3 h-3 bg-green-400 rounded-full animate-pulse"></div>
								</div>
								<div>
									<h3 class="text-sm font-medium text-green-800">Interactive Crop Tool</h3>
									<p class="text-xs text-green-600">Ready for cropping</p>
								</div>
							</div>
						</div>

						<!-- Crop Controls -->
						<div class="bg-white border border-gray-200 p-4 rounded-lg shadow-sm">
							<h3 class="text-lg font-medium text-gray-800 mb-3 flex items-center">
								<svg class="w-5 h-5 mr-2 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
								</svg>
								Crop Controls
							</h3>

							<!-- Aspect Ratio -->
							<div class="mb-4">
								<label class="block text-sm font-medium text-gray-700 mb-2">Aspect Ratio</label>
								<select bind:value={aspectRatio} on:change={(e) => setAspectRatio(e.target.value)} class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500">
									<option value="free">Free Form</option>
									<option value="square">Square (1:1)</option>
									<option value="4:3">Landscape (4:3)</option>
									<option value="16:9">Wide (16:9)</option>
									<option value="3:2">Photo (3:2)</option>
									<option value="2:3">Portrait (2:3)</option>
									<option value="9:16">Tall (9:16)</option>
								</select>
							</div>

							<!-- Zoom Controls -->
							<div class="mb-4">
								<label class="block text-sm font-medium text-gray-700 mb-2">Zoom</label>
								<div class="flex gap-2">
									<button on:click={zoomOut} class="px-3 py-1 bg-gray-200 hover:bg-gray-300 rounded text-sm">-</button>
									<span class="px-3 py-1 bg-white border border-gray-300 rounded text-sm">{Math.round(zoom * 100)}%</span>
									<button on:click={zoomIn} class="px-3 py-1 bg-gray-200 hover:bg-gray-300 rounded text-sm">+</button>
									<button on:click={resetZoom} class="px-3 py-1 bg-gray-200 hover:bg-gray-300 rounded text-sm">Reset</button>
								</div>
							</div>

							<!-- Rotation Controls -->
							<div class="mb-4">
								<label class="block text-sm font-medium text-gray-700 mb-2">Rotation</label>
								<div class="flex gap-2">
									<button on:click={rotateCounterClockwise} class="px-3 py-1 bg-gray-200 hover:bg-gray-300 rounded text-sm">↺</button>
									<span class="px-3 py-1 bg-white border border-gray-300 rounded text-sm">{rotation}°</span>
									<button on:click={rotateClockwise} class="px-3 py-1 bg-gray-200 hover:bg-gray-300 rounded text-sm">↻</button>
								</div>
							</div>

							<!-- Action Buttons -->
							<div class="space-y-3">
								<button
									on:click={showCropPreview}
									class="w-full px-4 py-3 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 text-white rounded-lg transition-all duration-200 transform hover:scale-105 shadow-lg hover:shadow-xl flex items-center justify-center"
								>
									<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
									</svg>
									Preview Crop
								</button>

								<div class="grid grid-cols-2 gap-2">
									<button
										on:click={resetCrop}
										class="px-3 py-2 bg-gradient-to-r from-gray-600 to-gray-700 hover:from-gray-700 hover:to-gray-800 text-white rounded-lg transition-all duration-200 transform hover:scale-105 shadow-md hover:shadow-lg flex items-center justify-center text-sm"
									>
										<svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
										</svg>
										Reset
									</button>

									<button
										on:click={() => { uploadedImage = null; originalImage = null; }}
										class="px-3 py-2 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-700 hover:to-red-800 text-white rounded-lg transition-all duration-200 transform hover:scale-105 shadow-md hover:shadow-lg flex items-center justify-center text-sm"
									>
										<svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
										</svg>
										Clear
									</button>
								</div>
							</div>
						</div>

						<!-- Crop Info -->
						<div class="bg-gray-50 p-4 rounded-lg">
							<h3 class="text-lg font-medium text-gray-800 mb-3">Crop Info</h3>
							<div class="space-y-2 text-sm">
								<p><span class="font-medium">Position:</span> {Math.round(cropArea.x)}, {Math.round(cropArea.y)}</p>
								<p><span class="font-medium">Size:</span> {Math.round(cropArea.width)} × {Math.round(cropArea.height)}</p>
								<p><span class="font-medium">Ratio:</span> {(cropArea.width / cropArea.height).toFixed(2)}</p>
							</div>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>

<!-- Crop Preview Modal -->
{#if showPreview}
	<div class="fixed inset-0 bg-black bg-opacity-60 backdrop-blur-sm flex items-center justify-center z-50 p-4 animate-in fade-in duration-200">
		<div class="bg-white rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh] max-w-3xl w-full transform animate-in zoom-in-95 duration-200">
			<!-- Header -->
			<div class="px-6 py-4 bg-gradient-to-r from-blue-600 to-indigo-600 text-white flex justify-between items-center">
				<div class="flex items-center space-x-3">
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
					</svg>
					<h3 class="text-xl font-bold">Crop Preview</h3>
				</div>
				<button
					on:click={closePreview}
					class="text-white hover:bg-white hover:bg-opacity-20 p-2 rounded-lg transition-all duration-200 transform hover:scale-110"
				>
					<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-5 h-5">
						<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>

			<!-- Preview Content -->
			<div class="p-8 flex justify-center bg-gray-50">
				<div class="relative">
					<canvas
						bind:this={previewCanvas}
						class="border-4 border-white shadow-lg rounded-lg"
					></canvas>
					<!-- Success Badge -->
					<div class="absolute -top-3 -right-3 bg-green-500 text-white px-2 py-1 rounded-full text-xs font-medium flex items-center">
						<svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
							<path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
						</svg>
						Cropped
					</div>
				</div>
			</div>

			<!-- Footer -->
			<div class="px-6 py-4 bg-gray-50 flex justify-between items-center">
				<div class="text-sm text-gray-600">
					<svg class="w-4 h-4 inline mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
					</svg>
					High quality preview
				</div>
				<div class="flex space-x-3">
					<button
						on:click={closePreview}
						class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-100 text-gray-700 transition-all duration-200 transform hover:scale-105"
					>
						Cancel
					</button>
					<button
						on:click={downloadCroppedImage}
						class="bg-gradient-to-r from-green-600 to-green-700 hover:from-green-700 hover:to-green-800 text-white px-6 py-2 rounded-lg transition-all duration-200 transform hover:scale-105 shadow-lg hover:shadow-xl flex items-center"
					>
						<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
						</svg>
						Download
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	canvas {
		cursor: move;
	}

	/* Custom cursor for resize handles */
	canvas:hover {
		cursor: move;
	}
</style>

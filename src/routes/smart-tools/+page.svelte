<script lang="ts">
	import { onMount } from "svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";

	// Components
	import ShortcutsGuide from "./components/ShortcutsGuide.svelte";
	import UploadArea from "./components/UploadArea.svelte";
	import CropCanvas from "./components/CropCanvas.svelte";
	import CropControls from "./components/CropControls.svelte";
	import CropInfo from "./components/CropInfo.svelte";
	import PreviewModal from "./components/PreviewModal.svelte";

	let isDragHover: boolean = false;
	let uploadedImage: string | null = null;
	let originalImage: HTMLImageElement | null = null;

	// Canvas reference to pass between CropCanvas and PreviewModal logic
	let canvasComponent: CropCanvas;
	let canvasElement: HTMLCanvasElement;

	// Crop area properties (Shared State)
	let cropArea = {
		x: 50,
		y: 50,
		width: 200,
		height: 200,
	};

	// State
	let aspectRatio: string = "free";
	let zoom: number = 1;
	let rotation: number = 0;
	let showPreview = false;

	onMount(() => {
		let unlisteners: (() => void)[] = [];

		const setup = async () => {
			// Listen for file Drop events
			const unlistenDrop = await listen(
				"tauri://file-drop",
				async (event) => {
					isDragHover = false;
					const files = event.payload as string[];
					if (files.length > 0) {
						await handleFileUpload(files[0]);
					}
				},
			);
			unlisteners.push(unlistenDrop);

			const unlistenHover = await listen(
				"tauri://file-drop-hover",
				() => {
					isDragHover = true;
				},
			);
			unlisteners.push(unlistenHover);

			const unlistenCancel = await listen(
				"tauri://file-drop-cancelled",
				() => {
					isDragHover = false;
				},
			);
			unlisteners.push(unlistenCancel);
		};

		setup();

		// Add keyboard shortcuts
		document.addEventListener("keydown", handleKeyDown);
		document.addEventListener("keyup", handleKeyUp);

		return () => {
			document.removeEventListener("keydown", handleKeyDown);
			document.removeEventListener("keyup", handleKeyUp);
			unlisteners.forEach((fn) => fn());
		};
	});

	function handleKeyDown(event: KeyboardEvent) {
		// Delegate specific actions
		if (event.code === "Space") {
			event.preventDefault();
			if (canvasComponent) canvasComponent.setPanning(true);
		}

		switch (event.key) {
			case "Enter":
				if (showPreview) {
					// Logic inside modal or trigger click? Modal handles it via its own buttons or we can call logic
					// For now, simpler to let user click or we can export a download method from modal if needed
					// But standard refactor separates concerns. Let's toggle preview.
				} else if (uploadedImage) {
					showPreview = true;
				}
				break;
			case "Escape":
				if (showPreview) showPreview = false;
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
			if (canvasComponent) canvasComponent.setPanning(false);
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
				// Canvas component will auto-init on prop change or we can force it
				if (canvasComponent) canvasComponent.initCanvas();
			};
			originalImage.src = uploadedImage;
		} catch (error) {
			console.error("Error loading image:", error);
		}
	}

	// --- Actions ---
	function zoomIn() {
		zoom = Math.min(zoom * 1.2, 5);
	}

	function zoomOut() {
		zoom = Math.max(zoom / 1.2, 0.1);
	}

	function resetZoom() {
		zoom = 1;
	}

	function rotateCW() {
		rotation = (rotation + 90) % 360;
	}

	function rotateCCW() {
		rotation = (rotation - 90 + 360) % 360;
	}

	function resetCrop() {
		if (!canvasElement) return;
		// Logic specific to canvas size, maybe ideally belongs in CropCanvas, but we can manage state here
		cropArea.x = canvasElement.width / 2 - 100;
		cropArea.y = canvasElement.height / 2 - 100;
		cropArea.width = 200;
		cropArea.height = 200;
		zoom = 1;
		rotation = 0;
		aspectRatio = "free";
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

		<ShortcutsGuide />

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
					<UploadArea {isDragHover} on:selectFile={selectFile} />
				{:else}
					<!-- Crop Interface -->
					<div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
						<!-- Canvas Area -->
						<div class="lg:col-span-3">
							<CropCanvas
								bind:this={canvasComponent}
								bind:canvasElement
								{originalImage}
								{zoom}
								{rotation}
								{aspectRatio}
								bind:cropArea
							/>
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

							<CropControls
								bind:aspectRatio
								{zoom}
								{rotation}
								on:ratioChange={(e) => (aspectRatio = e.detail)}
								on:zoomIn={zoomIn}
								on:zoomOut={zoomOut}
								on:zoomReset={resetZoom}
								on:rotateCW={rotateCW}
								on:rotateCCW={rotateCCW}
								on:preview={() => (showPreview = true)}
								on:reset={resetCrop}
								on:clear={() => {
									uploadedImage = null;
									originalImage = null;
								}}
							/>

							<CropInfo {cropArea} />
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
</div>

<PreviewModal
	isOpen={showPreview}
	{originalImage}
	{cropArea}
	{rotation}
	{zoom}
	canvas={canvasElement}
	on:close={() => (showPreview = false)}
/>

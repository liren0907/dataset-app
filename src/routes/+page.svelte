<script lang="ts">
	import { onMount } from "svelte";
	import { dragHandling } from "../funcs/file";
	import { convertFileSrc, invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { type } from "@tauri-apps/plugin-os";

	let isDragHover: boolean = false;

	onMount(async () => {
		const unlisten = await listen("tauri://file-drop", async (event) => {
			isDragHover = false;
		});
		await listen("tauri://file-drop-hover", async () => {
			isDragHover = true;
		});
		await listen("tauri://file-drop-cancelled", () => {
			isDragHover = false;
		});
	});
</script>

<svelte:head>
	<title>Dataset Viewer</title>
	<meta name="description" content="A modern dataset viewer application" />
</svelte:head>

<div class="container mx-auto px-4 py-8">
	<div class="max-w-4xl mx-auto">
		<div class="text-center mb-12">
			<h1 class="text-4xl font-bold text-gray-900 mb-4">Dataset Viewer</h1>
			<p class="text-lg text-gray-600">View and analyze your media files with ease</p>
		</div>

		<div class="bg-white rounded-lg shadow-lg p-8 mb-16">
			<div
				class="border-2 border-dashed border-gray-300 rounded-lg p-12 text-center
					{isDragHover ? 'bg-indigo-50 border-indigo-300' : 'hover:border-gray-400'}"
			>
				<div class="space-y-4">
					<div class="text-gray-500">
						{#if isDragHover}
							<p class="text-lg">Drop your files here</p>
						{:else}
							<p class="text-lg">Drag and drop files here</p>
							<p class="text-sm mt-2">or use the navigation links below</p>
						{/if}
					</div>
				</div>
			</div>
		</div>

		<div class="mb-12">
			<h2 class="text-2xl font-semibold text-gray-800 mb-6 text-center">Available Tools</h2>
			<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
				<a href="/smart-tools" class="block p-6 bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow duration-200 no-underline">
					<h3 class="text-lg font-semibold text-indigo-600 mb-2">Smart Tools</h3>
					<p class="text-sm text-gray-600">Interactive image cropping and processing tools with drag-and-drop functionality.</p>
				</a>

				<a href="/dataset-gallery" class="block p-6 bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow duration-200 no-underline">
					<h3 class="text-lg font-semibold text-indigo-600 mb-2">Dataset Gallery</h3>
					<p class="text-sm text-gray-600">Main tool for viewing and managing datasets with annotations, YOLO export, and LabelMe extraction.</p>
				</a>

				<a
					href="/crop-remap"
					class="block p-6 bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow duration-200 no-underline"
				>
					<h3 class="text-lg font-semibold text-indigo-600 mb-2">Crop & Remap Tool</h3>
					<p class="text-sm text-gray-600">Advanced annotation processing with dynamic label detection and safety equipment analysis.</p>
				</a>

				<a href="/optimizedGallery" class="block p-6 bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow duration-200 no-underline">
					<h3 class="text-lg font-semibold text-indigo-600 mb-2">Optimized Gallery</h3>
					<p class="text-sm text-gray-600">View images in a performance-optimized gallery.</p>
				</a>

				<a href="/imageViewer"
					class="block p-6 bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow duration-200 no-underline"
				>
					<h3 class="text-lg font-semibold text-indigo-600 mb-2">Image Viewer</h3>
					<p class="text-sm text-gray-600">View individual images and basic info.</p>
				</a>

				<a href="/imageViewer3"
					class="block p-6 bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow duration-200 no-underline"
				>
					<h3 class="text-lg font-semibold text-indigo-600 mb-2">Image Viewer 3</h3>
					<p class="text-sm text-gray-600">Another image viewer implementation.</p>
				</a>
			</div>
		</div>

	</div>
</div>

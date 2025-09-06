<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { listen } from "@tauri-apps/api/event";

	let isDragHover: boolean = false;

	let unlistenDrop: (() => void) | null = null;
	let unlistenHover: (() => void) | null = null;
	let unlistenCancel: (() => void) | null = null;

	onMount(async () => {
		unlistenDrop = await listen("tauri://file-drop", async () => {
			isDragHover = false;
		});
		unlistenHover = await listen("tauri://file-drop-hover", async () => {
			isDragHover = true;
		});
		unlistenCancel = await listen("tauri://file-drop-cancelled", () => {
			isDragHover = false;
		});
	});

	onDestroy(() => {
		unlistenDrop?.();
		unlistenHover?.();
		unlistenCancel?.();
	});
</script>

<svelte:head>
	<title>Dataset Viewer</title>
	<meta name="description" content="A modern dataset viewer application" />
</svelte:head>

<div class="px-4 py-10">
	<div class="max-w-6xl mx-auto">
		<section class="text-center mb-12">
			<h1 class="text-5xl font-extrabold tracking-tight text-slate-800 mb-4">
				Work with datasets beautifully
			</h1>
			<p class="text-lg text-slate-600 max-w-2xl mx-auto">
				Analyze, visualize, and export your datasets with a refined, fast, and modern interface.
			</p>
		</section>

		<section class="relative mb-16">
			<div class="rounded-2xl border border-slate-200/60 bg-white/70 backdrop-blur p-10 shadow-sm">
				<div class={`rounded-xl border-2 border-dashed text-center py-16 transition-all duration-300 ${isDragHover ? 'bg-indigo-50/70 border-indigo-300' : 'hover:border-slate-300'}`}>
					{#if isDragHover}
						<p class="text-lg text-slate-700">Drop your files here</p>
					{:else}
						<p class="text-lg text-slate-700">Drag & drop files anywhere here</p>
						<p class="text-sm text-slate-500 mt-2">or explore tools below</p>
					{/if}
				</div>
			</div>
		</section>

		<section class="mb-4">
			<h2 class="text-2xl font-semibold text-slate-800 mb-6 text-center">Tools</h2>
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
				<a href="/smart-tools" class="group no-underline">
					<div class="h-full rounded-xl border border-slate-200/60 bg-white/70 backdrop-blur p-6 shadow-sm transition-all duration-200 group-hover:shadow-md group-hover:-translate-y-0.5">
						<h3 class="text-lg font-semibold text-indigo-600 mb-1">Smart Tools</h3>
						<p class="text-sm text-slate-600">Interactive cropping and processing with drag-and-drop.</p>
					</div>
				</a>
				<a href="/dataset-gallery" class="group no-underline">
					<div class="h-full rounded-xl border border-slate-200/60 bg-white/70 backdrop-blur p-6 shadow-sm transition-all duration-200 group-hover:shadow-md group-hover:-translate-y-0.5">
						<h3 class="text-lg font-semibold text-indigo-600 mb-1">Dataset Gallery</h3>
						<p class="text-sm text-slate-600">Browse, annotate, export YOLO, and extract LabelMe.</p>
					</div>
				</a>
				<a href="/crop-remap" class="group no-underline">
					<div class="h-full rounded-xl border border-slate-200/60 bg-white/70 backdrop-blur p-6 shadow-sm transition-all duration-200 group-hover:shadow-md group-hover:-translate-y-0.5">
						<h3 class="text-lg font-semibold text-indigo-600 mb-1">Crop & Remap Tool</h3>
						<p class="text-sm text-slate-600">Advanced annotation processing with smart label detection.</p>
					</div>
				</a>
				<a href="/optimizedGallery" class="group no-underline">
					<div class="h-full rounded-xl border border-slate-200/60 bg-white/70 backdrop-blur p-6 shadow-sm transition-all duration-200 group-hover:shadow-md group-hover:-translate-y-0.5">
						<h3 class="text-lg font-semibold text-indigo-600 mb-1">Optimized Gallery</h3>
						<p class="text-sm text-slate-600">Performance-focused gallery for large collections.</p>
					</div>
				</a>
				<a href="/imageViewer" class="group no-underline">
					<div class="h-full rounded-xl border border-slate-200/60 bg-white/70 backdrop-blur p-6 shadow-sm transition-all duration-200 group-hover:shadow-md group-hover:-translate-y-0.5">
						<h3 class="text-lg font-semibold text-indigo-600 mb-1">Image Viewer</h3>
						<p class="text-sm text-slate-600">View single images with metadata.</p>
					</div>
				</a>
				<a href="/imageViewer3" class="group no-underline">
					<div class="h-full rounded-xl border border-slate-200/60 bg-white/70 backdrop-blur p-6 shadow-sm transition-all duration-200 group-hover:shadow-md group-hover:-translate-y-0.5">
						<h3 class="text-lg font-semibold text-indigo-600 mb-1">Image Viewer 3</h3>
						<p class="text-sm text-slate-600">Alternate viewer implementation.</p>
					</div>
				</a>
			</div>
		</section>
	</div>
</div>

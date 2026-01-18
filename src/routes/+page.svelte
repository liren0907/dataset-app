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
	<title>Dataset App</title>
	<meta name="description" content="A modern dataset application" />
</svelte:head>

<div class="hero min-h-[calc(100vh-8rem)] bg-base-100">
	<div class="hero-content text-center flex-col w-full max-w-5xl px-4">
		<!-- Hero Section -->
		<div class="max-w-xl mb-6">
			<h1 class="text-4xl font-bold mb-2">
				Dataset <span class="text-primary">App</span>
			</h1>
			<p class="text-base opacity-60">
				Manage, visualize, and process your datasets with ease.
			</p>
		</div>

		<!-- Compact Drop Zone -->
		<div class="w-full max-w-xl mb-12">
			<div
				class="border-2 border-dashed rounded-xl p-6 text-center transition-all duration-300
					{isDragHover
					? 'border-primary bg-primary/10'
					: 'border-base-300 hover:border-primary/50'}"
			>
				{#if isDragHover}
					<span
						class="material-symbols-rounded text-4xl text-primary mb-2"
						>folder_open</span
					>
					<p class="text-lg font-medium text-primary">
						Drop files to import
					</p>
				{:else}
					<div class="flex flex-col items-center gap-2">
						<span
							class="material-symbols-rounded text-3xl opacity-40"
							>cloud_upload</span
						>
						<p class="text-base font-medium opacity-70">
							Drag & drop dataset files here
						</p>
					</div>
				{/if}
			</div>
		</div>

		<!-- Feature Navigation Cards -->
		<div class="grid grid-cols-1 md:grid-cols-3 gap-6 w-full text-left">
			<!-- Card 1: Gallery -->
			<a
				href="/gallery"
				class="card bg-base-200 hover:bg-base-300 transition-all duration-300 shadow-sm border border-transparent hover:border-primary/20 group"
			>
				<div class="card-body p-6">
					<div
						class="p-3 w-fit rounded-lg bg-primary/10 text-primary mb-2 group-hover:scale-110 transition-transform"
					>
						<span class="material-symbols-rounded text-2xl"
							>photo_library</span
						>
					</div>
					<h2 class="card-title text-xl">Unified Gallery</h2>
					<p class="text-sm opacity-70">
						Browse, organize, and inspect your image datasets in one
						place.
					</p>
				</div>
			</a>

			<!-- Card 2: Turbo Export -->
			<a
				href="/turbo-export"
				class="card bg-base-200 hover:bg-base-300 transition-all duration-300 shadow-sm border border-transparent hover:border-primary/20 group"
			>
				<div class="card-body p-6">
					<div
						class="p-3 w-fit rounded-lg bg-secondary/10 text-secondary mb-2 group-hover:scale-110 transition-transform"
					>
						<span class="material-symbols-rounded text-2xl"
							>rocket_launch</span
						>
					</div>
					<h2 class="card-title text-xl">Turbo Export</h2>
					<p class="text-sm opacity-70">
						High-speed export tools for generating models and
						archives.
					</p>
				</div>
			</a>

			<!-- Card 3: Smart Tools -->
			<a
				href="/smart-tools"
				class="card bg-base-200 hover:bg-base-300 transition-all duration-300 shadow-sm border border-transparent hover:border-primary/20 group"
			>
				<div class="card-body p-6">
					<div
						class="p-3 w-fit rounded-lg bg-accent/10 text-accent mb-2 group-hover:scale-110 transition-transform"
					>
						<span class="material-symbols-rounded text-2xl"
							>auto_awesome</span
						>
					</div>
					<h2 class="card-title text-xl">Smart Tools</h2>
					<p class="text-sm opacity-70">
						AI-assisted utilities for auto-labeling and data quality
						checks.
					</p>
				</div>
			</a>
		</div>
	</div>
</div>

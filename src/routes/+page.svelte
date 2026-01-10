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

<div class="hero min-h-[calc(100vh-8rem)]">
	<div class="hero-content text-center flex-col">
		<!-- Hero Section -->
		<div class="max-w-2xl mb-8">
			<h1 class="text-5xl font-bold mb-4">
				Work with datasets
				<span class="text-primary">beautifully</span>
			</h1>
			<p class="text-lg opacity-70">
				Analyze, visualize, and export your datasets with a refined,
				fast, and modern interface.
			</p>
		</div>

		<!-- Drop Zone Card -->
		<div class="card bg-base-200 shadow-xl w-full max-w-3xl">
			<div class="card-body">
				<div
					class="border-2 border-dashed rounded-xl p-16 text-center transition-all duration-300
						{isDragHover
						? 'border-primary bg-primary/10'
						: 'border-base-300 hover:border-primary/50'}"
				>
					{#if isDragHover}
						<span
							class="material-symbols-rounded text-6xl text-primary mb-4"
							>folder_open</span
						>
						<p class="text-xl font-medium">Drop your files here</p>
					{:else}
						<span
							class="material-symbols-rounded text-6xl opacity-50 mb-4"
							>folder</span
						>
						<p class="text-xl font-medium">
							Drag & drop files here
						</p>
						<p class="text-sm opacity-60 mt-2">
							或從左側選單選擇工具開始使用
						</p>
					{/if}
				</div>
			</div>
		</div>

		<!-- Tip Section -->
		<div class="mt-6">
			<div class="flex items-center gap-2 text-sm opacity-60">
				<span class="material-symbols-rounded icon-sm">lightbulb</span>
				<span>提示：使用左側選單</span>
				<kbd class="kbd kbd-sm">←</kbd>
				<span>快速切換工具</span>
			</div>
		</div>
	</div>
</div>

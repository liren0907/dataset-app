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

<div class="flex flex-col items-center justify-center min-h-[calc(100vh-8rem)]">
	<section class="text-center mb-12">
		<h1
			class="text-5xl font-extrabold tracking-tight text-slate-800 dark:text-slate-100 mb-4"
		>
			Work with datasets beautifully
		</h1>
		<p class="text-lg text-slate-600 dark:text-slate-400 max-w-2xl mx-auto">
			Analyze, visualize, and export your datasets with a refined, fast,
			and modern interface.
		</p>
	</section>

	<section class="relative w-full max-w-3xl">
		<div
			class="rounded-2xl border border-slate-200/60 dark:border-slate-700/60 bg-white/70 dark:bg-slate-800/70 backdrop-blur p-10 shadow-sm"
		>
			<div
				class={`rounded-xl border-2 border-dashed text-center py-20 transition-all duration-300 
				${
					isDragHover
						? "bg-indigo-50/70 dark:bg-indigo-900/30 border-indigo-300 dark:border-indigo-500"
						: "border-slate-300 dark:border-slate-600 hover:border-slate-400 dark:hover:border-slate-500"
				}`}
			>
				{#if isDragHover}
					<div class="text-6xl mb-4">📂</div>
					<p
						class="text-xl text-slate-700 dark:text-slate-200 font-medium"
					>
						Drop your files here
					</p>
				{:else}
					<div class="text-6xl mb-4 opacity-50">📁</div>
					<p
						class="text-xl text-slate-700 dark:text-slate-300 font-medium"
					>
						Drag & drop files here
					</p>
					<p class="text-sm text-slate-500 dark:text-slate-400 mt-2">
						或從左側選單選擇工具開始使用
					</p>
				{/if}
			</div>
		</div>
	</section>

	<section class="mt-8 text-center">
		<p class="text-sm text-slate-400 dark:text-slate-500">
			💡 提示：使用左側選單 <kbd
				class="px-2 py-1 bg-slate-200 dark:bg-slate-700 rounded text-xs"
				>←</kbd
			> 快速切換工具
		</p>
	</section>
</div>

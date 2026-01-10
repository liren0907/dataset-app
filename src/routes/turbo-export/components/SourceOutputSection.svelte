<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import {
		sourceDir,
		outputDir,
		customDatasetName,
		isDraggingOver,
		activeDropZone,
		defaultDatasetName
	} from '../stores/exportStore';

	// Event dispatcher 用於通知父組件
	const dispatch = createEventDispatcher<{
		sourceSelected: string;
		outputSelected: string;
	}>();

	// Drop zone 元素參照
	export let sourceDropZone: HTMLElement | null = null;
	export let outputDropZone: HTMLElement | null = null;

	// ===== 選擇資料夾 =====
	async function selectSourceDir() {
		const selected = await open({
			directory: true,
			multiple: false,
			title: '選擇 LabelMe JSON 資料夾'
		});
		if (selected && typeof selected === 'string') {
			sourceDir.set(selected);
			dispatch('sourceSelected', selected);
		}
	}

	async function selectOutputDir() {
		const selected = await open({
			directory: true,
			multiple: false,
			title: '選擇輸出路徑'
		});
		if (selected && typeof selected === 'string') {
			outputDir.set(selected);
			dispatch('outputSelected', selected);
		}
	}
</script>

<section class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-6 shadow-sm">
	<h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 mb-4 flex items-center gap-2">
		📁 來源與輸出
	</h2>

	<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
		<!-- 來源路徑 -->
		<div
			bind:this={sourceDropZone}
			class="relative group"
			role="button"
			tabindex="0"
		>
			<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">來源路徑</label>
			<div class="flex gap-2 transition-all duration-200 {$isDraggingOver && $activeDropZone !== 'source' ? 'opacity-50' : ''}">
				<input
					type="text"
					value={$sourceDir}
					placeholder="選擇或拖放包含 LabelMe JSON 的資料夾"
					class="flex-1 px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-slate-50 dark:bg-slate-700 dark:text-slate-100 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all duration-200
						{$activeDropZone === 'source' ? 'border-indigo-400 ring-2 ring-indigo-200' : ''}"
					readonly
				/>
				<button
					on:click={selectSourceDir}
					class="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors text-sm font-medium"
				>
					瀏覽
				</button>
			</div>
			<!-- 磨砂玻璃拖放覆蓋層 -->
			{#if $activeDropZone === 'source'}
				<div class="absolute inset-0 rounded-lg overflow-hidden z-10 animate-pulse-subtle">
					<div class="absolute inset-0 bg-gradient-to-br from-indigo-500/30 via-indigo-400/20 to-purple-500/30 backdrop-blur-md"></div>
					<div class="absolute inset-0 border-2 border-dashed border-indigo-400 rounded-lg"></div>
					<div class="absolute inset-0 flex items-center justify-center">
						<div class="bg-white/80 dark:bg-slate-800/80 backdrop-blur-sm px-4 py-2 rounded-full shadow-lg flex items-center gap-2">
							<svg class="w-5 h-5 text-indigo-600 animate-bounce" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
							</svg>
							<span class="text-indigo-700 dark:text-indigo-300 font-semibold text-sm">放開以設定來源</span>
						</div>
					</div>
				</div>
			{:else if $isDraggingOver && $activeDropZone !== 'output'}
				<!-- 拖動中但不在此區域 - 顯示提示邊框 -->
				<div class="absolute inset-0 rounded-lg border-2 border-dashed border-slate-300 dark:border-slate-600 z-10 pointer-events-none"></div>
			{/if}
		</div>

		<!-- 輸出路徑 -->
		<div
			bind:this={outputDropZone}
			class="relative group"
			role="button"
			tabindex="0"
		>
			<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">輸出路徑 (選填)</label>
			<div class="flex gap-2 transition-all duration-200 {$isDraggingOver && $activeDropZone !== 'output' ? 'opacity-50' : ''}">
				<input
					type="text"
					value={$outputDir}
					placeholder="預設為來源路徑"
					class="flex-1 px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-slate-50 dark:bg-slate-700 dark:text-slate-100 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all duration-200
						{$activeDropZone === 'output' ? 'border-emerald-400 ring-2 ring-emerald-200' : ''}"
					readonly
				/>
				<button
					on:click={selectOutputDir}
					class="px-4 py-2 bg-slate-600 text-white rounded-lg hover:bg-slate-700 transition-colors text-sm font-medium"
				>
					瀏覽
				</button>
			</div>
			<!-- 磨砂玻璃拖放覆蓋層 -->
			{#if $activeDropZone === 'output'}
				<div class="absolute inset-0 rounded-lg overflow-hidden z-10 animate-pulse-subtle">
					<div class="absolute inset-0 bg-gradient-to-br from-emerald-500/30 via-emerald-400/20 to-teal-500/30 backdrop-blur-md"></div>
					<div class="absolute inset-0 border-2 border-dashed border-emerald-400 rounded-lg"></div>
					<div class="absolute inset-0 flex items-center justify-center">
						<div class="bg-white/80 dark:bg-slate-800/80 backdrop-blur-sm px-4 py-2 rounded-full shadow-lg flex items-center gap-2">
							<svg class="w-5 h-5 text-emerald-600 animate-bounce" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
							</svg>
							<span class="text-emerald-700 dark:text-emerald-300 font-semibold text-sm">放開以設定輸出</span>
						</div>
					</div>
				</div>
			{:else if $isDraggingOver && $activeDropZone !== 'source'}
				<!-- 拖動中但不在此區域 - 顯示提示邊框 -->
				<div class="absolute inset-0 rounded-lg border-2 border-dashed border-slate-300 dark:border-slate-600 z-10 pointer-events-none"></div>
			{/if}
		</div>
	</div>

	<!-- 自訂資料夾名稱 -->
	<div class="mt-4">
		<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">
			輸出資料夾名稱 (選填)
		</label>
		<input
			type="text"
			value={$customDatasetName}
			on:input={(e) => customDatasetName.set(e.currentTarget.value)}
			placeholder="留空則自動產生"
			class="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-slate-50 dark:bg-slate-700 dark:text-slate-100 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all duration-200"
		/>
		<div class="mt-2 text-xs text-slate-500 dark:text-slate-400">
			<span class="font-medium">預覽：</span>
			<code class="bg-slate-100 dark:bg-slate-600 px-2 py-0.5 rounded text-indigo-600 dark:text-indigo-300">
				{$customDatasetName || $defaultDatasetName || '請先選擇來源路徑'}
			</code>
		</div>
	</div>
</section>

<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { dndzone } from 'svelte-dnd-action';
	import { flip } from 'svelte/animate';
	import {
		sourceDir,
		useCustomLabels,
		labelList,
		isScanning,
		labelScanMessage,
		isCalculatingCounts,
		toggleLabel,
		selectAllLabels,
		deselectAllLabels,
		type LabelInfo
	} from '../stores/exportStore';

	const dispatch = createEventDispatcher<{
		rescan: void;
	}>();

	// 拖拉動畫時間
	const flipDurationMs = 200;

	// ===== svelte-dnd-action 事件處理 =====
	function handleDndConsider(e: CustomEvent<{ items: LabelInfo[] }>) {
		labelList.set(e.detail.items);
	}

	function handleDndFinalize(e: CustomEvent<{ items: LabelInfo[] }>) {
		labelList.set(e.detail.items);
	}

	// 重新掃描
	function handleRescan() {
		dispatch('rescan');
	}
</script>

<section class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-6 shadow-sm relative">
	<!-- 背景計算 Spinner -->
	{#if $isCalculatingCounts}
		<div class="absolute top-4 right-4 flex items-center gap-2 text-indigo-600 dark:text-indigo-400">
			<svg class="animate-spin h-4 w-4" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
				<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
			</svg>
			<span class="text-xs font-medium">計算數量中...</span>
		</div>
	{/if}
	<div class="flex items-center justify-between mb-4">
		<h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 flex items-center gap-2">
			🏷️ 標籤選擇
		</h2>
		{#if $labelScanMessage}
			<span class="text-sm text-emerald-600 dark:text-emerald-400 font-medium">{$labelScanMessage}</span>
		{/if}
	</div>

	<!-- 切換開關 -->
	<div class="flex items-center gap-3 mb-4">
		<label class="relative inline-flex items-center cursor-pointer">
			<input type="checkbox" checked={$useCustomLabels} on:change={() => useCustomLabels.update(v => !v)} class="sr-only peer" />
			<div class="w-11 h-6 bg-slate-200 dark:bg-slate-600 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 dark:peer-focus:ring-indigo-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600"></div>
		</label>
		<span class="text-sm text-slate-700 dark:text-slate-300">只匯出選定的類別</span>
	</div>

	{#if $useCustomLabels}
		<div class="space-y-3">
			<!-- 快速操作 -->
			<div class="flex gap-2 flex-wrap">
				<button
					on:click={selectAllLabels}
					class="px-3 py-1 text-xs bg-slate-100 dark:bg-slate-700 hover:bg-slate-200 dark:hover:bg-slate-600 text-slate-700 dark:text-slate-300 rounded-md transition-colors"
				>
					全選
				</button>
				<button
					on:click={deselectAllLabels}
					class="px-3 py-1 text-xs bg-slate-100 dark:bg-slate-700 hover:bg-slate-200 dark:hover:bg-slate-600 text-slate-700 dark:text-slate-300 rounded-md transition-colors"
				>
					全不選
				</button>
				<button
					on:click={handleRescan}
					disabled={$isScanning || !$sourceDir}
					class="px-3 py-1 text-xs bg-indigo-100 dark:bg-indigo-900/50 hover:bg-indigo-200 dark:hover:bg-indigo-800/50 text-indigo-700 dark:text-indigo-300 rounded-md transition-colors disabled:opacity-50"
				>
					{$isScanning ? '掃描中...' : '重新掃描'}
				</button>
				<span class="ml-auto text-xs text-slate-500 dark:text-slate-400">
					已選 {$labelList.filter(l => l.selected).length} / {$labelList.length}
				</span>
			</div>

			<!-- 可拖拉排序的標籤表格 -->
			{#if $labelList.length > 0}
				<div class="border border-slate-200 dark:border-slate-700 rounded-lg overflow-hidden">
					<!-- 表頭 -->
					<div class="grid grid-cols-[50px_1fr_80px_50px] gap-2 px-3 py-2 bg-slate-50 dark:bg-slate-700/50 border-b border-slate-200 dark:border-slate-700 text-xs font-medium text-slate-600 dark:text-slate-400">
						<span class="text-center">ID</span>
						<span>標籤名稱</span>
						<span class="text-right">數量</span>
						<span class="text-center">選取</span>
					</div>
					<!-- 拖拉提示 -->
					<div class="px-3 py-1.5 bg-amber-50 dark:bg-amber-900/30 border-b border-amber-100 dark:border-amber-800/50 text-xs text-amber-700 dark:text-amber-300 flex items-center gap-1">
						<span>💡</span>
						<span>直接拖拉標籤列調整順序 = 調整輸出的 class ID</span>
					</div>
					<!-- 標籤列表（可拖拉排序）-->
					<div
						class="divide-y divide-slate-100 dark:divide-slate-700"
						use:dndzone="{{ items: $labelList, flipDurationMs, dropTargetStyle: {} }}"
						on:consider={handleDndConsider}
						on:finalize={handleDndFinalize}
					>
						{#each $labelList as label, index (label.id)}
							<div
								animate:flip="{{ duration: flipDurationMs }}"
								class="grid grid-cols-[50px_1fr_80px_50px] gap-2 px-3 py-2 items-center bg-white dark:bg-slate-800 cursor-grab active:cursor-grabbing hover:bg-slate-50 dark:hover:bg-slate-700/50
									{label.selected ? '' : 'text-slate-400 dark:text-slate-500 bg-slate-50 dark:bg-slate-800/50'}"
							>
								<!-- ID -->
								<span class="text-center font-mono text-sm {label.selected ? 'text-indigo-600 dark:text-indigo-400 font-bold' : ''}">
									{label.selected ? $labelList.slice(0, index + 1).filter(l => l.selected).length - 1 : '-'}
								</span>
								<!-- 拖拉手把 + 標籤名稱 -->
								<div class="flex items-center gap-2 select-none">
									<span class="text-slate-400 dark:text-slate-500">⋮⋮</span>
									<span class="text-sm font-medium text-slate-800 dark:text-slate-200">{label.name}</span>
								</div>
								<!-- 數量 -->
								<span class="text-right text-sm text-slate-500 dark:text-slate-400">
									{#if $isCalculatingCounts && label.count === 0}
										<svg class="animate-spin h-4 w-4 inline text-slate-400" viewBox="0 0 24 24">
											<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
											<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
										</svg>
									{:else}
										{label.count.toLocaleString()}
									{/if}
								</span>
								<!-- 選取 checkbox -->
								<div class="flex justify-center" on:mousedown|stopPropagation on:touchstart|stopPropagation>
									<input
										type="checkbox"
										checked={label.selected}
										on:change={() => toggleLabel(label.id)}
										class="w-4 h-4 text-indigo-600 rounded focus:ring-indigo-500 cursor-pointer"
									/>
								</div>
							</div>
						{/each}
					</div>
				</div>
			{:else}
				<div class="text-center py-8 text-slate-500 dark:text-slate-400">
					{#if !$sourceDir}
						請先選擇來源路徑以掃描可用標籤
					{:else if $isScanning}
						<div class="flex items-center justify-center gap-2">
							<svg class="animate-spin h-5 w-5" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
							正在掃描標籤...
						</div>
					{:else}
						點擊「重新掃描」以取得標籤列表
					{/if}
				</div>
			{/if}
		</div>
	{:else}
		<p class="text-sm text-slate-500 dark:text-slate-400">
			將自動匯出所有標籤，ID 按照首次出現順序分配
		</p>
	{/if}
</section>

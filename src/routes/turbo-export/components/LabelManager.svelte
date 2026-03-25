<script lang="ts">
	import { dndzone } from "svelte-dnd-action";
	import { flip } from "svelte/animate";
	import {
		paths,
		labels,
		detection,
		toggleLabel,
		selectAllLabels,
		deselectAllLabels,
		type LabelInfo,
	} from "../stores/exportStore.svelte";

	let { onrescan }: {
		onrescan?: () => void;
	} = $props();

	// 拖拉動畫時間
	const flipDurationMs = 200;

	// ===== svelte-dnd-action 事件處理 =====
	function handleDndConsider(e: CustomEvent<{ items: LabelInfo[] }>) {
		labels.labelList = e.detail.items;
	}

	function handleDndFinalize(e: CustomEvent<{ items: LabelInfo[] }>) {
		labels.labelList = e.detail.items;
	}

	// 重新掃描
	function handleRescan() {
		onrescan?.();
	}
</script>

<section
	class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-6 shadow-sm relative"
>
	<!-- 背景計算 Spinner -->
	{#if labels.isCalculatingCounts}
		<div
			class="absolute top-4 right-4 flex items-center gap-2 text-indigo-600 dark:text-indigo-400"
		>
			<svg class="animate-spin h-4 w-4" viewBox="0 0 24 24">
				<circle
					class="opacity-25"
					cx="12"
					cy="12"
					r="10"
					stroke="currentColor"
					stroke-width="4"
					fill="none"
				></circle>
				<path
					class="opacity-75"
					fill="currentColor"
					d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
				></path>
			</svg>
			<span class="text-xs font-medium">計算數量中...</span>
		</div>
	{/if}
	<div class="flex items-center justify-between mb-4">
		<h2
			class="text-lg font-semibold text-slate-800 dark:text-slate-100 flex items-center gap-2"
		>
			🏷️ 標籤選擇
		</h2>
		<div class="flex items-center gap-2">
			{#if labels.labelScanMessage}
				<span
					class="text-sm text-emerald-600 dark:text-emerald-400 font-medium"
					>{labels.labelScanMessage}</span
				>
			{/if}
			<button
				onclick={handleRescan}
				disabled={labels.isScanning || !paths.sourceDir}
				class="px-3 py-1.5 text-xs bg-indigo-100 dark:bg-indigo-900/50 hover:bg-indigo-200 dark:hover:bg-indigo-800/50 text-indigo-700 dark:text-indigo-300 rounded-md transition-colors disabled:opacity-50"
				title="重新掃描標籤列表"
			>
				{labels.isScanning ? "掃描中..." : "🔄 重新掃描"}
			</button>
		</div>
	</div>

	<!-- 🆕 格式檢測結果顯示 -->
	{#if detection.detectedFormat}
		<div
			class="mb-4 p-3 bg-blue-50 dark:bg-blue-900/30 rounded-lg border border-blue-200 dark:border-blue-700"
		>
			<div class="flex items-center gap-2 text-sm">
				<span class="text-blue-600 dark:text-blue-400">📊</span>
				<span class="font-medium text-blue-700 dark:text-blue-300"
					>檢測到的格式：</span
				>
				<span class="text-blue-600 dark:text-blue-400"
					>{detection.detectedFormat.format_description}</span
				>
				<span class="ml-auto text-xs text-blue-500 dark:text-blue-400">
					信心度：{detection.detectedFormat.confidence_percent}
				</span>
			</div>
		</div>
	{:else if detection.isDetectingFormat}
		<div
			class="mb-4 p-3 bg-slate-50 dark:bg-slate-700/50 rounded-lg border border-slate-200 dark:border-slate-600"
		>
			<div
				class="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-400"
			>
				<svg class="animate-spin h-4 w-4" viewBox="0 0 24 24">
					<circle
						class="opacity-25"
						cx="12"
						cy="12"
						r="10"
						stroke="currentColor"
						stroke-width="4"
						fill="none"
					></circle>
					<path
						class="opacity-75"
						fill="currentColor"
						d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
					></path>
				</svg>
				<span>正在檢測資料集格式...</span>
			</div>
		</div>
	{/if}

	<!-- 切換開關 -->
	<div class="flex items-center gap-3 mb-4 flex-wrap">
		<div class="flex items-center gap-3">
			<label class="relative inline-flex items-center cursor-pointer">
				<input
					type="checkbox"
					checked={labels.useCustomLabels}
					onchange={() => labels.useCustomLabels = !labels.useCustomLabels}
					class="sr-only peer"
				/>
				<div
					class="w-11 h-6 bg-slate-200 dark:bg-slate-600 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 dark:peer-focus:ring-indigo-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600"
				></div>
			</label>
			<span class="text-sm text-slate-700 dark:text-slate-300"
				>只匯出選定的類別</span
			>
		</div>

		<!-- 輸出無標籤圖片（僅在啟用自訂標籤時顯示） -->
		{#if labels.useCustomLabels}
			<div
				class="flex items-center gap-2 pl-3 border-l border-slate-200 dark:border-slate-600"
			>
				<label
					class="flex items-center gap-2 cursor-pointer"
					title="篩選後若圖片不含任何選定標籤，是否仍輸出該圖片（空標籤檔）"
				>
					<input
						type="checkbox"
						checked={labels.includeEmptyLabelImages}
						onchange={() =>
							labels.includeEmptyLabelImages = !labels.includeEmptyLabelImages}
						class="w-4 h-4 text-indigo-600 rounded focus:ring-indigo-500"
					/>
					<span class="text-sm text-slate-600 dark:text-slate-400"
						>輸出無標籤圖片</span
					>
				</label>
			</div>
		{/if}

		{#if labels.useCustomLabels && labels.labelList.length > 0}
			<span
				class="text-xs text-slate-500 dark:text-slate-400 ml-auto pr-3"
			>
				已選 {labels.labelList.filter((l) => l.selected).length} / {labels.labelList.length}
			</span>
		{/if}
	</div>

	{#if labels.useCustomLabels}
		<div class="space-y-2">
			<!-- 可拖拉排序的標籤表格 -->
			{#if labels.labelList.length > 0}
				<div
					class="border border-slate-200 dark:border-slate-700 rounded-lg overflow-hidden"
				>
					<!-- 表頭 -->
					<div
						class="grid grid-cols-[50px_1fr_80px_50px] gap-2 px-3 py-2 bg-slate-50 dark:bg-slate-700/50 border-b border-slate-200 dark:border-slate-700 text-xs font-medium text-slate-600 dark:text-slate-400"
					>
						<span class="text-center">ID</span>
						<span>標籤名稱</span>
						<span class="text-right">數量</span>
						<div class="flex items-center justify-center gap-1.5">
							<input
								type="checkbox"
								checked={labels.labelList.every((l) => l.selected)}
								indeterminate={labels.labelList.some(
									(l) => l.selected,
								) && !labels.labelList.every((l) => l.selected)}
								onchange={(e) => {
									if (e.currentTarget.checked)
										selectAllLabels();
									else deselectAllLabels();
								}}
								class="w-4 h-4 text-indigo-600 rounded focus:ring-indigo-500 cursor-pointer"
								title="全選/取消全選"
							/>
							<span class="text-xs" title="全選/取消全選"></span>
						</div>
					</div>
					<!-- 拖拉提示 -->
					<div
						class="px-3 py-1.5 bg-amber-50 dark:bg-amber-900/30 border-b border-amber-100 dark:border-amber-800/50 text-xs text-amber-700 dark:text-amber-300 flex items-center gap-1"
					>
						<span>💡</span>
						<span>直接拖拉標籤列調整順序 = 調整輸出的 class ID</span
						>
					</div>
					<!-- 標籤列表（可拖拉排序）-->
					<div
						class="divide-y divide-slate-100 dark:divide-slate-700"
						use:dndzone={{
							items: labels.labelList,
							flipDurationMs,
							dropTargetStyle: {},
						}}
						onconsider={handleDndConsider}
						onfinalize={handleDndFinalize}
					>
						{#each labels.labelList as label, index (label.id)}
							<div
								animate:flip={{ duration: flipDurationMs }}
								class="grid grid-cols-[50px_1fr_80px_50px] gap-2 px-3 py-2 items-center bg-white dark:bg-slate-800 cursor-grab active:cursor-grabbing hover:bg-slate-50 dark:hover:bg-slate-700/50
									{label.selected
									? ''
									: 'text-slate-400 dark:text-slate-500 bg-slate-50 dark:bg-slate-800/50'}"
							>
								<!-- ID -->
								<span
									class="text-center font-mono text-sm {label.selected
										? 'text-indigo-600 dark:text-indigo-400 font-bold'
										: ''}"
								>
									{label.selected
										? labels.labelList
												.slice(0, index + 1)
												.filter((l) => l.selected)
												.length - 1
										: "-"}
								</span>
								<!-- 拖拉手把 + 標籤名稱 -->
								<div
									class="flex items-center gap-2 select-none"
								>
									<span
										class="text-slate-400 dark:text-slate-500"
										>⋮⋮</span
									>
									<span
										class="text-sm font-medium text-slate-800 dark:text-slate-200"
										>{label.name}</span
									>
								</div>
								<!-- 數量 -->
								<span
									class="text-right text-sm text-slate-500 dark:text-slate-400"
								>
									{#if labels.isCalculatingCounts && label.count === 0}
										<svg
											class="animate-spin h-4 w-4 inline text-slate-400"
											viewBox="0 0 24 24"
										>
											<circle
												class="opacity-25"
												cx="12"
												cy="12"
												r="10"
												stroke="currentColor"
												stroke-width="4"
												fill="none"
											></circle>
											<path
												class="opacity-75"
												fill="currentColor"
												d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
											></path>
										</svg>
									{:else}
										{label.count.toLocaleString()}
									{/if}
								</span>
								<!-- 選取 checkbox -->
								<div
									class="flex justify-center"
									onmousedown={(e) => { e.stopPropagation(); }}
									ontouchstart={(e) => { e.stopPropagation(); }}
								>
									<input
										type="checkbox"
										checked={label.selected}
										onchange={() => toggleLabel(label.id)}
										class="w-4 h-4 text-indigo-600 rounded focus:ring-indigo-500 cursor-pointer"
									/>
								</div>
							</div>
						{/each}
					</div>
				</div>
			{:else}
				<div
					class="text-center py-8 text-slate-500 dark:text-slate-400"
				>
					{#if !paths.sourceDir}
						請先選擇來源路徑以掃描可用標籤
					{:else if labels.isScanning}
						<div class="flex items-center justify-center gap-2">
							<svg
								class="animate-spin h-5 w-5"
								viewBox="0 0 24 24"
							>
								<circle
									class="opacity-25"
									cx="12"
									cy="12"
									r="10"
									stroke="currentColor"
									stroke-width="4"
									fill="none"
								></circle>
								<path
									class="opacity-75"
									fill="currentColor"
									d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
								></path>
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

<script lang="ts">
	import {
		stats,
		detailedStats,
		showInvalidDetails,
		invalidReasonGroups
	} from '../stores/exportStore';
	import { writable } from 'svelte/store';

	// 本地狀態：是否展開背景圖片列表
	const showBackgroundFiles = writable(false);
</script>

<!-- 詳細統計（轉換完成後顯示） -->
<div class="mt-4 pt-4 border-t border-slate-200 dark:border-slate-700">
	<!-- 數據摘要 -->
	<div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-center text-sm">
		<div>
			<div class="text-2xl font-bold text-slate-800 dark:text-slate-100">{$stats.processed}</div>
			<div class="text-slate-500 dark:text-slate-400">檔案處理</div>
		</div>
		<div>
			<div class="text-2xl font-bold text-emerald-600 dark:text-emerald-400">
				{$detailedStats.totalAnnotations.toLocaleString()}
			</div>
			<div class="text-slate-500 dark:text-slate-400">標註匯出</div>
		</div>
		<div>
			<div class="text-2xl font-bold text-amber-600 dark:text-amber-400">
				{$detailedStats.skippedAnnotations.toLocaleString()}
			</div>
			<div class="text-slate-500 dark:text-slate-400">標註跳過</div>
		</div>
		<div>
			<div class="text-2xl font-bold text-blue-600 dark:text-blue-400">
				{$detailedStats.backgroundImages}
			</div>
			<div class="text-slate-500 dark:text-slate-400">背景圖片</div>
		</div>
	</div>

	<!-- 跳過的標籤提示 -->
	{#if $detailedStats.skippedLabels.length > 0}
		<div class="mt-4 p-3 bg-amber-50 dark:bg-amber-900/30 rounded-lg border border-amber-200 dark:border-amber-800">
			<div class="flex items-start gap-2">
				<span class="text-amber-500">⚠️</span>
				<div class="text-sm">
					<div class="font-medium text-amber-700 dark:text-amber-300">
						以下標籤未被匯出（不在選擇列表中）：
					</div>
					<div class="text-amber-600 dark:text-amber-400 mt-1">
						{$detailedStats.skippedLabels.join(', ')}
					</div>
				</div>
			</div>
		</div>
	{/if}

	<!-- 無效標註詳情（點擊展開） -->
	{#if $detailedStats.invalidAnnotations.length > 0}
		<div class="mt-4 p-3 bg-orange-50 dark:bg-orange-900/30 rounded-lg border border-orange-200 dark:border-orange-800">
			<button
				on:click={() => showInvalidDetails.update(v => !v)}
				class="w-full flex items-center justify-between text-left"
			>
				<div class="flex items-start gap-2">
					<span class="text-orange-500">🔍</span>
					<div class="text-sm">
						<div class="font-medium text-orange-700 dark:text-orange-300">
							發現 {$detailedStats.invalidAnnotations.length} 個無效標註
							{#if $detailedStats.invalidAnnotations.length >= 100}
								<span class="text-orange-500">（僅顯示前 100 筆）</span>
							{/if}
						</div>
						<div class="text-orange-600 dark:text-orange-400 mt-0.5">
							點擊查看詳情，了解為何這些標註無法轉換
						</div>
					</div>
				</div>
				<span class="text-orange-500 transition-transform {$showInvalidDetails ? 'rotate-180' : ''}">
					▼
				</span>
			</button>

			{#if $showInvalidDetails}
				<div class="mt-3 pt-3 border-t border-orange-200 dark:border-orange-700">
					<!-- 按原因分組統計 -->
					<div class="space-y-3">
						{#each Object.entries($invalidReasonGroups) as [reason, items]}
							<div class="bg-white dark:bg-slate-800 rounded-lg p-3 border border-orange-100 dark:border-orange-800/50">
								<div class="flex items-center justify-between mb-2">
									<span class="font-medium text-orange-700 dark:text-orange-300 text-sm">
										{reason}
									</span>
									<span class="text-xs bg-orange-100 dark:bg-orange-800 text-orange-700 dark:text-orange-300 px-2 py-0.5 rounded-full">
										{items.length} 個
									</span>
								</div>
								<div class="max-h-32 overflow-y-auto text-xs space-y-1">
									{#each items.slice(0, 20) as item}
										<div class="flex items-center gap-2 text-slate-600 dark:text-slate-400 bg-slate-50 dark:bg-slate-700/50 rounded px-2 py-1">
											<span class="text-slate-400 dark:text-slate-500">📄</span>
											<span class="font-mono truncate flex-1" title={item.file}>{item.file}</span>
											<span class="text-orange-600 dark:text-orange-400 whitespace-nowrap">
												{item.label}
											</span>
											<span class="text-slate-400 dark:text-slate-500 whitespace-nowrap">
												({item.shape_type}, {item.points_count}點)
											</span>
										</div>
									{/each}
									{#if items.length > 20}
										<div class="text-center text-slate-400 dark:text-slate-500 py-1">
											...還有 {items.length - 20} 個
										</div>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	{/if}

	<!-- 背景圖片清單（點擊展開） -->
	{#if $detailedStats.backgroundImages > 0}
		<div class="mt-4 p-3 bg-blue-50 dark:bg-blue-900/30 rounded-lg border border-blue-200 dark:border-blue-800">
			<button
				on:click={() => showBackgroundFiles.update(v => !v)}
				class="w-full flex items-center justify-between text-left"
			>
				<div class="flex items-start gap-2">
					<span class="text-blue-500">🖼️</span>
					<div class="text-sm">
						<div class="font-medium text-blue-700 dark:text-blue-300">
							{$detailedStats.backgroundImages} 張背景圖片（無標註）
							{#if $detailedStats.backgroundFiles.length >= 100}
								<span class="text-blue-500">（僅顯示前 100 筆）</span>
							{/if}
						</div>
						<div class="text-blue-600 dark:text-blue-400 mt-0.5">
							點擊查看這些圖片的檔名
						</div>
					</div>
				</div>
				<span class="text-blue-500 transition-transform {$showBackgroundFiles ? 'rotate-180' : ''}">
					▼
				</span>
			</button>

			{#if $showBackgroundFiles}
				<div class="mt-3 pt-3 border-t border-blue-200 dark:border-blue-700">
					<div class="max-h-40 overflow-y-auto text-xs space-y-1">
						{#each $detailedStats.backgroundFiles.slice(0, 100) as fileName}
							<div class="flex items-center gap-2 text-slate-600 dark:text-slate-400 bg-slate-50 dark:bg-slate-700/50 rounded px-2 py-1">
								<span class="text-slate-400 dark:text-slate-500">🖼️</span>
								<span class="font-mono truncate flex-1" title={fileName}>{fileName}</span>
							</div>
						{/each}
						{#if $detailedStats.backgroundImages > 100}
							<div class="text-center text-slate-400 dark:text-slate-500 py-1">
								...還有 {$detailedStats.backgroundImages - 100} 個
							</div>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	{/if}

	<!-- 失敗提示 -->
	{#if $stats.failed > 0}
		<div class="mt-4 p-3 bg-rose-50 dark:bg-rose-900/30 rounded-lg border border-rose-200 dark:border-rose-800">
			<div class="flex items-start gap-2">
				<span class="text-rose-500">❌</span>
				<div class="text-sm text-rose-600 dark:text-rose-400">
					有 {$stats.failed} 個檔案處理失敗，請檢查來源檔案是否完整。
				</div>
			</div>
		</div>
	{/if}
</div>

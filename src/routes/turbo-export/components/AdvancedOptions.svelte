<script lang="ts">
	import {
		showAdvanced,
		workerCount,
		randomSeed,
		removeImageData,
		outputTarget
	} from '../stores/exportStore';

	// 是否為 LabelMe 輸出格式
	$: isLabelMeOutput = $outputTarget === 'labelme';
</script>

<section class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 shadow-sm overflow-hidden">
	<button
		on:click={() => showAdvanced.update(v => !v)}
		class="w-full px-6 py-4 flex items-center justify-between text-left hover:bg-slate-50 dark:hover:bg-slate-700/50 transition-colors"
	>
		<h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 flex items-center gap-2">
			⚙️ 進階選項
		</h2>
		<svg
			class="w-5 h-5 text-slate-400 transition-transform {$showAdvanced ? 'rotate-180' : ''}"
			fill="none"
			stroke="currentColor"
			viewBox="0 0 24 24"
		>
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
		</svg>
	</button>

	{#if $showAdvanced}
		<div class="px-6 pb-6 space-y-4 border-t border-slate-100 dark:border-slate-700 pt-4">
			<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
				<!-- Worker 數量 -->
				<div>
					<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">
						Worker 線程數
					</label>
					<input
						type="number"
						value={$workerCount}
						on:input={(e) => workerCount.set(parseInt(e.currentTarget.value) || 0)}
						min="0"
						max="32"
						class="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-white dark:bg-slate-700 text-slate-800 dark:text-slate-100 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
					/>
					<p class="text-xs text-slate-500 dark:text-slate-400 mt-1">0 = 自動 (CPU 核心數)</p>
				</div>

				<!-- 隨機種子（非 LabelMe 格式時顯示）-->
				{#if !isLabelMeOutput}
					<div>
						<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">
							隨機種子
						</label>
						<input
							type="number"
							value={$randomSeed}
							on:input={(e) => randomSeed.set(parseInt(e.currentTarget.value) || 0)}
							min="0"
							class="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-white dark:bg-slate-700 text-slate-800 dark:text-slate-100 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
						/>
						<p class="text-xs text-slate-500 dark:text-slate-400 mt-1">用於資料集分割的隨機性</p>
					</div>
				{/if}
			</div>

			<!-- 移除 base64 圖片資料（LabelMe 格式專用）-->
			{#if isLabelMeOutput}
				<label class="flex items-center gap-3 cursor-pointer">
					<input
						type="checkbox"
						checked={$removeImageData}
						on:change={() => removeImageData.update(v => !v)}
						class="w-4 h-4 text-indigo-600 rounded focus:ring-indigo-500"
					/>
					<div>
						<div class="text-sm font-medium text-slate-700 dark:text-slate-300">移除內嵌圖片資料</div>
						<div class="text-xs text-slate-500 dark:text-slate-400">將 imageData 設為 null，大幅減少 JSON 檔案大小</div>
					</div>
				</label>
			{/if}
		</div>
	{/if}
</section>

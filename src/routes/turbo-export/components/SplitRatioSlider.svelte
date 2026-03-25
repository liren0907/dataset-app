<script lang="ts">
	import {
		split,
		adjustSplitRatios
	} from '../stores/exportStore.svelte';

	// 處理滑桿輸入
	function handleRangeInput(name: 'train' | 'val' | 'test', value: number) {
		if (name === 'train') split.trainRatio = value;
		else if (name === 'val') split.valRatio = value;
		else split.testRatio = value;
		adjustSplitRatios(name);
	}
</script>

<section class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-6 shadow-sm">
	<h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 mb-4 flex items-center gap-2">
		📊 資料集分割
	</h2>

	<div class="space-y-4">
		<!-- Train -->
		<div class="flex items-center gap-4">
			<label class="w-20 text-sm font-medium text-slate-700 dark:text-slate-300">Train</label>
			<input
				type="range"
				value={split.trainRatio}
				oninput={(e) => handleRangeInput('train', parseInt(e.currentTarget.value))}
				min="0"
				max="100"
				class="flex-1 h-2 bg-slate-200 dark:bg-slate-600 rounded-lg appearance-none cursor-pointer accent-blue-600"
			/>
			<input
				type="number"
				value={split.trainRatio}
				onchange={(e) => handleRangeInput('train', parseInt(e.currentTarget.value))}
				min="0"
				max="100"
				class="w-20 px-3 py-1.5 text-right text-sm font-mono border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-700 text-slate-800 dark:text-slate-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
			/>
			<span class="text-sm text-slate-500 dark:text-slate-400">%</span>
		</div>

		<!-- Val -->
		<div class="flex items-center gap-4">
			<label class="w-20 text-sm font-medium text-slate-700 dark:text-slate-300">Val</label>
			<input
				type="range"
				value={split.valRatio}
				oninput={(e) => handleRangeInput('val', parseInt(e.currentTarget.value))}
				min="0"
				max="100"
				class="flex-1 h-2 bg-slate-200 dark:bg-slate-600 rounded-lg appearance-none cursor-pointer accent-amber-500"
			/>
			<input
				type="number"
				value={split.valRatio}
				onchange={(e) => handleRangeInput('val', parseInt(e.currentTarget.value))}
				min="0"
				max="100"
				class="w-20 px-3 py-1.5 text-right text-sm font-mono border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-700 text-slate-800 dark:text-slate-100 focus:ring-2 focus:ring-amber-500 focus:border-amber-500"
			/>
			<span class="text-sm text-slate-500 dark:text-slate-400">%</span>
		</div>

		<!-- Test -->
		<div class="flex items-center gap-4">
			<label class="w-20 text-sm font-medium text-slate-700 dark:text-slate-300">Test</label>
			<input
				type="range"
				value={split.testRatio}
				oninput={(e) => handleRangeInput('test', parseInt(e.currentTarget.value))}
				min="0"
				max="100"
				class="flex-1 h-2 bg-slate-200 dark:bg-slate-600 rounded-lg appearance-none cursor-pointer accent-rose-500"
			/>
			<input
				type="number"
				value={split.testRatio}
				onchange={(e) => handleRangeInput('test', parseInt(e.currentTarget.value))}
				min="0"
				max="100"
				class="w-20 px-3 py-1.5 text-right text-sm font-mono border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-700 text-slate-800 dark:text-slate-100 focus:ring-2 focus:ring-rose-500 focus:border-rose-500"
			/>
			<span class="text-sm text-slate-500 dark:text-slate-400">%</span>
		</div>

		<!-- 視覺化比例條 -->
		<div class="flex h-3 rounded-full overflow-hidden mt-2">
			<div class="bg-blue-500" style="width: {split.trainRatio}%"></div>
			<div class="bg-amber-500" style="width: {split.valRatio}%"></div>
			<div class="bg-rose-500" style="width: {split.testRatio}%"></div>
		</div>
		<div class="flex text-xs text-slate-500 dark:text-slate-400">
			<span class="flex-1">🔵 Train {split.trainRatio}%</span>
			<span class="flex-1 text-center">🟡 Val {split.valRatio}%</span>
			<span class="flex-1 text-right">🔴 Test {split.testRatio}%</span>
		</div>
	</div>
</section>

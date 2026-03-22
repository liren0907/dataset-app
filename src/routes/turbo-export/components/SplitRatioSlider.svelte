<script lang="ts">
	import {
		trainRatio,
		valRatio,
		testRatio,
		adjustSplitRatios
	} from '../stores/exportStore';

	// 滑桿配置
	const sliders = [
		{ name: 'train', label: 'Train', store: trainRatio, color: 'blue', emoji: '🔵' },
		{ name: 'val', label: 'Val', store: valRatio, color: 'amber', emoji: '🟡' },
		{ name: 'test', label: 'Test', store: testRatio, color: 'rose', emoji: '🔴' }
	] as const;

	// 處理滑桿輸入
	function handleRangeInput(name: 'train' | 'val' | 'test', value: number) {
		if (name === 'train') trainRatio.set(value);
		else if (name === 'val') valRatio.set(value);
		else testRatio.set(value);
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
				value={$trainRatio}
				oninput={(e) => handleRangeInput('train', parseInt(e.currentTarget.value))}
				min="0"
				max="100"
				class="flex-1 h-2 bg-slate-200 dark:bg-slate-600 rounded-lg appearance-none cursor-pointer accent-blue-600"
			/>
			<input
				type="number"
				value={$trainRatio}
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
				value={$valRatio}
				oninput={(e) => handleRangeInput('val', parseInt(e.currentTarget.value))}
				min="0"
				max="100"
				class="flex-1 h-2 bg-slate-200 dark:bg-slate-600 rounded-lg appearance-none cursor-pointer accent-amber-500"
			/>
			<input
				type="number"
				value={$valRatio}
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
				value={$testRatio}
				oninput={(e) => handleRangeInput('test', parseInt(e.currentTarget.value))}
				min="0"
				max="100"
				class="flex-1 h-2 bg-slate-200 dark:bg-slate-600 rounded-lg appearance-none cursor-pointer accent-rose-500"
			/>
			<input
				type="number"
				value={$testRatio}
				onchange={(e) => handleRangeInput('test', parseInt(e.currentTarget.value))}
				min="0"
				max="100"
				class="w-20 px-3 py-1.5 text-right text-sm font-mono border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-700 text-slate-800 dark:text-slate-100 focus:ring-2 focus:ring-rose-500 focus:border-rose-500"
			/>
			<span class="text-sm text-slate-500 dark:text-slate-400">%</span>
		</div>

		<!-- 視覺化比例條 -->
		<div class="flex h-3 rounded-full overflow-hidden mt-2">
			<div class="bg-blue-500" style="width: {$trainRatio}%"></div>
			<div class="bg-amber-500" style="width: {$valRatio}%"></div>
			<div class="bg-rose-500" style="width: {$testRatio}%"></div>
		</div>
		<div class="flex text-xs text-slate-500 dark:text-slate-400">
			<span class="flex-1">🔵 Train {$trainRatio}%</span>
			<span class="flex-1 text-center">🟡 Val {$valRatio}%</span>
			<span class="flex-1 text-right">🔴 Test {$testRatio}%</span>
		</div>
	</div>
</section>

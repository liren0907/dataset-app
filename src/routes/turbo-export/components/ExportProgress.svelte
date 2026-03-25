<script lang="ts">
	import type { Snippet } from 'svelte';
	import {
		paths,
		execution
	} from '../stores/exportStore.svelte';

	let { onstartExport, children }: {
		onstartExport?: () => void;
		children?: Snippet;
	} = $props();

	function handleStartExport() {
		onstartExport?.();
	}
</script>

<section class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-6 shadow-sm">
	{#if execution.isProcessing}
		<!-- 進度顯示 -->
		<div class="space-y-4">
			<div class="flex items-center justify-between">
				<span class="text-sm font-medium text-slate-700 dark:text-slate-300">處理進度</span>
				<span class="text-sm text-slate-500 dark:text-slate-400">{Math.round(execution.progress)}%</span>
			</div>
			<div class="w-full bg-slate-200 dark:bg-slate-700 rounded-full h-3 overflow-hidden">
				<div
					class="bg-gradient-to-r from-indigo-500 to-indigo-600 h-3 rounded-full transition-all duration-300"
					style="width: {execution.progress}%"
				></div>
			</div>
			<div class="flex items-center justify-center gap-2 text-sm text-slate-500 dark:text-slate-400">
				<svg class="animate-spin h-4 w-4" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
				</svg>
				<span>正在轉換中，請稍候...</span>
			</div>
		</div>
	{:else}
		<!-- 開始按鈕 -->
		<button
			onclick={handleStartExport}
			disabled={!paths.sourceDir}
			class="w-full py-4 bg-gradient-to-r from-indigo-600 to-indigo-700 text-white rounded-xl font-bold text-lg hover:from-indigo-700 hover:to-indigo-800 transition-all disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-indigo-500/25"
		>
			🚀 開始轉換
		</button>
	{/if}

	{#if execution.statusMessage}
		<div class="mt-4 text-center text-sm {execution.statusMessage.includes('✅') ? 'text-emerald-600 dark:text-emerald-400' : execution.statusMessage.includes('❌') ? 'text-rose-600 dark:text-rose-400' : 'text-slate-600 dark:text-slate-400'}">
			{execution.statusMessage}
		</div>
	{/if}

	<!-- 詳細統計 -->
	{@render children?.()}
</section>

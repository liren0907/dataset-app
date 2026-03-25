<script lang="ts">
	import {
		format,
		type OutputFormat,
		type AnnotationType,
		type LabelMeOutputFormat
	} from '../stores/exportStore.svelte';

	// 格式選項配置
	const outputFormats: Array<{
		value: OutputFormat;
		label: string;
		description: string;
	}> = [
		{ value: 'yolo', label: 'YOLO', description: 'YOLO格式' },
		{ value: 'coco', label: 'COCO', description: 'instances.json' },
		{ value: 'labelme', label: 'LabelMe', description: '過濾標籤' }
	];

	const annotationTypes: Array<{
		value: AnnotationType;
		label: string;
		description: string;
	}> = [
		{ value: 'bbox', label: 'Bounding Box', description: '物件偵測' },
		{ value: 'polygon', label: 'Polygon', description: '實例分割' }
	];

	// LabelMe 輸出點格式選項
	const labelmeOutputFormats: Array<{
		value: LabelMeOutputFormat;
		label: string;
		description: string;
	}> = [
		{ value: 'original', label: '保持原樣', description: '不轉換' },
		{ value: 'bbox_2point', label: 'Bbox (2點)', description: '對角線表示' },
		{ value: 'bbox_4point', label: 'Bbox (4點)', description: '四角落表示' }
	];

	// LabelMe 格式時顯示 LabelMe 輸出選項，其他格式顯示標註類型
	let showAnnotationType = $derived(format.outputTarget !== 'labelme');
	let showLabelMeOutputFormat = $derived(format.outputTarget === 'labelme');
</script>

<section class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-6 shadow-sm">
	<h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 mb-4 flex items-center gap-2">
		🎯 輸出格式
	</h2>

	<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
		<!-- 目標格式 -->
		<div class="{showAnnotationType || showLabelMeOutputFormat ? '' : 'md:col-span-2'}">
			<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">目標格式</label>
			<div class="flex gap-2">
				{#each outputFormats as fmt}
					<button
						onclick={() => format.outputTarget = fmt.value}
						class="flex-1 px-4 py-3 rounded-lg border-2 transition-all text-sm font-medium
							{format.outputTarget === fmt.value
								? 'border-indigo-500 bg-indigo-50 dark:bg-indigo-900/50 text-indigo-700 dark:text-indigo-300'
								: 'border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-500'}"
					>
						<div class="font-bold">{fmt.label}</div>
						<div class="text-xs opacity-75">{fmt.description}</div>
					</button>
				{/each}
			</div>
		</div>

		<!-- 標註類型（YOLO/COCO 格式時顯示）-->
		{#if showAnnotationType}
			<div>
				<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">標註類型</label>
				<div class="flex gap-2">
					{#each annotationTypes as type}
						<button
							onclick={() => format.annotationType = type.value}
							class="flex-1 px-4 py-3 rounded-lg border-2 transition-all text-sm font-medium
								{format.annotationType === type.value
									? 'border-emerald-500 bg-emerald-50 dark:bg-emerald-900/50 text-emerald-700 dark:text-emerald-300'
									: 'border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-500'}"
						>
							<div class="font-bold">{type.label}</div>
							<div class="text-xs opacity-75">{type.description}</div>
						</button>
					{/each}
				</div>
			</div>
		{/if}

		<!-- LabelMe 輸出點格式（LabelMe 格式時顯示）-->
		{#if showLabelMeOutputFormat}
			<div>
				<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">輸出點格式</label>
				<div class="flex gap-2">
					{#each labelmeOutputFormats as fmt}
						<button
							onclick={() => format.labelmeOutputFormat = fmt.value}
							class="flex-1 px-4 py-3 rounded-lg border-2 transition-all text-sm font-medium
								{format.labelmeOutputFormat === fmt.value
									? 'border-amber-500 bg-amber-50 dark:bg-amber-900/50 text-amber-700 dark:text-amber-300'
									: 'border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-500'}"
						>
							<div class="font-bold">{fmt.label}</div>
							<div class="text-xs opacity-75">{fmt.description}</div>
						</button>
					{/each}
				</div>
			</div>
		{/if}
	</div>
</section>

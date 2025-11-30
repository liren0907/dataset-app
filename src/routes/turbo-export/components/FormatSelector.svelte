<script lang="ts">
	import {
		outputTarget,
		annotationType,
		type OutputFormat,
		type AnnotationType
	} from '../stores/exportStore';

	// 格式選項配置
	const outputFormats: Array<{
		value: OutputFormat;
		label: string;
		description: string;
	}> = [
		{ value: 'yolo', label: 'YOLO', description: 'YOLOv5 / v8 ~ v12' },
		{ value: 'coco', label: 'COCO', description: 'instances.json' }
	];

	const annotationTypes: Array<{
		value: AnnotationType;
		label: string;
		description: string;
	}> = [
		{ value: 'bbox', label: 'Bounding Box', description: '物件偵測' },
		{ value: 'polygon', label: 'Polygon', description: '實例分割' }
	];
</script>

<section class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-6 shadow-sm">
	<h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 mb-4 flex items-center gap-2">
		🎯 輸出格式
	</h2>

	<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
		<!-- 目標格式 -->
		<div>
			<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">目標格式</label>
			<div class="flex gap-2">
				{#each outputFormats as format}
					<button
						on:click={() => outputTarget.set(format.value)}
						class="flex-1 px-4 py-3 rounded-lg border-2 transition-all text-sm font-medium
							{$outputTarget === format.value
								? 'border-indigo-500 bg-indigo-50 dark:bg-indigo-900/50 text-indigo-700 dark:text-indigo-300'
								: 'border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-500'}"
					>
						<div class="font-bold">{format.label}</div>
						<div class="text-xs opacity-75">{format.description}</div>
					</button>
				{/each}
			</div>
		</div>

		<!-- 標註類型 -->
		<div>
			<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">標註類型</label>
			<div class="flex gap-2">
				{#each annotationTypes as type}
					<button
						on:click={() => annotationType.set(type.value)}
						class="flex-1 px-4 py-3 rounded-lg border-2 transition-all text-sm font-medium
							{$annotationType === type.value
								? 'border-emerald-500 bg-emerald-50 dark:bg-emerald-900/50 text-emerald-700 dark:text-emerald-300'
								: 'border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-500'}"
					>
						<div class="font-bold">{type.label}</div>
						<div class="text-xs opacity-75">{type.description}</div>
					</button>
				{/each}
			</div>
		</div>
	</div>
</section>

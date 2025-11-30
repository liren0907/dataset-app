<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	// ===== 匯入組件 =====
	import SourceOutputSection from './components/SourceOutputSection.svelte';
	import FormatSelector from './components/FormatSelector.svelte';
	import SplitRatioSlider from './components/SplitRatioSlider.svelte';
	import LabelManager from './components/LabelManager.svelte';
	import AdvancedOptions from './components/AdvancedOptions.svelte';
	import ExportProgress from './components/ExportProgress.svelte';
	import ExportResult from './components/ExportResult.svelte';

	// ===== 匯入 Composables =====
	import { setupDragDropListener, cleanupDragDropListeners } from './composables/useTauriDragDrop';

	// ===== 匯入 Services =====
	import {
		scanLabels as scanLabelsService,
		scanLabelsWithCounts,
		convertLabelMe,
		getTotalAnnotationCount,
		type ConvertLabelMeRequest
	} from './services/exportService';

	// ===== 匯入 Store =====
	import {
		// Stores
		sourceDir,
		outputDir,
		outputTarget,
		annotationType,
		customDatasetName,
		trainRatio,
		valRatio,
		testRatio,
		useCustomLabels,
		labelList,
		isScanning,
		labelScanMessage,
		isCalculatingCounts,
		includeBackground,
		workerCount,
		randomSeed,
		removeImageData,
		isProcessing,
		progress,
		statusMessage,
		stats,
		detailedStats,
		showInvalidDetails
	} from './stores/exportStore';

	// ===== 來源與輸出設定（已移至 store）=====
	// let sourceDir: string = '';
	// let outputDir: string = '';

	// ===== Tauri 拖放狀態 =====
	let sourceDropZone: HTMLElement | null = null;
	let outputDropZone: HTMLElement | null = null;
	let unlistenFns: Array<() => void> = [];

	// ===== 設置拖放監聽器（使用 composable）=====
	async function initDragDrop() {
		const refs = { sourceDropZone, outputDropZone };
		const callbacks = {
			onSourceDrop: (path: string) => {
				sourceDir.set(path);
				scanLabels();
			},
			onOutputDrop: (path: string) => {
				outputDir.set(path);
			}
		};
		unlistenFns = await setupDragDropListener(refs, callbacks);
	}

	// ===== 處理來源路徑選擇事件（由 SourceOutputSection 發出）=====
	function handleSourceSelected() {
		scanLabels();
	}

	onMount(() => {
		// 需要等 bind:this 完成，所以用 setTimeout
		setTimeout(() => initDragDrop(), 0);
	});

	onDestroy(() => {
		cleanupDragDropListeners(unlistenFns);
	});

	// ===== 輸出格式設定（已移至 store / FormatSelector）=====
	// ===== 資料集分割設定（已移至 store / SplitRatioSlider）=====
	// ===== 標籤設定（已移至 store / LabelManager）=====

	// 背景統計狀態（本地使用）
	let countCalculationPromise: Promise<void> | null = null;
	let labelScanAbortController: AbortController | null = null;

	// ===== 進階選項（已移至 store）=====
	// showAdvanced, includeBackground, workerCount, randomSeed 已移至 store

	// ===== 執行狀態（已移至 store）=====
	// isProcessing, progress, statusMessage, stats, detailedStats, showInvalidDetails 已移至 store
	// invalidReasonGroups 由 store 的 derived store 計算
	let progressInterval: ReturnType<typeof setInterval> | null = null;

	// ===== 掃描標籤（完全非阻塞版本）=====
	async function scanLabels() {
		if (!$sourceDir) return;

		// 取消之前的計算（如果有）
		if (labelScanAbortController) {
			labelScanAbortController.abort();
		}
		labelScanAbortController = new AbortController();
		const signal = labelScanAbortController.signal;

		isScanning.set(true);
		statusMessage.set('正在掃描標籤...');

		// 立即開始後端呼叫，但不等待
		const scanPromise = scanLabelsService($sourceDir);

		// 同時在背景啟動數量計算（核彈級別：完全並行）
		scheduleBackgroundCountCalculation($sourceDir, signal);

		try {
			// 等待標籤掃描完成
			const result = await scanPromise;

			if (signal.aborted) return;

			// DEBUG: 輸出原始結果
			console.log('🔍 scan_labelme_labels 原始回傳:', result);

			// 轉換為 labelList 格式，並加入 id
			// 先設定 count 為 0，背景計算後再更新
			labelList.set(result.map((name: string, i: number) => ({
				id: i + 1,
				name,
				count: 0,
				selected: true
			})));

			// DEBUG: 輸出轉換後結果
			console.log('🔍 轉換後 labelList:', $labelList);

			labelScanMessage.set(`找到 ${$labelList.length} 個標籤`);
			statusMessage.set('');
		} catch (error) {
			if (signal.aborted) return;
			console.error('掃描標籤失敗:', error);
			statusMessage.set(`掃描失敗: ${error}`);
		} finally {
			if (!signal.aborted) {
				isScanning.set(false);
			}
		}
	}

	// ===== 核彈級別：完全非阻塞的背景計算 =====
	// 使用多重調度確保 UI 完全不受影響
	function scheduleBackgroundCountCalculation(sourceDir: string, signal: AbortSignal) {
		if ($isCalculatingCounts) return;

		// 第一層：使用 requestAnimationFrame 確保當前渲染幀完成
		requestAnimationFrame(() => {
			if (signal.aborted) return;

			// 第二層：使用 setTimeout 跳出當前事件循環
			setTimeout(() => {
				if (signal.aborted) return;

				// 第三層：使用 requestIdleCallback（如果支援）在瀏覽器空閒時執行
				// 這是最重要的一層，確保只在瀏覽器「真正空閒」時才執行
				const scheduleTask = window.requestIdleCallback || ((cb: () => void) => setTimeout(cb, 1));

				scheduleTask(() => {
					if (signal.aborted) return;
					executeBackgroundCountCalculation(sourceDir, signal);
				});
			}, 50); // 給 UI 50ms 的喘息空間
		});
	}

	// ===== 實際執行背景計算 =====
	function executeBackgroundCountCalculation(sourceDir: string, signal: AbortSignal) {
		if (signal.aborted || $isCalculatingCounts) return;

		isCalculatingCounts.set(true);
		console.log('📊 開始背景計算標籤數量...');

		countCalculationPromise = (async () => {
			try {
				const counts = await scanLabelsWithCounts(sourceDir);

				if (signal.aborted) return;

				console.log('📊 標籤數量統計完成:', counts);

				// 更新 labelList 中的 count
				labelList.update(list => list.map(label => ({
					...label,
					count: counts[label.name] ?? 0
				})));

				// 計算總數
				const totalCount = getTotalAnnotationCount(counts);
				labelScanMessage.set(`找到 ${$labelList.length} 個標籤，共 ${totalCount.toLocaleString()} 個標註`);
			} catch (error) {
				if (signal.aborted) return;
				console.error('📊 標籤數量計算失敗:', error);
			} finally {
				if (!signal.aborted) {
					isCalculatingCounts.set(false);
					countCalculationPromise = null;
				}
			}
		})();
	}

	// ===== 開始轉換 =====
	async function startExport() {
		if (!$sourceDir) {
			statusMessage.set('請先選擇來源路徑！');
			return;
		}

		isProcessing.set(true);
		progress.set(0);
		stats.set({ total: 0, processed: 0, success: 0, skipped: 0, failed: 0 });
		detailedStats.set({ totalAnnotations: 0, skippedAnnotations: 0, backgroundImages: 0, backgroundFiles: [], skippedLabels: [], invalidAnnotations: [] });
		showInvalidDetails.set(false);
		statusMessage.set('開始處理...');

		// 啟動模擬進度條（因為後端沒有即時回報進度）
		startProgressSimulation();

		try {
			// 建立標籤列表（按順序）
			let labelListForConvert: string[] = [];
			if ($useCustomLabels && $labelList.length > 0) {
				labelListForConvert = $labelList
					.filter(l => l.selected)
					.map(l => l.name);
			}

			// 組裝請求參數
			const request: ConvertLabelMeRequest = {
				input_dir: $sourceDir,
				output_dir: $outputDir || null,
				output_format: $outputTarget,
				annotation_format: $annotationType,
				val_size: $valRatio / 100,
				test_size: $testRatio / 100,
				seed: $randomSeed,
				include_background: $includeBackground,
				label_list: labelListForConvert,
				deterministic_labels: $useCustomLabels,
				segmentation_mode: $annotationType === 'polygon' ? 'polygon' : 'bbox_only',
				custom_dataset_name: $customDatasetName || null,
				remove_image_data: $removeImageData
			};

			// 呼叫後端進行轉換
			const result = await convertLabelMe(request);

			// 停止模擬進度
			stopProgressSimulation();

			if (result.success) {
				stats.set({
					total: result.stats.total_files,
					processed: result.stats.processed_files,
					success: result.stats.processed_files - result.stats.failed_files,
					skipped: result.stats.skipped_files,
					failed: result.stats.failed_files
				});

				// 詳細統計
				detailedStats.set({
					totalAnnotations: result.stats.total_annotations,
					skippedAnnotations: result.stats.skipped_annotations,
					backgroundImages: result.stats.background_images,
					backgroundFiles: result.stats.background_files || [],
					skippedLabels: result.stats.skipped_labels || [],
					invalidAnnotations: result.stats.invalid_annotations || []
				});

				progress.set(100);

				// 構建完成訊息
				let message = `✅ 轉換完成！共處理 ${result.stats.total_annotations.toLocaleString()} 個標註`;
				if (result.stats.skipped_annotations > 0) {
					message += `，跳過 ${result.stats.skipped_annotations.toLocaleString()} 個`;
				}
				if (result.stats.background_images > 0) {
					message += `，背景圖片 ${result.stats.background_images} 張`;
				}
				statusMessage.set(message);
			} else {
				statusMessage.set(`❌ 轉換失敗: ${result.errors.join(', ')}`);
			}
		} catch (error) {
			stopProgressSimulation();
			console.error('轉換失敗:', error);
			statusMessage.set(`❌ 轉換失敗: ${error}`);
		} finally {
			isProcessing.set(false);
		}
	}

	// ===== 模擬進度條 =====
	function startProgressSimulation() {
		// 模擬進度：快速到 30%，然後慢慢到 90%
		progress.set(0);
		progressInterval = setInterval(() => {
			progress.update(p => {
				if (p < 30) return p + 5;
				else if (p < 60) return p + 2;
				else if (p < 90) return p + 0.5;
				return p;
			});
			// 最多到 90%，剩下的等實際完成
		}, 100);
	}

	function stopProgressSimulation() {
		if (progressInterval) {
			clearInterval(progressInterval);
			progressInterval = null;
		}
	}
</script>

<svelte:head>
	<title>Turbo Export - Dataset App</title>
</svelte:head>

<div class="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800 px-4 py-8">
	<div class="max-w-4xl mx-auto">
		<!-- 標題 -->
		<div class="text-center mb-8">
			<h1 class="text-4xl font-bold text-slate-800 dark:text-slate-100 mb-2">
				⚡ Turbo Export
			</h1>
			<p class="text-slate-600 dark:text-slate-400">
				高效能 LabelMe 轉換工具 — 比 Python 快 100 倍
			</p>
		</div>

		<!-- 主要設定區塊 -->
		<div class="space-y-6">
			<!-- 來源與輸出 -->
			<SourceOutputSection
				bind:sourceDropZone
				bind:outputDropZone
				on:sourceSelected={handleSourceSelected}
			/>

			<!-- 輸出格式 -->
			<FormatSelector />

			<!-- 資料集分割（LabelMe 輸出時不需要分割）-->
			{#if $outputTarget !== 'labelme'}
				<SplitRatioSlider />
			{/if}

			<!-- 標籤選擇 -->
			<LabelManager on:rescan={scanLabels} />

			<!-- 進階選項 -->
			<AdvancedOptions />

			<!-- 執行區塊 -->
			<ExportProgress on:startExport={startExport}>
				{#if $progress === 100 && $stats.processed > 0}
					<ExportResult />
				{/if}
			</ExportProgress>
		</div>

		<!-- 返回按鈕 -->
		<div class="mt-8 text-center">
			<a href="/" class="text-sm text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 transition-colors">
				← 返回首頁
			</a>
		</div>
	</div>
</div>

<style>
	/* 微妙的脈動動畫 */
	@keyframes pulse-subtle {
		0%, 100% {
			opacity: 1;
		}
		50% {
			opacity: 0.85;
		}
	}

	:global(.animate-pulse-subtle) {
		animation: pulse-subtle 2s ease-in-out infinite;
	}
</style>
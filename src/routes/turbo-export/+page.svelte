<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { dndzone } from 'svelte-dnd-action';
	import { flip } from 'svelte/animate';
	import { invoke } from '@tauri-apps/api/core';

	// ===== 來源與輸出設定 =====
	let sourceDir: string = '';
	let outputDir: string = '';

	// ===== Tauri 拖放狀態 =====
	let isDraggingOver: boolean = false;
	let sourceDropZone: HTMLElement | null = null;
	let outputDropZone: HTMLElement | null = null;
	let activeDropZone: 'source' | 'output' | null = null;
	let unlistenFns: Array<() => void> = [];

	// 檢查物理座標是否在元素範圍內
	// Tauri 給的是 PhysicalPosition（考慮 DPI 縮放），需要轉換為 CSS 座標
	function isPointInElement(physicalX: number, physicalY: number, element: HTMLElement | null): boolean {
		if (!element) return false;

		// 將物理座標轉換為 CSS 座標（除以 DPI 縮放比例）
		const scaleFactor = window.devicePixelRatio || 1;
		const cssX = physicalX / scaleFactor;
		const cssY = physicalY / scaleFactor;

		const rect = element.getBoundingClientRect();
		return cssX >= rect.left && cssX <= rect.right && cssY >= rect.top && cssY <= rect.bottom;
	}

	// 設置 Tauri 拖放監聽器
	async function setupDragDropListener() {
		if (typeof window === 'undefined' || !(window as any).__TAURI__) return;

		try {
			const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
			const appWindow = getCurrentWebviewWindow();

			const unlisten = await appWindow.onDragDropEvent((event) => {
				handleDragDropEvent(event.payload);
			});

			unlistenFns.push(unlisten);
		} catch (error) {
			console.error('❌ 拖放監聽器設置失敗:', error);
		}
	}

	// 處理拖放事件
	function handleDragDropEvent(payload: any) {
		const eventType = payload.type;

		if (eventType === 'enter' || eventType === 'over') {
			isDraggingOver = true;
			const position = payload.position;

			// 檢查滑鼠在哪個區域
			if (position && sourceDropZone && isPointInElement(position.x, position.y, sourceDropZone)) {
				activeDropZone = 'source';
			} else if (position && outputDropZone && isPointInElement(position.x, position.y, outputDropZone)) {
				activeDropZone = 'output';
			} else {
				activeDropZone = null;
			}
		} else if (eventType === 'drop') {
			const paths = payload.paths;
			const dropPosition = payload.position;

			// 用 drop 事件的座標計算目標區域
			let dropZone: 'source' | 'output' | null = null;
			if (dropPosition && sourceDropZone && outputDropZone) {
				if (isPointInElement(dropPosition.x, dropPosition.y, sourceDropZone)) {
					dropZone = 'source';
				} else if (isPointInElement(dropPosition.x, dropPosition.y, outputDropZone)) {
					dropZone = 'output';
				}
			} else {
				dropZone = activeDropZone;
			}

			if (paths && paths.length > 0 && dropZone) {
				const droppedPath = paths[0];
				if (dropZone === 'source') {
					sourceDir = droppedPath;
					scanLabels();
				} else if (dropZone === 'output') {
					outputDir = droppedPath;
				}
			}

			isDraggingOver = false;
			activeDropZone = null;
		} else if (eventType === 'leave' || eventType === 'cancel') {
			isDraggingOver = false;
			activeDropZone = null;
		}
	}

	onMount(() => {
		setupDragDropListener();
	});

	onDestroy(() => {
		unlistenFns.forEach((fn) => fn());
	});

	// ===== 輸出格式設定 =====
	let outputTarget: 'yolo' | 'coco' = 'yolo';
	let annotationType: 'bbox' | 'polygon' = 'bbox';

	// ===== 輸出路徑命名 =====
	let customDatasetName: string = '';

	// 生成預設資料夾名稱預覽
	$: defaultDatasetName = generateDefaultName(sourceDir, outputTarget, annotationType);

	function generateDefaultName(source: string, format: string, annotation: string): string {
		if (!source) return '';
		const sourceName = source.split(/[\\/]/).pop() || 'dataset';
		const now = new Date();
		const datetime = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}_${String(now.getHours()).padStart(2, '0')}${String(now.getMinutes()).padStart(2, '0')}${String(now.getSeconds()).padStart(2, '0')}`;
		return `${sourceName}_${format}_${annotation}_${datetime}`;
	}

	// ===== 資料集分割設定 =====
	let trainRatio: number = 70;
	let valRatio: number = 20;
	let testRatio: number = 10;

	// 確保比例總和為 100，調整時自動分配剩餘空間
	function adjustRatios(changed: 'train' | 'val' | 'test') {
		// 計算當前變更的值佔用後，剩餘多少給其他兩個
		if (changed === 'train') {
			const remaining = 100 - trainRatio;
			const otherTotal = valRatio + testRatio;
			if (otherTotal === 0) {
				valRatio = remaining;
				testRatio = 0;
			} else {
				const scale = remaining / otherTotal;
				valRatio = Math.round(valRatio * scale);
				testRatio = 100 - trainRatio - valRatio;
			}
		} else if (changed === 'val') {
			const remaining = 100 - valRatio;
			const otherTotal = trainRatio + testRatio;
			if (otherTotal === 0) {
				trainRatio = remaining;
				testRatio = 0;
			} else {
				const scale = remaining / otherTotal;
				trainRatio = Math.round(trainRatio * scale);
				testRatio = 100 - trainRatio - valRatio;
			}
		} else {
			const remaining = 100 - testRatio;
			const otherTotal = trainRatio + valRatio;
			if (otherTotal === 0) {
				trainRatio = remaining;
				valRatio = 0;
			} else {
				const scale = remaining / otherTotal;
				trainRatio = Math.round(trainRatio * scale);
				valRatio = 100 - trainRatio - testRatio;
			}
		}
		// 確保不會有負數
		trainRatio = Math.max(0, trainRatio);
		valRatio = Math.max(0, valRatio);
		testRatio = Math.max(0, testRatio);
	}

	// ===== 標籤設定 =====
	let useCustomLabels: boolean = false;

	// 標籤資料結構：包含 id（必須）、名稱、數量、是否選中
	interface LabelInfo {
		id: number;      // svelte-dnd-action 必須要有 id
		name: string;
		count: number;
		selected: boolean;
	}
	let labelList: LabelInfo[] = []; // 順序即為 class ID 映射
	let isScanning: boolean = false;
	let labelScanMessage: string = '';

	// 背景統計狀態
	let isCalculatingCounts: boolean = false;
	let countCalculationPromise: Promise<void> | null = null;

	// 拖拉動畫時間
	const flipDurationMs = 200;

	function toggleLabel(id: number) {
		labelList = labelList.map(l =>
			l.id === id ? { ...l, selected: !l.selected } : l
		);
	}

	function selectAllLabels() {
		labelList = labelList.map(l => ({ ...l, selected: true }));
	}

	function deselectAllLabels() {
		labelList = labelList.map(l => ({ ...l, selected: false }));
	}

	// ===== svelte-dnd-action 事件處理 =====
	function handleDndConsider(e: CustomEvent<{items: LabelInfo[]}>) {
		labelList = e.detail.items;
	}

	function handleDndFinalize(e: CustomEvent<{items: LabelInfo[]}>) {
		labelList = e.detail.items;
	}

	// 取得標籤 ID 映射表（順序就是 class ID）
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	function getLabelIdMapping(): Record<string, number> {
		const mapping: Record<string, number> = {};
		let classId = 0;
		for (const label of labelList) {
			if (label.selected) {
				mapping[label.name] = classId++;
			}
		}
		return mapping;
	}

	// ===== 進階選項 =====
	let showAdvanced: boolean = false;
	let includeBackground: boolean = false;
	let workerCount: number = 0; // 0 = 自動
	let randomSeed: number = 42;
	// deterministic 已移除，因為排序就是映射

	// ===== 執行狀態 =====
	let isProcessing: boolean = false;
	let progress: number = 0;
	let statusMessage: string = '';
	let progressInterval: ReturnType<typeof setInterval> | null = null;
	let stats = {
		total: 0,
		processed: 0,
		success: 0,
		skipped: 0,
		failed: 0
	};

	// 無效標註記錄
	interface InvalidAnnotation {
		file: string;
		label: string;
		reason: string;
		shape_type: string;
		points_count: number;
	}

	// 詳細統計
	let detailedStats = {
		totalAnnotations: 0,
		skippedAnnotations: 0,
		backgroundImages: 0,
		skippedLabels: [] as string[],
		invalidAnnotations: [] as InvalidAnnotation[]
	};

	// 是否展開無效標註詳情
	let showInvalidDetails: boolean = false;

	// 計算無效標註按原因分組
	$: invalidReasonGroups = detailedStats.invalidAnnotations.reduce((acc, item) => {
		if (!acc[item.reason]) acc[item.reason] = [];
		acc[item.reason].push(item);
		return acc;
	}, {} as Record<string, InvalidAnnotation[]>);

	// ===== 選擇資料夾 =====
	async function selectSourceDir() {
		const selected = await open({
			directory: true,
			multiple: false,
			title: '選擇 LabelMe JSON 資料夾'
		});
		if (selected && typeof selected === 'string') {
			sourceDir = selected;
			// 自動掃描標籤
			await scanLabels();
		}
	}

	async function selectOutputDir() {
		const selected = await open({
			directory: true,
			multiple: false,
			title: '選擇輸出路徑'
		});
		if (selected && typeof selected === 'string') {
			outputDir = selected;
		}
	}

	// ===== 掃描標籤 =====
	async function scanLabels() {
		if (!sourceDir) return;

		isScanning = true;
		statusMessage = '正在掃描標籤...';

		try {
			// 呼叫 Rust 後端掃描標籤（回傳 string[] 而非 Record）
			const result = await invoke<string[]>('scan_labelme_labels', {
				inputDir: sourceDir
			});

			// DEBUG: 輸出原始結果
			console.log('🔍 scan_labelme_labels 原始回傳:', result);
			console.log('🔍 結果類型:', typeof result);
			console.log('🔍 是否為陣列:', Array.isArray(result));
			if (result && result.length > 0) {
				console.log('🔍 第一個元素:', result[0], '類型:', typeof result[0]);
			}

			// 轉換為 labelList 格式，並加入 id
			// 先設定 count 為 0，背景計算後再更新
			labelList = result.map((name, i) => ({
				id: i + 1,
				name,
				count: 0,
				selected: true
			}));

			// DEBUG: 輸出轉換後結果
			console.log('🔍 轉換後 labelList:', labelList);

			labelScanMessage = `找到 ${labelList.length} 個標籤`;
			statusMessage = '';

			// 背景計算標籤數量
			startCountCalculation();
		} catch (error) {
			console.error('掃描標籤失敗:', error);
			statusMessage = `掃描失敗: ${error}`;
		} finally {
			isScanning = false;
		}
	}

	// ===== 背景計算標籤數量 =====
	async function startCountCalculation() {
		if (!sourceDir || isCalculatingCounts) return;

		isCalculatingCounts = true;
		console.log('📊 開始背景計算標籤數量...');

		countCalculationPromise = (async () => {
			try {
				const counts = await invoke<Record<string, number>>('scan_labelme_labels_with_counts', {
					inputDir: sourceDir
				});

				console.log('📊 標籤數量統計完成:', counts);

				// 更新 labelList 中的 count
				labelList = labelList.map(label => ({
					...label,
					count: counts[label.name] ?? 0
				}));

				// 計算總數
				const totalCount = Object.values(counts).reduce((sum, c) => sum + c, 0);
				labelScanMessage = `找到 ${labelList.length} 個標籤，共 ${totalCount.toLocaleString()} 個標註`;
			} catch (error) {
				console.error('📊 標籤數量計算失敗:', error);
			} finally {
				isCalculatingCounts = false;
				countCalculationPromise = null;
			}
		})();
	}

	// ===== 開始轉換 =====
	async function startExport() {
		if (!sourceDir) {
			statusMessage = '請先選擇來源路徑！';
			return;
		}

		isProcessing = true;
		progress = 0;
		stats = { total: 0, processed: 0, success: 0, skipped: 0, failed: 0 };
		detailedStats = { totalAnnotations: 0, skippedAnnotations: 0, backgroundImages: 0, skippedLabels: [], invalidAnnotations: [] };
		showInvalidDetails = false;
		statusMessage = '開始處理...';

		// 啟動模擬進度條（因為後端沒有即時回報進度）
		startProgressSimulation();

		try {
			// 建立標籤列表（按順序）
			let labelListForConvert: string[] = [];
			if (useCustomLabels && labelList.length > 0) {
				labelListForConvert = labelList
					.filter(l => l.selected)
					.map(l => l.name);
			}

			// 呼叫 Rust 後端進行轉換
			// 注意：後端期望一個 request 物件
			const result = await invoke<{
				success: boolean;
				output_dir: string;
				stats: {
					total_files: number;
					processed_files: number;
					skipped_files: number;
					failed_files: number;
					total_annotations: number;
					skipped_annotations: number;
					background_images: number;
					labels_found: string[];
					skipped_labels: string[];
					invalid_annotations: InvalidAnnotation[];
				};
				errors: string[];
			}>('convert_labelme', {
				request: {
					input_dir: sourceDir,
					output_dir: outputDir || null,
					output_format: outputTarget,
					annotation_format: annotationType,
					val_size: valRatio / 100,
					test_size: testRatio / 100,
					seed: randomSeed,
					include_background: includeBackground,
					label_list: labelListForConvert,
					deterministic_labels: useCustomLabels,
					segmentation_mode: annotationType === 'polygon' ? 'polygon' : 'bbox_only',
					custom_dataset_name: customDatasetName || null
				}
			});

			// 停止模擬進度
			stopProgressSimulation();

			if (result.success) {
				stats.total = result.stats.total_files;
				stats.processed = result.stats.processed_files;
				stats.success = result.stats.processed_files - result.stats.failed_files;
				stats.skipped = result.stats.skipped_files;
				stats.failed = result.stats.failed_files;

				// 詳細統計
				detailedStats.totalAnnotations = result.stats.total_annotations;
				detailedStats.skippedAnnotations = result.stats.skipped_annotations;
				detailedStats.backgroundImages = result.stats.background_images;
				detailedStats.skippedLabels = result.stats.skipped_labels || [];
				detailedStats.invalidAnnotations = result.stats.invalid_annotations || [];

				progress = 100;

				// 構建完成訊息
				let message = `✅ 轉換完成！共處理 ${result.stats.total_annotations.toLocaleString()} 個標註`;
				if (detailedStats.skippedAnnotations > 0) {
					message += `，跳過 ${detailedStats.skippedAnnotations.toLocaleString()} 個`;
				}
				if (detailedStats.backgroundImages > 0) {
					message += `，背景圖片 ${detailedStats.backgroundImages} 張`;
				}
				statusMessage = message;
			} else {
				statusMessage = `❌ 轉換失敗: ${result.errors.join(', ')}`;
			}
		} catch (error) {
			stopProgressSimulation();
			console.error('轉換失敗:', error);
			statusMessage = `❌ 轉換失敗: ${error}`;
		} finally {
			isProcessing = false;
		}
	}

	// ===== 模擬進度條 =====
	function startProgressSimulation() {
		// 模擬進度：快速到 30%，然後慢慢到 90%
		progress = 0;
		progressInterval = setInterval(() => {
			if (progress < 30) {
				progress += 5;
			} else if (progress < 60) {
				progress += 2;
			} else if (progress < 90) {
				progress += 0.5;
			}
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
			<section class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-6 shadow-sm">
				<h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 mb-4 flex items-center gap-2">
					📁 來源與輸出
				</h2>

				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					<!-- 來源路徑 -->
					<div
						bind:this={sourceDropZone}
						class="relative group"
						role="button"
						tabindex="0"
					>
						<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">來源路徑</label>
						<div class="flex gap-2 transition-all duration-200 {isDraggingOver && activeDropZone !== 'source' ? 'opacity-50' : ''}">
							<input
								type="text"
								bind:value={sourceDir}
								placeholder="選擇或拖放包含 LabelMe JSON 的資料夾"
								class="flex-1 px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-slate-50 dark:bg-slate-700 dark:text-slate-100 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all duration-200
									{activeDropZone === 'source' ? 'border-indigo-400 ring-2 ring-indigo-200' : ''}"
								readonly
							/>
							<button
								on:click={selectSourceDir}
								class="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors text-sm font-medium"
							>
								瀏覽
							</button>
						</div>
						<!-- 磨砂玻璃拖放覆蓋層 -->
						{#if activeDropZone === 'source'}
							<div class="absolute inset-0 rounded-lg overflow-hidden z-10 animate-pulse-subtle">
								<div class="absolute inset-0 bg-gradient-to-br from-indigo-500/30 via-indigo-400/20 to-purple-500/30 backdrop-blur-md"></div>
								<div class="absolute inset-0 border-2 border-dashed border-indigo-400 rounded-lg"></div>
								<div class="absolute inset-0 flex items-center justify-center">
									<div class="bg-white/80 dark:bg-slate-800/80 backdrop-blur-sm px-4 py-2 rounded-full shadow-lg flex items-center gap-2">
										<svg class="w-5 h-5 text-indigo-600 animate-bounce" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
										</svg>
										<span class="text-indigo-700 dark:text-indigo-300 font-semibold text-sm">放開以設定來源</span>
									</div>
								</div>
							</div>
						{:else if isDraggingOver && activeDropZone !== 'output'}
							<!-- 拖動中但不在此區域 - 顯示提示邊框 -->
							<div class="absolute inset-0 rounded-lg border-2 border-dashed border-slate-300 dark:border-slate-600 z-10 pointer-events-none"></div>
						{/if}
					</div>

					<!-- 輸出路徑 -->
					<div
						bind:this={outputDropZone}
						class="relative group"
						role="button"
						tabindex="0"
					>
						<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">輸出路徑 (選填)</label>
						<div class="flex gap-2 transition-all duration-200 {isDraggingOver && activeDropZone !== 'output' ? 'opacity-50' : ''}">
							<input
								type="text"
								bind:value={outputDir}
								placeholder="預設為來源路徑"
								class="flex-1 px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-slate-50 dark:bg-slate-700 dark:text-slate-100 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all duration-200
									{activeDropZone === 'output' ? 'border-emerald-400 ring-2 ring-emerald-200' : ''}"
								readonly
							/>
							<button
								on:click={selectOutputDir}
								class="px-4 py-2 bg-slate-600 text-white rounded-lg hover:bg-slate-700 transition-colors text-sm font-medium"
							>
								瀏覽
							</button>
						</div>
						<!-- 磨砂玻璃拖放覆蓋層 -->
						{#if activeDropZone === 'output'}
							<div class="absolute inset-0 rounded-lg overflow-hidden z-10 animate-pulse-subtle">
								<div class="absolute inset-0 bg-gradient-to-br from-emerald-500/30 via-emerald-400/20 to-teal-500/30 backdrop-blur-md"></div>
								<div class="absolute inset-0 border-2 border-dashed border-emerald-400 rounded-lg"></div>
								<div class="absolute inset-0 flex items-center justify-center">
									<div class="bg-white/80 dark:bg-slate-800/80 backdrop-blur-sm px-4 py-2 rounded-full shadow-lg flex items-center gap-2">
										<svg class="w-5 h-5 text-emerald-600 animate-bounce" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
										</svg>
										<span class="text-emerald-700 dark:text-emerald-300 font-semibold text-sm">放開以設定輸出</span>
									</div>
								</div>
							</div>
						{:else if isDraggingOver && activeDropZone !== 'source'}
							<!-- 拖動中但不在此區域 - 顯示提示邊框 -->
							<div class="absolute inset-0 rounded-lg border-2 border-dashed border-slate-300 dark:border-slate-600 z-10 pointer-events-none"></div>
						{/if}
					</div>
				</div>

				<!-- 自訂資料夾名稱 -->
				<div class="mt-4">
					<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">
						輸出資料夾名稱 (選填)
					</label>
					<input
						type="text"
						bind:value={customDatasetName}
						placeholder="留空則自動產生"
						class="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-slate-50 dark:bg-slate-700 dark:text-slate-100 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all duration-200"
					/>
					<div class="mt-2 text-xs text-slate-500 dark:text-slate-400">
						<span class="font-medium">預覽：</span>
						<code class="bg-slate-100 dark:bg-slate-600 px-2 py-0.5 rounded text-indigo-600 dark:text-indigo-300">
							{customDatasetName || defaultDatasetName || '請先選擇來源路徑'}
						</code>
					</div>
				</div>
			</section>

			<!-- 輸出格式 -->
			<section class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-6 shadow-sm">
				<h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 mb-4 flex items-center gap-2">
					🎯 輸出格式
				</h2>

				<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
					<!-- 目標格式 -->
					<div>
						<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">目標格式</label>
						<div class="flex gap-2">
							<button
								on:click={() => outputTarget = 'yolo'}
								class="flex-1 px-4 py-3 rounded-lg border-2 transition-all text-sm font-medium
									{outputTarget === 'yolo'
										? 'border-indigo-500 bg-indigo-50 dark:bg-indigo-900/50 text-indigo-700 dark:text-indigo-300'
										: 'border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-500'}"
							>
								<div class="font-bold">YOLO</div>
								<div class="text-xs opacity-75">YOLOv5 / v8 ~ v12</div>
							</button>
							<button
								on:click={() => outputTarget = 'coco'}
								class="flex-1 px-4 py-3 rounded-lg border-2 transition-all text-sm font-medium
									{outputTarget === 'coco'
										? 'border-indigo-500 bg-indigo-50 dark:bg-indigo-900/50 text-indigo-700 dark:text-indigo-300'
										: 'border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-500'}"
							>
								<div class="font-bold">COCO</div>
								<div class="text-xs opacity-75">instances.json</div>
							</button>
						</div>
					</div>

					<!-- 標註類型 -->
					<div>
						<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">標註類型</label>
						<div class="flex gap-2">
							<button
								on:click={() => annotationType = 'bbox'}
								class="flex-1 px-4 py-3 rounded-lg border-2 transition-all text-sm font-medium
									{annotationType === 'bbox'
										? 'border-emerald-500 bg-emerald-50 dark:bg-emerald-900/50 text-emerald-700 dark:text-emerald-300'
										: 'border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-500'}"
							>
								<div class="font-bold">Bounding Box</div>
								<div class="text-xs opacity-75">物件偵測</div>
							</button>
							<button
								on:click={() => annotationType = 'polygon'}
								class="flex-1 px-4 py-3 rounded-lg border-2 transition-all text-sm font-medium
									{annotationType === 'polygon'
										? 'border-emerald-500 bg-emerald-50 dark:bg-emerald-900/50 text-emerald-700 dark:text-emerald-300'
										: 'border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-500'}"
							>
								<div class="font-bold">Polygon</div>
								<div class="text-xs opacity-75">實例分割</div>
							</button>
						</div>
					</div>
				</div>
			</section>

			<!-- 資料集分割 -->
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
							bind:value={trainRatio}
							on:change={() => adjustRatios('train')}
							min="0"
							max="100"
							class="flex-1 h-2 bg-slate-200 dark:bg-slate-600 rounded-lg appearance-none cursor-pointer accent-blue-600"
						/>
						<input
							type="number"
							bind:value={trainRatio}
							on:change={() => adjustRatios('train')}
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
							bind:value={valRatio}
							on:change={() => adjustRatios('val')}
							min="0"
							max="100"
							class="flex-1 h-2 bg-slate-200 dark:bg-slate-600 rounded-lg appearance-none cursor-pointer accent-amber-500"
						/>
						<input
							type="number"
							bind:value={valRatio}
							on:change={() => adjustRatios('val')}
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
							bind:value={testRatio}
							on:change={() => adjustRatios('test')}
							min="0"
							max="100"
							class="flex-1 h-2 bg-slate-200 dark:bg-slate-600 rounded-lg appearance-none cursor-pointer accent-rose-500"
						/>
						<input
							type="number"
							bind:value={testRatio}
							on:change={() => adjustRatios('test')}
							min="0"
							max="100"
							class="w-20 px-3 py-1.5 text-right text-sm font-mono border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-700 text-slate-800 dark:text-slate-100 focus:ring-2 focus:ring-rose-500 focus:border-rose-500"
						/>
						<span class="text-sm text-slate-500 dark:text-slate-400">%</span>
					</div>

					<!-- 視覺化比例條 -->
					<div class="flex h-3 rounded-full overflow-hidden mt-2">
						<div class="bg-blue-500" style="width: {trainRatio}%"></div>
						<div class="bg-amber-500" style="width: {valRatio}%"></div>
						<div class="bg-rose-500" style="width: {testRatio}%"></div>
					</div>
					<div class="flex text-xs text-slate-500 dark:text-slate-400">
						<span class="flex-1">🔵 Train {trainRatio}%</span>
						<span class="flex-1 text-center">🟡 Val {valRatio}%</span>
						<span class="flex-1 text-right">🔴 Test {testRatio}%</span>
					</div>
				</div>
			</section>

			<!-- 標籤選擇 -->
			<section class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-6 shadow-sm relative">
				<!-- 背景計算 Spinner -->
				{#if isCalculatingCounts}
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
					{#if labelScanMessage}
						<span class="text-sm text-emerald-600 dark:text-emerald-400 font-medium">{labelScanMessage}</span>
					{/if}
				</div>

				<!-- 切換開關 -->
				<div class="flex items-center gap-3 mb-4">
					<label class="relative inline-flex items-center cursor-pointer">
						<input type="checkbox" bind:checked={useCustomLabels} class="sr-only peer" />
						<div class="w-11 h-6 bg-slate-200 dark:bg-slate-600 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 dark:peer-focus:ring-indigo-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600"></div>
					</label>
					<span class="text-sm text-slate-700 dark:text-slate-300">只匯出選定的類別</span>
				</div>

				{#if useCustomLabels}
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
								on:click={scanLabels}
								disabled={isScanning || !sourceDir}
								class="px-3 py-1 text-xs bg-indigo-100 dark:bg-indigo-900/50 hover:bg-indigo-200 dark:hover:bg-indigo-800/50 text-indigo-700 dark:text-indigo-300 rounded-md transition-colors disabled:opacity-50"
							>
								{isScanning ? '掃描中...' : '重新掃描'}
							</button>
							<span class="ml-auto text-xs text-slate-500 dark:text-slate-400">
								已選 {labelList.filter(l => l.selected).length} / {labelList.length}
							</span>
						</div>

						<!-- 可拖拉排序的標籤表格 -->
						{#if labelList.length > 0}
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
									use:dndzone="{{ items: labelList, flipDurationMs, dropTargetStyle: {} }}"
									on:consider={handleDndConsider}
									on:finalize={handleDndFinalize}
								>
									{#each labelList as label, index (label.id)}
										<div
											animate:flip="{{ duration: flipDurationMs }}"
											class="grid grid-cols-[50px_1fr_80px_50px] gap-2 px-3 py-2 items-center bg-white dark:bg-slate-800 cursor-grab active:cursor-grabbing hover:bg-slate-50 dark:hover:bg-slate-700/50
												{label.selected ? '' : 'text-slate-400 dark:text-slate-500 bg-slate-50 dark:bg-slate-800/50'}"
										>
											<!-- ID -->
											<span class="text-center font-mono text-sm {label.selected ? 'text-indigo-600 dark:text-indigo-400 font-bold' : ''}">
												{label.selected ? labelList.slice(0, index + 1).filter(l => l.selected).length - 1 : '-'}
											</span>
											<!-- 拖拉手把 + 標籤名稱 -->
											<div class="flex items-center gap-2 select-none">
												<span class="text-slate-400 dark:text-slate-500">⋮⋮</span>
												<span class="text-sm font-medium text-slate-800 dark:text-slate-200">{label.name}</span>
											</div>
											<!-- 數量 -->
											<span class="text-right text-sm text-slate-500 dark:text-slate-400">
												{#if isCalculatingCounts && label.count === 0}
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
								{#if !sourceDir}
									請先選擇來源路徑以掃描可用標籤
								{:else if isScanning}
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

			<!-- 進階選項 -->
			<section class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 shadow-sm overflow-hidden">
				<button
					on:click={() => showAdvanced = !showAdvanced}
					class="w-full px-6 py-4 flex items-center justify-between text-left hover:bg-slate-50 dark:hover:bg-slate-700/50 transition-colors"
				>
					<h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 flex items-center gap-2">
						⚙️ 進階選項
					</h2>
					<svg
						class="w-5 h-5 text-slate-400 transition-transform {showAdvanced ? 'rotate-180' : ''}"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
					</svg>
				</button>

				{#if showAdvanced}
					<div class="px-6 pb-6 space-y-4 border-t border-slate-100 dark:border-slate-700 pt-4">
						<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
							<!-- 包含背景圖 -->
							<label class="flex items-center gap-3 cursor-pointer">
								<input type="checkbox" bind:checked={includeBackground} class="w-4 h-4 text-indigo-600 rounded focus:ring-indigo-500" />
								<div>
									<div class="text-sm font-medium text-slate-700 dark:text-slate-300">包含背景圖片</div>
									<div class="text-xs text-slate-500 dark:text-slate-400">將無標註的圖片也複製到輸出</div>
								</div>
							</label>

							<!-- Worker 數量 -->
							<div>
								<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">Worker 線程數</label>
								<input
									type="number"
									bind:value={workerCount}
									min="0"
									max="32"
									class="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-white dark:bg-slate-700 text-slate-800 dark:text-slate-100 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
								/>
								<p class="text-xs text-slate-500 dark:text-slate-400 mt-1">0 = 自動 (CPU 核心數)</p>
							</div>
						</div>

						<div class="grid grid-cols-1 md:grid-cols-2 gap-4">

							<!-- 隨機種子 -->
							<div>
								<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">隨機種子</label>
								<input
									type="number"
									bind:value={randomSeed}
									min="0"
									class="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-white dark:bg-slate-700 text-slate-800 dark:text-slate-100 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
								/>
								<p class="text-xs text-slate-500 dark:text-slate-400 mt-1">用於資料集分割的隨機性</p>
							</div>
						</div>
					</div>
				{/if}
			</section>

			<!-- 執行區塊 -->
			<section class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-6 shadow-sm">
				{#if isProcessing}
					<!-- 進度顯示 -->
					<div class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm font-medium text-slate-700 dark:text-slate-300">處理進度</span>
							<span class="text-sm text-slate-500 dark:text-slate-400">{Math.round(progress)}%</span>
						</div>
						<div class="w-full bg-slate-200 dark:bg-slate-700 rounded-full h-3 overflow-hidden">
							<div
								class="bg-gradient-to-r from-indigo-500 to-indigo-600 h-3 rounded-full transition-all duration-300"
								style="width: {progress}%"
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
						on:click={startExport}
						disabled={!sourceDir}
						class="w-full py-4 bg-gradient-to-r from-indigo-600 to-indigo-700 text-white rounded-xl font-bold text-lg hover:from-indigo-700 hover:to-indigo-800 transition-all disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-indigo-500/25"
					>
						🚀 開始轉換
					</button>
				{/if}

				{#if statusMessage}
					<div class="mt-4 text-center text-sm {statusMessage.includes('✅') ? 'text-emerald-600 dark:text-emerald-400' : statusMessage.includes('❌') ? 'text-rose-600 dark:text-rose-400' : 'text-slate-600 dark:text-slate-400'}">
						{statusMessage}
					</div>
				{/if}

				<!-- 詳細統計（轉換完成後顯示） -->
				{#if progress === 100 && stats.processed > 0}
					<div class="mt-4 pt-4 border-t border-slate-200 dark:border-slate-700">
						<div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-center text-sm">
							<div>
								<div class="text-2xl font-bold text-slate-800 dark:text-slate-100">{stats.processed}</div>
								<div class="text-slate-500 dark:text-slate-400">檔案處理</div>
							</div>
							<div>
								<div class="text-2xl font-bold text-emerald-600 dark:text-emerald-400">{detailedStats.totalAnnotations.toLocaleString()}</div>
								<div class="text-slate-500 dark:text-slate-400">標註匯出</div>
							</div>
							<div>
								<div class="text-2xl font-bold text-amber-600 dark:text-amber-400">{detailedStats.skippedAnnotations.toLocaleString()}</div>
								<div class="text-slate-500 dark:text-slate-400">標註跳過</div>
							</div>
							<div>
								<div class="text-2xl font-bold text-blue-600 dark:text-blue-400">{detailedStats.backgroundImages}</div>
								<div class="text-slate-500 dark:text-slate-400">背景圖片</div>
							</div>
						</div>

						<!-- 跳過的標籤提示 -->
						{#if detailedStats.skippedLabels.length > 0}
							<div class="mt-4 p-3 bg-amber-50 dark:bg-amber-900/30 rounded-lg border border-amber-200 dark:border-amber-800">
								<div class="flex items-start gap-2">
									<span class="text-amber-500">⚠️</span>
									<div class="text-sm">
										<div class="font-medium text-amber-700 dark:text-amber-300">以下標籤未被匯出（不在選擇列表中）：</div>
										<div class="text-amber-600 dark:text-amber-400 mt-1">
											{detailedStats.skippedLabels.join(', ')}
										</div>
									</div>
								</div>
							</div>
						{/if}

						<!-- 無效標註詳情（點擊展開） -->
						{#if detailedStats.invalidAnnotations.length > 0}
							<div class="mt-4 p-3 bg-orange-50 dark:bg-orange-900/30 rounded-lg border border-orange-200 dark:border-orange-800">
								<button
									on:click={() => showInvalidDetails = !showInvalidDetails}
									class="w-full flex items-center justify-between text-left"
								>
									<div class="flex items-start gap-2">
										<span class="text-orange-500">🔍</span>
										<div class="text-sm">
											<div class="font-medium text-orange-700 dark:text-orange-300">
												發現 {detailedStats.invalidAnnotations.length} 個無效標註
												{#if detailedStats.invalidAnnotations.length >= 100}
													<span class="text-orange-500">（僅顯示前 100 筆）</span>
												{/if}
											</div>
											<div class="text-orange-600 dark:text-orange-400 mt-0.5">
												點擊查看詳情，了解為何這些標註無法轉換
											</div>
										</div>
									</div>
									<span class="text-orange-500 transition-transform {showInvalidDetails ? 'rotate-180' : ''}">
										▼
									</span>
								</button>

								{#if showInvalidDetails}
									<div class="mt-3 pt-3 border-t border-orange-200 dark:border-orange-700">
										<!-- 按原因分組統計 -->
										<div class="space-y-3">
											{#each Object.entries(invalidReasonGroups) as [reason, items]}
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

						<!-- 失敗提示 -->
						{#if stats.failed > 0}
							<div class="mt-4 p-3 bg-rose-50 dark:bg-rose-900/30 rounded-lg border border-rose-200 dark:border-rose-800">
								<div class="flex items-start gap-2">
									<span class="text-rose-500">❌</span>
									<div class="text-sm text-rose-600 dark:text-rose-400">
										有 {stats.failed} 個檔案處理失敗，請檢查來源檔案是否完整。
									</div>
								</div>
							</div>
						{/if}
					</div>
				{/if}
			</section>
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
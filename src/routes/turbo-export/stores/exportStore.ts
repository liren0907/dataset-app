/**
 * Turbo Export 狀態管理
 *
 * 集中管理所有轉換相關的狀態，讓元件專注於 UI 呈現
 */

import { writable, derived, get } from 'svelte/store';

// ===== 類型定義 =====

/** 支援的輸出格式 */
export type OutputFormat = 'yolo' | 'coco' | 'labelme';

/** 標註類型 */
export type AnnotationType = 'bbox' | 'polygon';

/** LabelMe 輸出格式（僅 LabelMe → LabelMe 時使用） */
export type LabelMeOutputFormat = 'original' | 'bbox_2point' | 'bbox_4point';

/** 標籤資訊 */
export interface LabelInfo {
	id: number;
	name: string;
	count: number;
	selected: boolean;
}

/** 無效標註記錄 */
export interface InvalidAnnotation {
	file: string;
	label: string;
	reason: string;
	shape_type: string;
	points_count: number;
}

/** 處理統計 */
export interface ProcessingStats {
	total: number;
	processed: number;
	success: number;
	skipped: number;
	failed: number;
}

/** 詳細統計 */
export interface DetailedStats {
	totalAnnotations: number;
	skippedAnnotations: number;
	/** 背景圖片數量（原本就沒有 JSON 標註檔的圖片） */
	backgroundImages: number;
	/** 背景圖片檔名列表 */
	backgroundFiles: string[];
	/** 因標籤篩選而變空的圖片數量 */
	filteredEmptyImages: number;
	/** 因標籤篩選而變空的圖片檔名列表 */
	filteredEmptyFiles: string[];
	skippedLabels: string[];
	invalidAnnotations: InvalidAnnotation[];
}

/** 資料集分割比例 */
export interface SplitRatio {
	train: number;
	val: number;
	test: number;
}

/** 資料集分析結果（格式檢測） */
export interface DatasetAnalysis {
	/** 檢測到的輸入格式："Bbox2Point" | "Bbox4Point" | "Polygon" | "Unknown" */
	input_format: string;
	/** 資料集中的總檔案數 */
	total_files: number;
	/** 取樣分析的檔案數 */
	sample_files: number;
	/** 取樣分析的標註數 */
	sample_annotations: number;
	/** 信心分數 (0.0 - 1.0) */
	confidence: number;
	/** 信心分數百分比字串 (例如："87.5%") */
	confidence_percent: string;
	/** 點數分布統計 */
	points_distribution: Record<number, number>;
	/** 格式描述（人類可讀） */
	format_description: string;
}

/** 進階選項 */
export interface AdvancedOptions {
	includeBackground: boolean;
	workerCount: number;
	randomSeed: number;
}

/** 拖放區域類型 */
export type DropZoneType = 'source' | 'output' | null;

// ===== Store 定義 =====

// --- 來源與輸出 ---
export const sourceDir = writable<string>('');
export const outputDir = writable<string>('');
export const customDatasetName = writable<string>('');

// --- Tauri 拖放狀態 ---
export const isDraggingOver = writable<boolean>(false);
export const activeDropZone = writable<DropZoneType>(null);

// --- 格式設定 ---
export const outputTarget = writable<OutputFormat>('yolo');
export const outputFormat = outputTarget; // 別名，保持向後相容
export const annotationType = writable<AnnotationType>('bbox');
export const labelmeOutputFormat = writable<LabelMeOutputFormat>('original'); // LabelMe 輸出點格式

// --- 資料集分割（個別 store 方便 UI binding）---
export const trainRatio = writable<number>(70);
export const valRatio = writable<number>(20);
export const testRatio = writable<number>(10);

// 合併的 splitRatio（保持向後相容）
export const splitRatio = derived(
	[trainRatio, valRatio, testRatio],
	([$train, $val, $test]) => ({
		train: $train,
		val: $val,
		test: $test
	})
);

// --- 標籤管理 ---
export const useCustomLabels = writable<boolean>(false);
export const includeEmptyLabelImages = writable<boolean>(true); // 輸出篩選後無標籤的圖片
export const labelList = writable<LabelInfo[]>([]);
export const isScanning = writable<boolean>(false);
export const labelScanMessage = writable<string>('');
export const isCalculatingCounts = writable<boolean>(false);

// --- 格式檢測 ---
export const detectedFormat = writable<DatasetAnalysis | null>(null);
export const isDetectingFormat = writable<boolean>(false);


// --- 進階選項（個別 store 方便 UI binding）---
export const showAdvanced = writable<boolean>(false);
export const workerCount = writable<number>(0);
export const randomSeed = writable<number>(42);
export const removeImageData = writable<boolean>(true); // LabelMe 輸出專用：移除 base64 圖片資料

// 合併的 advancedOptions（保持向後相容）
// 注意：includeBackground 已移至標籤管理區的 includeEmptyLabelImages
export const advancedOptions = derived(
	[includeEmptyLabelImages, workerCount, randomSeed],
	([$includeEmptyLabelImages, $workerCount, $randomSeed]) => ({
		includeBackground: $includeEmptyLabelImages, // 向後相容映射
		workerCount: $workerCount,
		randomSeed: $randomSeed
	})
);

// --- 執行狀態 ---
export const isProcessing = writable<boolean>(false);
export const progress = writable<number>(0);
export const statusMessage = writable<string>('');
export const stats = writable<ProcessingStats>({
	total: 0,
	processed: 0,
	success: 0,
	skipped: 0,
	failed: 0
});
export const detailedStats = writable<DetailedStats>({
	totalAnnotations: 0,
	skippedAnnotations: 0,
	backgroundImages: 0,
	backgroundFiles: [],
	skippedLabels: [],
	invalidAnnotations: []
});
export const showInvalidDetails = writable<boolean>(false);

// ===== Derived Stores =====

/** 是否需要顯示資料集分割（僅訓練格式需要） */
export const showSplitRatio = derived(
	outputFormat,
	($outputFormat) => $outputFormat !== 'labelme'
);

/** 生成預設資料夾名稱 */
export const defaultDatasetName = derived(
	[sourceDir, outputFormat, annotationType],
	([$sourceDir, $outputFormat, $annotationType]) => {
		if (!$sourceDir) return '';
		const sourceName = $sourceDir.split(/[\\/]/).pop() || 'dataset';
		const now = new Date();
		const datetime = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}_${String(now.getHours()).padStart(2, '0')}${String(now.getMinutes()).padStart(2, '0')}${String(now.getSeconds()).padStart(2, '0')}`;
		return `${sourceName}_${$outputFormat}_${$annotationType}_${datetime}`;
	}
);

/** 已選取的標籤數量 */
export const selectedLabelCount = derived(
	labelList,
	($labelList) => $labelList.filter((l) => l.selected).length
);

/** 無效標註按原因分組 */
export const invalidReasonGroups = derived(detailedStats, ($detailedStats) =>
	$detailedStats.invalidAnnotations.reduce(
		(acc, item) => {
			if (!acc[item.reason]) acc[item.reason] = [];
			acc[item.reason].push(item);
			return acc;
		},
		{} as Record<string, InvalidAnnotation[]>
	)
);

/** 是否可以開始轉換 */
export const canStartExport = derived(
	[sourceDir, isProcessing],
	([$sourceDir, $isProcessing]) => !!$sourceDir && !$isProcessing
);

// ===== Actions =====

/** 調整分割比例，確保總和為 100 */
export function adjustSplitRatios(changed: 'train' | 'val' | 'test') {
	const train = get(trainRatio);
	const val = get(valRatio);
	const test = get(testRatio);

	if (changed === 'train') {
		const remaining = 100 - train;
		const otherTotal = val + test;
		if (otherTotal === 0) {
			valRatio.set(remaining);
			testRatio.set(0);
		} else {
			const scale = remaining / otherTotal;
			valRatio.set(Math.max(0, Math.round(val * scale)));
			testRatio.set(Math.max(0, 100 - train - get(valRatio)));
		}
	} else if (changed === 'val') {
		const remaining = 100 - val;
		const otherTotal = train + test;
		if (otherTotal === 0) {
			trainRatio.set(remaining);
			testRatio.set(0);
		} else {
			const scale = remaining / otherTotal;
			trainRatio.set(Math.max(0, Math.round(train * scale)));
			testRatio.set(Math.max(0, 100 - get(trainRatio) - val));
		}
	} else {
		const remaining = 100 - test;
		const otherTotal = train + val;
		if (otherTotal === 0) {
			trainRatio.set(remaining);
			valRatio.set(0);
		} else {
			const scale = remaining / otherTotal;
			trainRatio.set(Math.max(0, Math.round(train * scale)));
			valRatio.set(Math.max(0, 100 - get(trainRatio) - test));
		}
	}
}

/** 切換標籤選取狀態 */
export function toggleLabel(id: number) {
	labelList.update((list) => list.map((l) => (l.id === id ? { ...l, selected: !l.selected } : l)));
}

/** 全選標籤 */
export function selectAllLabels() {
	labelList.update((list) => list.map((l) => ({ ...l, selected: true })));
}

/** 取消全選 */
export function deselectAllLabels() {
	labelList.update((list) => list.map((l) => ({ ...l, selected: false })));
}

/** 重置執行狀態 */
export function resetExportState() {
	progress.set(0);
	stats.set({ total: 0, processed: 0, success: 0, skipped: 0, failed: 0 });
	detailedStats.set({
		totalAnnotations: 0,
		skippedAnnotations: 0,
		backgroundImages: 0,
		backgroundFiles: [],
		filteredEmptyImages: 0,
		filteredEmptyFiles: [],
		skippedLabels: [],
		invalidAnnotations: []
	});
	showInvalidDetails.set(false);
	statusMessage.set('');
}

/** 取得標籤 ID 映射表（順序就是 class ID） */
export function getLabelIdMapping(): Record<string, number> {
	const list = get(labelList);
	const mapping: Record<string, number> = {};
	let classId = 0;
	for (const label of list) {
		if (label.selected) {
			mapping[label.name] = classId++;
		}
	}
	return mapping;
}

/** 取得要轉換的標籤列表（按順序） */
export function getSelectedLabels(): string[] {
	const list = get(labelList);
	const useCustom = get(useCustomLabels);

	if (!useCustom || list.length === 0) {
		return [];
	}

	return list.filter((l) => l.selected).map((l) => l.name);
}

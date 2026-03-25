/**
 * Turbo Export 狀態管理 (Svelte 5 Runes)
 *
 * 集中管理所有轉換相關的狀態，讓元件專注於 UI 呈現
 * 使用 $state 物件分組，方便跨模組直接賦值
 */

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

// ===== State 定義 =====

// --- 來源與輸出 ---
export const paths = $state({
	sourceDir: '',
	outputDir: '',
	customDatasetName: ''
});

// --- Tauri 拖放狀態 ---
export const dragDrop = $state({
	isDraggingOver: false,
	activeDropZone: null as DropZoneType
});

// --- 格式設定 ---
export const format = $state({
	outputTarget: 'yolo' as OutputFormat,
	annotationType: 'bbox' as AnnotationType,
	labelmeOutputFormat: 'original' as LabelMeOutputFormat
});

// --- 資料集分割 ---
export const split = $state({
	trainRatio: 70,
	valRatio: 20,
	testRatio: 10
});

// 合併的 splitRatio（getter 函數，因為不能 export $derived）
export function getSplitRatio(): SplitRatio {
	return {
		train: split.trainRatio,
		val: split.valRatio,
		test: split.testRatio
	};
}

// --- 標籤管理 ---
export const labels = $state({
	useCustomLabels: false,
	includeEmptyLabelImages: true,
	labelList: [] as LabelInfo[],
	isScanning: false,
	labelScanMessage: '',
	isCalculatingCounts: false
});

// --- 格式檢測 ---
export const detection = $state({
	detectedFormat: null as DatasetAnalysis | null,
	isDetectingFormat: false
});

// --- 進階選項 ---
export const advanced = $state({
	showAdvanced: false,
	workerCount: 0,
	randomSeed: 42,
	removeImageData: true
});

// 合併的 advancedOptions（getter 函數）
export function getAdvancedOptions(): AdvancedOptions {
	return {
		includeBackground: labels.includeEmptyLabelImages,
		workerCount: advanced.workerCount,
		randomSeed: advanced.randomSeed
	};
}

// --- 執行狀態 ---
export const execution = $state({
	isProcessing: false,
	progress: 0,
	statusMessage: '',
	stats: {
		total: 0,
		processed: 0,
		success: 0,
		skipped: 0,
		failed: 0
	} as ProcessingStats,
	detailedStats: {
		totalAnnotations: 0,
		skippedAnnotations: 0,
		backgroundImages: 0,
		backgroundFiles: [],
		filteredEmptyImages: 0,
		filteredEmptyFiles: [],
		skippedLabels: [],
		invalidAnnotations: []
	} as DetailedStats,
	showInvalidDetails: false
});

// ===== Getter Functions (替代 derived stores) =====

/** 是否需要顯示資料集分割（僅訓練格式需要） */
export function getShowSplitRatio(): boolean {
	return format.outputTarget !== 'labelme';
}

/** 生成預設資料夾名稱 */
export function getDefaultDatasetName(): string {
	if (!paths.sourceDir) return '';
	const sourceName = paths.sourceDir.split(/[\\/]/).pop() || 'dataset';
	const now = new Date();
	const datetime = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}_${String(now.getHours()).padStart(2, '0')}${String(now.getMinutes()).padStart(2, '0')}${String(now.getSeconds()).padStart(2, '0')}`;
	return `${sourceName}_${format.outputTarget}_${format.annotationType}_${datetime}`;
}

/** 已選取的標籤數量 */
export function getSelectedLabelCount(): number {
	return labels.labelList.filter((l) => l.selected).length;
}

/** 無效標註按原因分組 */
export function getInvalidReasonGroups(): Record<string, InvalidAnnotation[]> {
	return execution.detailedStats.invalidAnnotations.reduce(
		(acc, item) => {
			if (!acc[item.reason]) acc[item.reason] = [];
			acc[item.reason].push(item);
			return acc;
		},
		{} as Record<string, InvalidAnnotation[]>
	);
}

/** 是否可以開始轉換 */
export function getCanStartExport(): boolean {
	return !!paths.sourceDir && !execution.isProcessing;
}

// ===== Actions =====

/** 調整分割比例，確保總和為 100 */
export function adjustSplitRatios(changed: 'train' | 'val' | 'test') {
	const train = split.trainRatio;
	const val = split.valRatio;
	const test = split.testRatio;

	if (changed === 'train') {
		const remaining = 100 - train;
		const otherTotal = val + test;
		if (otherTotal === 0) {
			split.valRatio = remaining;
			split.testRatio = 0;
		} else {
			const scale = remaining / otherTotal;
			split.valRatio = Math.max(0, Math.round(val * scale));
			split.testRatio = Math.max(0, 100 - train - split.valRatio);
		}
	} else if (changed === 'val') {
		const remaining = 100 - val;
		const otherTotal = train + test;
		if (otherTotal === 0) {
			split.trainRatio = remaining;
			split.testRatio = 0;
		} else {
			const scale = remaining / otherTotal;
			split.trainRatio = Math.max(0, Math.round(train * scale));
			split.testRatio = Math.max(0, 100 - split.trainRatio - val);
		}
	} else {
		const remaining = 100 - test;
		const otherTotal = train + val;
		if (otherTotal === 0) {
			split.trainRatio = remaining;
			split.valRatio = 0;
		} else {
			const scale = remaining / otherTotal;
			split.trainRatio = Math.max(0, Math.round(train * scale));
			split.valRatio = Math.max(0, 100 - split.trainRatio - test);
		}
	}
}

/** 切換標籤選取狀態 */
export function toggleLabel(id: number) {
	labels.labelList = labels.labelList.map((l) => (l.id === id ? { ...l, selected: !l.selected } : l));
}

/** 全選標籤 */
export function selectAllLabels() {
	labels.labelList = labels.labelList.map((l) => ({ ...l, selected: true }));
}

/** 取消全選 */
export function deselectAllLabels() {
	labels.labelList = labels.labelList.map((l) => ({ ...l, selected: false }));
}

/** 重置執行狀態 */
export function resetExportState() {
	execution.progress = 0;
	execution.stats = { total: 0, processed: 0, success: 0, skipped: 0, failed: 0 };
	execution.detailedStats = {
		totalAnnotations: 0,
		skippedAnnotations: 0,
		backgroundImages: 0,
		backgroundFiles: [],
		filteredEmptyImages: 0,
		filteredEmptyFiles: [],
		skippedLabels: [],
		invalidAnnotations: []
	};
	execution.showInvalidDetails = false;
	execution.statusMessage = '';
}

/** 取得標籤 ID 映射表（順序就是 class ID） */
export function getLabelIdMapping(): Record<string, number> {
	const list = labels.labelList;
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
	const list = labels.labelList;
	const useCustom = labels.useCustomLabels;

	if (!useCustom || list.length === 0) {
		return [];
	}

	return list.filter((l) => l.selected).map((l) => l.name);
}

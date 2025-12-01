/**
 * Export Service - 封裝 Turbo Export 後端呼叫邏輯
 *
 * 這個服務層負責：
 * 1. 定義請求/回應的 TypeScript 介面
 * 2. 封裝 Tauri invoke 呼叫
 * 3. 提供型別安全的 API
 */

import { invoke } from '@tauri-apps/api/core';

// ===== 型別定義 =====

/** 支援的輸出格式 */
export type OutputFormat = 'yolo' | 'coco' | 'labelme';

/** 標註類型 */
export type AnnotationType = 'bbox' | 'polygon';

/** 分割模式 */
export type SegmentationMode = 'polygon' | 'bbox_only';

/** LabelMe 輸出點格式（僅 LabelMe → LabelMe 時使用） */
export type LabelMeOutputFormat = 'original' | 'bbox_2point' | 'bbox_4point';

/** 無效標註記錄 */
export interface InvalidAnnotation {
	file: string;
	label: string;
	reason: string;
	shape_type: string;
	points_count: number;
}

/** 轉換統計結果 */
export interface ConversionStats {
	total_files: number;
	processed_files: number;
	skipped_files: number;
	failed_files: number;
	total_annotations: number;
	skipped_annotations: number;
	/** 背景圖片數量（原本就沒有 JSON 標註檔的圖片） */
	background_images: number;
	/** 背景圖片檔名列表（限制前 100 筆） */
	background_files: string[];
	/** 因標籤篩選而變空的圖片數量 */
	filtered_empty_images: number;
	/** 因標籤篩選而變空的圖片檔名列表（限制前 100 筆） */
	filtered_empty_files: string[];
	labels_found: string[];
	skipped_labels: string[];
	invalid_annotations: InvalidAnnotation[];
}

/** 轉換請求參數 */
export interface ConvertLabelMeRequest {
	/** 來源目錄 */
	input_dir: string;
	/** 輸出目錄（null 則自動產生） */
	output_dir: string | null;
	/** 輸出格式 */
	output_format: OutputFormat;
	/** 標註格式（bbox 或 polygon） */
	annotation_format: AnnotationType;
	/** 驗證集比例 (0-1) */
	val_size: number;
	/** 測試集比例 (0-1) */
	test_size: number;
	/** 隨機種子 */
	seed: number;
	/** 是否包含背景圖片 */
	include_background: boolean;
	/** 標籤列表（空陣列表示全部） */
	label_list: string[];
	/** 是否使用自訂標籤順序 */
	deterministic_labels: boolean;
	/** 分割模式 */
	segmentation_mode: SegmentationMode;
	/** 自訂資料集名稱 */
	custom_dataset_name: string | null;
	/** 移除 imageData（僅 LabelMe 輸出格式） */
	remove_image_data: boolean;
	/** LabelMe 輸出點格式（僅 LabelMe 輸出格式） */
	labelme_output_format: LabelMeOutputFormat;
}

/** 轉換回應結果 */
export interface ConvertLabelMeResponse {
	success: boolean;
	output_dir: string;
	stats: ConversionStats;
	errors: string[];
}

/** 標籤數量統計 */
export type LabelCounts = Record<string, number>;

// ===== API 函數 =====

/**
 * 掃描 LabelMe 資料集中的標籤（快速版，僅回傳名稱）
 * @param inputDir 來源目錄路徑
 * @returns 標籤名稱陣列
 */
export async function scanLabels(inputDir: string): Promise<string[]> {
	return invoke<string[]>('scan_labelme_labels', {
		inputDir
	});
}

/**
 * 掃描 LabelMe 資料集中的標籤（完整版，包含數量統計）
 * @param inputDir 來源目錄路徑
 * @returns 標籤名稱到數量的映射
 */
export async function scanLabelsWithCounts(inputDir: string): Promise<LabelCounts> {
	return invoke<LabelCounts>('scan_labelme_labels_with_counts', {
		inputDir
	});
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

/**
 * 分析 LabelMe 資料集以檢測輸入格式
 * @param inputDir 來源目錄路徑
 * @returns 資料集分析結果
 */
export async function analyzeLabelMeDataset(inputDir: string): Promise<DatasetAnalysis> {
	return invoke<DatasetAnalysis>('analyze_labelme_dataset', {
		inputDir
	});
}


/**
 * 執行 LabelMe 格式轉換
 * @param request 轉換請求參數
 * @returns 轉換結果
 */
export async function convertLabelMe(request: ConvertLabelMeRequest): Promise<ConvertLabelMeResponse> {
	return invoke<ConvertLabelMeResponse>('convert_labelme', {
		request
	});
}

// ===== 便利函數 =====

/**
 * 建立預設的轉換請求參數
 * @param inputDir 來源目錄
 * @param outputFormat 輸出格式
 * @returns 預設參數物件
 */
export function createDefaultRequest(
	inputDir: string,
	outputFormat: OutputFormat = 'yolo'
): ConvertLabelMeRequest {
	return {
		input_dir: inputDir,
		output_dir: null,
		output_format: outputFormat,
		annotation_format: 'bbox',
		val_size: 0.2,
		test_size: 0.1,
		seed: 42,
		include_background: false,
		label_list: [],
		deterministic_labels: false,
		segmentation_mode: 'bbox_only',
		custom_dataset_name: null,
		remove_image_data: true,
		labelme_output_format: 'original'
	};
}

/**
 * 計算總標註數量
 * @param counts 標籤數量映射
 * @returns 總數
 */
export function getTotalAnnotationCount(counts: LabelCounts): number {
	return Object.values(counts).reduce((sum, count) => sum + count, 0);
}

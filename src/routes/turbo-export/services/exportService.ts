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
	background_images: number;
	/** 背景圖片檔名列表（限制前 100 筆） */
	background_files: string[];
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
		remove_image_data: true
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

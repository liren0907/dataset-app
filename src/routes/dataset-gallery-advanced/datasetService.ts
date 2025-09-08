import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";

// Interface for the structure of an image object from the backend (basic version)
interface BackendImage {
    path: string;
    name: string;
    size?: number;
    dimensions?: { width: number; height: number };
    created?: string; // Assuming it's a string, might need Date parsing later
    has_json?: boolean; // LabelMe JSON existence
    annotations?: any[]; // Annotations from LabelMe JSON
    // Add other fields as returned by your backend
}

// Interface for the processed image object used in the frontend
export interface ProcessedImage extends BackendImage {
    previewUrl: string;
    isLoaded?: boolean; // Optional: for lazy loading state if managed here
    displayIndex?: number; // Optional: for certain UI logic
    annotated?: boolean; // Derived from annotations presence
    // Ensure dimensions and other details from get_image_details can be stored
    dimensions?: { width: number; height: number };
}

// Interface for the paginated response from the backend
interface PaginatedImageResponse {
    images: BackendImage[];
    total: number;
    total_pages: number;
    // current_page: number; // Backend might also return this
}

// Interface for the result of fetching paginated images
export interface FetchPaginatedImagesResult {
    processedImages: ProcessedImage[];
    totalImages: number;
    totalPages: number;
}

// Helper function to process images with smart loading strategy
// First page images get displayIndex < pageSize for immediate loading
function processImagesForService(backendImages: BackendImage[], isFirstPageLoad: boolean, existingImagesLength: number = 0, pageSize: number = 30): ProcessedImage[] {
    return backendImages.map((img, index) => {
        const actualIndex = isFirstPageLoad ? index : existingImagesLength + index;
        return {
            ...img,
            previewUrl: convertFileSrc(img.path), // Generate preview URL
            isLoaded: false, // Default for lazy loading
            displayIndex: actualIndex, // Critical: used for smart loading decision
            annotated: !!(img.annotations && img.annotations.length > 0 && img.has_json) // Simplified logic
        };
    });
}

/**
 * Fetches a paginated list of images from the backend and processes them.
 * @param path The directory path.
 * @param page The page number to fetch.
 * @param pageSize The number of images per page.
 * @returns A promise that resolves to an object containing processed images, total images, and total pages.
 * @throws Will throw an error if the backend call fails or returns invalid data.
 */
export async function fetchPaginatedImages(
    path: string,
    page: number,
    pageSize: number
): Promise<FetchPaginatedImagesResult> {
    console.log(
        `Service: Fetching page ${page}, size ${pageSize} from ${path}`
    );

    if (!path || !path.trim()) {
        throw new Error("Directory path cannot be empty.");
    }

    try {
        const resultStr = await invoke("get_paginated_images", {
            path: path,
            page: page,
            pageSize: pageSize,
        }) as string | null;

        if (!resultStr) {
            console.error("Service: Empty result received from get_paginated_images");
            throw new Error("No data received from image service.");
        }

        const data = JSON.parse(resultStr) as PaginatedImageResponse;

        if (!data || !Array.isArray(data.images)) {
            console.error("Service: Invalid data structure received", data);
            throw new Error("Invalid data structure for images.");
        }

        console.log(
            `Service: Received ${data.images.length} images, total: ${data.total}, total_pages: ${data.total_pages}`
        );

        // Process images with smart loading strategy
        // Pass pageSize to enable smart loading decisions in processImagesForService
        const processedImages = processImagesForService(data.images, page === 1, 0, pageSize);

        return {
            processedImages,
            totalImages: data.total,
            totalPages: data.total_pages,
        };
    } catch (err) {
        console.error("Service: Error in fetchPaginatedImages:", err);
        // Re-throw a more generic error or the original, depending on desired error handling strategy
        const errorMessage = err instanceof Error ? err.message : String(err);
        throw new Error(`Failed to fetch images from service: ${errorMessage}`);
    }
}

// Interface for the detailed image data (can be partial, as we merge it)
// We expect it to at least provide dimensions, but it might provide more.
// It could potentially return a full BackendImage structure or a subset of its fields.
// For now, let's assume it returns fields that can be part of BackendImage.
export interface ImageDetailData extends Partial<BackendImage> {
    // Explicitly expect dimensions, other fields are optional from BackendImage
    dimensions: { width: number; height: number };
}

/**
 * Fetches detailed information for a single image from the backend.
 * @param imagePath The path to the image.
 * @returns A promise that resolves to the detailed image data.
 * @throws Will throw an error if the backend call fails or returns invalid data.
 */
export async function fetchImageDetails(imagePath: string): Promise<ImageDetailData> {
    console.log(`Service: Fetching details for image: ${imagePath}`);
    if (!imagePath || !imagePath.trim()) {
        throw new Error("Image path cannot be empty for fetching details.");
    }

    try {
        const resultStr = await invoke("get_image_details", {
            path: imagePath
        }) as string | null;

        if (!resultStr) {
            console.error(`Service: Empty result received from get_image_details for ${imagePath}`);
            throw new Error(`No detailed data received for image: ${imagePath}`);
        }

        const detailedData = JSON.parse(resultStr) as ImageDetailData;

        if (!detailedData || !detailedData.dimensions) {
            console.error("Service: Invalid or incomplete detail data received", detailedData);
            throw new Error("Invalid or incomplete detailed data for image (missing dimensions).");
        }

        console.log(`Service: Received details for ${imagePath}:`, detailedData);
        return detailedData;
    } catch (err) {
        console.error(`Service: Error in fetchImageDetails for ${imagePath}:`, err);
        const errorMessage = err instanceof Error ? err.message : String(err);
        throw new Error(`Failed to fetch image details from service: ${errorMessage}`);
    }
}

// Interface for the Dataset Summary
export interface DatasetSummary {
    total_images: number;
    images_with_annotations: number;
    total_annotations: number;
    unique_labels: number;
    label_counts: { [key: string]: number };
    annotation_types: string[]; // e.g., ["rectangle", "polygon"]
    // Add any other fields your backend returns for the summary
}

/**
 * Fetches the LabelMe dataset summary from the backend.
 * @param path The directory path.
 * @returns A promise that resolves to the dataset summary object.
 * @throws Will throw an error if the backend call fails or returns invalid data.
 */
export async function fetchDatasetSummary(path: string): Promise<DatasetSummary> {
    console.log(`Service: Fetching dataset summary for path: ${path}`);
    if (!path || !path.trim()) {
        throw new Error("Directory path cannot be empty for fetching dataset summary.");
    }

    try {
        const resultStr = await invoke("get_labelme_summary", {
            path: path
        }) as string | null;

        if (!resultStr) {
            console.error(`Service: Empty result received from get_labelme_summary for ${path}`);
            throw new Error(`No summary data received for path: ${path}`);
        }

        const summaryData = JSON.parse(resultStr) as DatasetSummary;

        // Basic validation of the summary data structure
        if (!summaryData || typeof summaryData.total_images !== 'number') {
            console.error("Service: Invalid or incomplete summary data received", summaryData);
            throw new Error("Invalid or incomplete summary data received from service.");
        }

        console.log(`Service: Received dataset summary for ${path}:`, summaryData);
        return summaryData;
    } catch (err) {
        console.error(`Service: Error in fetchDatasetSummary for ${path}:`, err);
        const errorMessage = err instanceof Error ? err.message : String(err);
        throw new Error(`Failed to fetch dataset summary from service: ${errorMessage}`);
    }
}

// Interface for the response from the auto_annotate_images backend call
// This might return a list of images with updated annotation information.
// Assuming it returns objects similar to BackendImage but potentially with focused annotation updates.
export interface AnnotatedImageInfo extends Partial<BackendImage> {
    path: string; // Ensure path is always present for matching
    annotations?: any[];
    has_json?: boolean;
}

export interface AutoAnnotationResult {
    annotated_images: AnnotatedImageInfo[];
    // Potentially other summary info from the annotation process if backend provides it
}

/**
 * Calls the backend to perform auto-annotation on images in a directory.
 * @param path The directory path.
 * @param page The current page (if applicable to the backend logic, might be for updating currently viewed images).
 * @param pageSize The page size (similarly, if applicable).
 * @returns A promise that resolves to the annotation results containing all annotation types.
 * @throws Will throw an error if the backend call fails or returns invalid data.
 */
export async function performAutoAnnotation(
    path: string,
    page: number,
    pageSize: number
): Promise<AutoAnnotationResult> {
    console.log(
        `Service: Performing auto-annotation (all types) for path: ${path}, page: ${page}, pageSize: ${pageSize}`
    );

    if (!path || !path.trim()) {
        throw new Error("Directory path cannot be empty for auto-annotation.");
    }

    try {
        const resultStr = await invoke("auto_annotate_images", {
            path: path,
            page: page,
            pageSize: pageSize,
        }) as string | null;

        if (!resultStr) {
            console.error(`Service: Empty result received from auto_annotate_images for ${path}`);
            throw new Error(`No annotation data received for path: ${path}`);
        }

        const annotationData = JSON.parse(resultStr) as AutoAnnotationResult;

        if (!annotationData || !Array.isArray(annotationData.annotated_images)) {
            console.error("Service: Invalid or incomplete annotation data received", annotationData);
            throw new Error("Invalid or incomplete annotation data structure received from service.");
        }

        console.log(`Service: Received annotation results for ${path}:`, annotationData.annotated_images.length, "images affected.");
        return annotationData;
    } catch (err) {
        console.error(`Service: Error in performAutoAnnotation for ${path}:`, err);
        const errorMessage = err instanceof Error ? err.message : String(err);
        throw new Error(`Failed to perform auto-annotation via service: ${errorMessage}`);
    }
}

/**
 * Calls the backend to perform crop and remap annotations.
 * @param sourceDir The source directory containing images and annotations.
 * @param outputDir The directory where cropped images and remapped annotations will be saved.
 * @param parentLabel The label of the parent object to crop around.
 * @returns A promise that resolves to a success message string from the backend.
 * @throws Will throw an error if the backend call fails.
 */
export async function performCropAndRemap(
    sourceDir: string,
    outputDir: string,
    parentLabel: string
): Promise<string> {
    console.log(
        `Service: Performing crop and remap. Source: ${sourceDir}, Output: ${outputDir}, Parent: ${parentLabel}`
    );

    if (!sourceDir || !sourceDir.trim()) {
        throw new Error("Source directory cannot be empty for crop & remap.");
    }
    if (!outputDir || !outputDir.trim()) {
        throw new Error("Output directory cannot be empty for crop & remap.");
    }
    if (!parentLabel || !parentLabel.trim()) {
        throw new Error("Parent label cannot be empty for crop & remap.");
    }

    try {
        const message = await invoke("crop_and_remap_annotations", {
            sourceDir: sourceDir,
            outputDir: outputDir,
            parentLabel: parentLabel
        }) as string;

        console.log(`Service: Crop and remap successful. Message: ${message}`);
        return message;
    } catch (err) {
        console.error(`Service: Error in performCropAndRemap:`, err);
        const errorMessage = err instanceof Error ? err.message : String(err);
        throw new Error(`Failed to perform crop and remap via service: ${errorMessage}`);
    }
}

// Interface for the parameters of the dataset export function
export interface DatasetExportParams {
    sourceDir: string;
    outputDir: string;
    mode: 'yolo' | 'labelme';
    trainRatio?: number; // Optional for labelme mode
    valRatio?: number;   // Optional for labelme mode
    testRatio?: number;  // Optional for labelme mode
    shapeType?: "polygon" | "bounding_box"; // Optional for labelme mode
    includedLabels: string[]; // Array of label names to include
}

/**
 * Calls the backend to perform dataset export (either YOLO or LabelMe extraction).
 * @param params The export parameters.
 * @returns A promise that resolves to a success message string from the backend.
 * @throws Will throw an error if the backend call fails or parameters are invalid for the mode.
 */
export async function performDatasetExport(params: DatasetExportParams): Promise<string> {
    const {
        sourceDir,
        outputDir,
        mode,
        trainRatio,
        valRatio,
        testRatio,
        shapeType,
        includedLabels
    } = params;

    console.log(`Service: Performing dataset export. Mode: ${mode}, Source: ${sourceDir}, Output: ${outputDir}`);

    if (!sourceDir || !sourceDir.trim()) throw new Error("Source directory is required for export.");
    if (!outputDir || !outputDir.trim()) throw new Error("Output directory is required for export.");
    if (!includedLabels || includedLabels.length === 0) throw new Error("At least one label must be included for export.");

    const includedLabelsStr = includedLabels.join(',');

    try {
        let resultMessage: string;
        if (mode === 'yolo') {
            if (trainRatio === undefined || valRatio === undefined || testRatio === undefined || shapeType === undefined) {
                throw new Error("Missing YOLO specific parameters (ratios, shapeType) for export.");
            }
            // Basic validation for ratios sum (more thorough validation might be in UI or backend)
            const sum = trainRatio + valRatio + testRatio;
            if (Math.abs(sum - 1.0) > 0.015) {
                throw new Error(`YOLO split ratios must sum to 1.0. Current sum: ${sum.toFixed(2)}`);
            }

            console.log("Service: Exporting to YOLO with params:", params);
            resultMessage = await invoke("export_to_yolo_new", {
                sourceDir,
                outputDir,
                trainRatio,
                valRatio,
                testRatio,
                shape: shapeType,
                specificLabels: true,
                classNamesStr: includedLabelsStr,
            }) as string;

        } else if (mode === 'labelme') {
            console.log("Service: Exporting LabelMe (Extracting Labels) with params:", params);
            resultMessage = await invoke("extract_labels", {
                sourceDir,
                outputDir,
                labelsStr: includedLabelsStr,
            }) as string;
        } else {
            throw new Error(`Unsupported export mode: ${mode}`);
        }

        console.log(`Service: Dataset export successful. Message: ${resultMessage}`);
        return resultMessage;

    } catch (err) {
        console.error(`Service: Error in performDatasetExport (mode: ${mode}):`, err);
        const errorMessage = err instanceof Error ? err.message : String(err);
        throw new Error(`Failed to perform dataset export via service: ${errorMessage}`);
    }
}

// Local datasetService functions for dataset-gallery-advanced
// Add advanced-specific features here for rapid development
export async function advancedDatasetAnalysis(path: string) {
    // Placeholder for advanced dataset analysis
    console.log(`Advanced: Analyzing dataset at ${path}`);
    // Add your advanced analysis logic here
}

export async function advancedImageProcessing(path: string) {
    // Placeholder for advanced image processing
    console.log(`Advanced: Processing image at ${path}`);
    // Add your advanced processing logic here
}

import Konva from 'konva';
import { convertFileSrc } from "@tauri-apps/api/core";

export interface KonvaState {
    scale: number;
    stageX: number;
    stageY: number;
    selectedAnnotationCount: number;
    totalAnnotations: number;
}

export type StateCallback = (state: Partial<KonvaState>) => void;

export class KonvaManager {
    private stage: Konva.Stage | null = null;
    private mainLayer: Konva.Layer | null = null;
    private annotationLayer: Konva.Layer | null = null;
    private uiLayer: Konva.Layer | null = null;
    private imageNode: Konva.Image | null = null;
    private transformer: Konva.Transformer | null = null;
    private annotationGroups: Konva.Group[] = [];
    private selectedAnnotation: any = null; // Group or Group[]

    private container: HTMLDivElement | null = null;
    private onStateChange: StateCallback | null = null;

    constructor(onStateChange?: StateCallback) {
        if (onStateChange) this.onStateChange = onStateChange;
    }

    public initialize(container: HTMLDivElement, image: any) {
        this.container = container;
        console.log('initializeKonvaStage called with:', image);

        const containerRect = container.getBoundingClientRect();

        // Clean up existing stage
        this.destroy();

        const stageWidth = Math.max(containerRect.width || 800, 800);
        const stageHeight = Math.max(containerRect.height || 600, 600);

        this.stage = new Konva.Stage({
            container: container,
            width: stageWidth,
            height: stageHeight,
            draggable: true,
        });

        this.mainLayer = new Konva.Layer();
        this.annotationLayer = new Konva.Layer();
        this.uiLayer = new Konva.Layer();

        this.stage.add(this.mainLayer);
        this.stage.add(this.annotationLayer);
        this.stage.add(this.uiLayer);

        this.transformer = new Konva.Transformer({
            rotateEnabled: true,
            borderEnabled: true,
            borderStroke: '#4A90E2',
            borderStrokeWidth: 2,
            anchorFill: '#4A90E2',
            anchorStroke: '#FFFFFF',
            anchorStrokeWidth: 2,
            anchorSize: 8,
        });
        this.uiLayer.add(this.transformer);

        this.setupZoomControls();
        this.setupStageEvents();

        this.loadImageWithAnnotations(image);
    }

    public destroy() {
        if (this.stage) {
            this.stage.destroy();
            this.stage = null;
            this.mainLayer = null;
            this.annotationLayer = null;
            this.uiLayer = null;
            this.imageNode = null;
            this.transformer = null;
            this.annotationGroups = [];
            this.selectedAnnotation = null;
        }
    }

    private updateState(partialState: Partial<KonvaState>) {
        if (this.onStateChange) {
            this.onStateChange(partialState);
        }
    }

    private setupZoomControls() {
        if (!this.stage) return;

        this.stage.on('wheel', (e) => {
            e.evt.preventDefault();
            if (!this.stage) return;

            const pointer = this.stage.getPointerPosition();
            if (!pointer) return;

            const oldScale = this.stage.scaleX();
            const newScale = e.evt.deltaY > 0 ? oldScale * 0.9 : oldScale * 1.1;

            // Limit zoom
            const clampedScale = Math.max(0.1, Math.min(5, newScale));

            const mousePointTo = {
                x: (pointer.x - this.stage.x()) / oldScale,
                y: (pointer.y - this.stage.y()) / oldScale,
            };

            this.stage.scale({ x: clampedScale, y: clampedScale });

            const newPos = {
                x: pointer.x - mousePointTo.x * clampedScale,
                y: pointer.y - mousePointTo.y * clampedScale,
            };

            this.stage.position(newPos);
            this.stage.batchDraw();

            this.updateState({
                scale: clampedScale,
                stageX: newPos.x,
                stageY: newPos.y
            });
        });
    }

    private setupStageEvents() {
        if (!this.stage) return;

        this.stage.on('click tap', (e) => {
            if (e.target === this.stage) {
                this.deselectAnnotation();
            } else if (e.target.getParent() && this.annotationGroups.includes(e.target.getParent() as Konva.Group)) {
                this.selectAnnotation(e.target.getParent());
            }
        });

        this.stage.on('dragstart', (e) => {
            if (this.annotationGroups.includes(e.target.getParent() as Konva.Group)) {
                e.cancelBubble = true;
            }
        });
    }

    private loadImageWithAnnotations(image: any) {
        if (!this.stage || !this.mainLayer) return;

        const imageObj = new Image();
        imageObj.crossOrigin = 'anonymous';

        imageObj.onload = async () => {
            const scaleX = this.stage!.width() / imageObj.width;
            const scaleY = this.stage!.height() / imageObj.height;
            const fitScale = Math.min(scaleX, scaleY, 1);

            const scaledWidth = imageObj.width * fitScale;
            const scaledHeight = imageObj.height * fitScale;

            const x = (this.stage!.width() - scaledWidth) / 2;
            const y = (this.stage!.height() - scaledHeight) / 2;

            this.imageNode = new Konva.Image({
                x: x,
                y: y,
                image: imageObj,
                width: scaledWidth,
                height: scaledHeight,
            });

            this.mainLayer!.add(this.imageNode);
            this.mainLayer!.draw();

            if (image.annotations && image.annotations.length > 0) {
                this.drawAnnotationsOnCanvas(image.annotations, fitScale, x, y);
            }

            this.stage!.batchDraw();
        };

        imageObj.src = convertFileSrc(image.path);
    }

    private drawAnnotationsOnCanvas(annotations: any[], scale: number, offsetX: number, offsetY: number) {
        if (!this.annotationLayer || !this.uiLayer) return;

        this.annotationLayer.destroyChildren();
        this.annotationGroups = [];

        annotations.forEach((annotation: any, index: number) => {
            const annotationGroup = this.drawAnnotationShape(annotation, scale, offsetX, offsetY, index);
            if (annotationGroup) {
                this.annotationGroups.push(annotationGroup);
            }
        });

        this.annotationLayer.draw();
        this.uiLayer.draw();

        this.updateState({
            totalAnnotations: this.annotationGroups.length,
            selectedAnnotationCount: 0
        });
    }

    private drawAnnotationShape(annotation: any, scale: number, offsetX: number, offsetY: number, index: number) {
        if (!this.annotationLayer || !this.uiLayer) return null;

        const colors = [
            { fill: '#FF6B6B', stroke: '#E53E3E', text: '#FFFFFF' },
            { fill: '#4ECDC4', stroke: '#319795', text: '#FFFFFF' },
            { fill: '#45B7D1', stroke: '#3182CE', text: '#FFFFFF' },
            { fill: '#96CEB4', stroke: '#38A169', text: '#FFFFFF' },
            { fill: '#FFEAA7', stroke: '#D69E2E', text: '#000000' },
            { fill: '#DDA0DD', stroke: '#9F7AEA', text: '#FFFFFF' },
            { fill: '#98D8C8', stroke: '#4FD1C9', text: '#000000' },
        ];
        const colorScheme = colors[index % colors.length];

        try {
            const annotationGroup = new Konva.Group({
                draggable: true,
                name: `annotation-${index}`,
            });

            let shape;
            let bounds;

            // Simplified shape creation logic for brevity, reusing the core logic
            if ((annotation.shape_type === 'rectangle' || annotation.shape_type === 'bounding_box') && annotation.points?.length >= 2) {
                const x1 = annotation.points[0][0] * scale + offsetX;
                const y1 = annotation.points[0][1] * scale + offsetY;
                const x2 = annotation.points[1][0] * scale + offsetX;
                const y2 = annotation.points[1][1] * scale + offsetY;

                bounds = { x: Math.min(x1, x2), y: Math.min(y1, y2), width: Math.abs(x2 - x1), height: Math.abs(y2 - y1) };

                shape = new Konva.Rect({
                    ...bounds,
                    fill: colorScheme.fill,
                    stroke: colorScheme.stroke,
                    strokeWidth: 2,
                    opacity: 0.25,
                });
            } else if (annotation.shape_type === 'polygon' && annotation.points?.length > 2) {
                const points: number[] = [];
                annotation.points.forEach((point: number[]) => {
                    points.push(point[0] * scale + offsetX);
                    points.push(point[1] * scale + offsetY);
                });
                // Simplified bounds calculation
                const xs = annotation.points.map((p: number[]) => p[0] * scale + offsetX);
                const ys = annotation.points.map((p: number[]) => p[1] * scale + offsetY);
                bounds = { x: Math.min(...xs), y: Math.min(...ys), width: Math.max(...xs) - Math.min(...xs), height: Math.max(...ys) - Math.min(...ys) };

                shape = new Konva.Line({
                    points: points,
                    fill: colorScheme.fill,
                    stroke: colorScheme.stroke,
                    strokeWidth: 2,
                    opacity: 0.25,
                    closed: true,
                });
            }

            if (shape && bounds) {
                annotationGroup.add(shape);

                // Label logic
                if (annotation.label) {
                    const labelBg = new Konva.Rect({
                        x: bounds.x, y: bounds.y - 25,
                        width: annotation.label.length * 8 + 10, height: 20,
                        fill: colorScheme.stroke, opacity: 0.8, cornerRadius: 4,
                    });
                    const label = new Konva.Text({
                        x: bounds.x + 5, y: bounds.y - 23,
                        text: annotation.label,
                        fontSize: 12, fontFamily: 'Arial', fontStyle: 'bold', fill: colorScheme.text,
                    });
                    annotationGroup.add(labelBg);
                    annotationGroup.add(label);
                }

                // Tooltip logic (simplified)
                this.setupTooltip(annotationGroup, annotation, bounds, this.uiLayer, this.stage!);

                // Selection logic
                annotationGroup.on('click tap', (e) => {
                    e.cancelBubble = true;
                    this.selectAnnotation(annotationGroup);
                });

                this.annotationLayer!.add(annotationGroup);
                return annotationGroup;
            }
        } catch (error) {
            console.error('Error drawing annotation:', error);
        }
        return null;
    }

    private setupTooltip(group: Konva.Group, annotation: any, bounds: any, uiLayer: Konva.Layer, stage: Konva.Stage) {
        const tooltip = new Konva.Text({
            text: `${annotation.label || 'Unknown'}\nType: ${annotation.shape_type}\nConfidence: ${annotation.confidence || 'N/A'}`,
            fontSize: 12, fontFamily: 'Arial', fill: '#000000', padding: 8, visible: false, opacity: 0.9,
        });
        const tooltipBg = new Konva.Rect({
            fill: '#FFFFFF', stroke: '#CCCCCC', strokeWidth: 1, cornerRadius: 4, shadowBlur: 8, shadowOpacity: 0.2, visible: false
        });
        uiLayer.add(tooltipBg);
        uiLayer.add(tooltip);

        group.on('mouseenter', () => {
            if (document.body) document.body.style.cursor = 'pointer';
            const mousePos = stage.getPointerPosition();
            if (mousePos) {
                tooltip.position({ x: mousePos.x + 10, y: mousePos.y - 10 });
                tooltipBg.position({ x: mousePos.x + 5, y: mousePos.y - 15 });
                tooltipBg.width(tooltip.width() + 16);
                tooltipBg.height(tooltip.height() + 16);
                tooltip.visible(true);
                tooltipBg.visible(true);
                uiLayer.draw();
            }
        });
        group.on('mouseleave', () => {
            if (document.body) document.body.style.cursor = 'default';
            tooltip.visible(false);
            tooltipBg.visible(false);
            uiLayer.draw();
        });
    }

    public selectAnnotation(annotationGroup: any) {
        if (!this.transformer) return;
        this.selectedAnnotation = annotationGroup;
        this.transformer.nodes([annotationGroup]);
        this.uiLayer!.draw();

        this.updateState({ selectedAnnotationCount: 1 });
    }

    public deselectAnnotation() {
        if (!this.transformer) return;
        this.selectedAnnotation = null;
        this.transformer.nodes([]);
        this.uiLayer!.draw();

        this.updateState({ selectedAnnotationCount: 0 });
    }

    public selectAllAnnotations() {
        if (!this.transformer || !this.stage) return;
        if (this.annotationGroups.length > 0) {
            this.selectedAnnotation = this.annotationGroups;
            this.transformer.nodes(this.annotationGroups);
            this.uiLayer!.draw();

            this.updateState({ selectedAnnotationCount: this.annotationGroups.length });
        }
    }

    public deleteSelectedAnnotation() {
        if (!this.selectedAnnotation || !this.annotationLayer) return;

        if (Array.isArray(this.selectedAnnotation)) {
            this.selectedAnnotation.forEach(group => group.destroy());
            this.annotationGroups = this.annotationGroups.filter(g => !this.selectedAnnotation.includes(g));
        } else {
            this.selectedAnnotation.destroy();
            this.annotationGroups = this.annotationGroups.filter(g => g !== this.selectedAnnotation);
        }

        this.deselectAnnotation();
        this.annotationLayer.draw();

        this.updateState({ totalAnnotations: this.annotationGroups.length });
    }

    public zoomIn() {
        if (!this.stage) return;
        const newScale = Math.min(5, this.stage.scaleX() * 1.2);
        this.zoomToScale(newScale);
    }

    public zoomOut() {
        if (!this.stage) return;
        const newScale = Math.max(0.1, this.stage.scaleX() * 0.8);
        this.zoomToScale(newScale);
    }

    public resetZoom() {
        this.zoomToScale(1);
        this.stage?.position({ x: 0, y: 0 });
        this.stage?.batchDraw();

        this.updateState({ scale: 1, stageX: 0, stageY: 0 });
    }

    public fitToScreen() {
        if (!this.stage || !this.imageNode || !this.container) return;
        const containerRect = this.container.getBoundingClientRect();
        const imageRect = this.imageNode.getClientRect(); // This might need original dims, but let's try

        // Actually best way to get original dims is from imageNode directly
        const imgWidth = this.imageNode.width();
        const imgHeight = this.imageNode.height();

        const scaleX = containerRect.width / imgWidth;
        const scaleY = containerRect.height / imgHeight;
        const newScale = Math.min(scaleX, scaleY, 1);

        this.zoomToScale(newScale);
        this.stage.position({ x: 0, y: 0 });

        this.updateState({ scale: newScale, stageX: 0, stageY: 0 });
    }

    private zoomToScale(newScale: number) {
        if (!this.stage) return;
        const center = { x: this.stage.width() / 2, y: this.stage.height() / 2 };
        const relatedTo = {
            x: (center.x - this.stage.x()) / this.stage.scaleX(),
            y: (center.y - this.stage.y()) / this.stage.scaleY(),
        };

        this.stage.scale({ x: newScale, y: newScale });
        const newPos = {
            x: center.x - relatedTo.x * newScale,
            y: center.y - relatedTo.y * newScale,
        };
        this.stage.position(newPos);
        this.stage.batchDraw();

        this.updateState({ scale: newScale, stageX: newPos.x, stageY: newPos.y });
    }
}

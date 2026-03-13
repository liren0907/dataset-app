import { fabric } from "fabric";

export type Mode = "select" | "bbox" | "polygon" | "polyline" | "keypoint" | "pan";

export interface BBox {
    id: number;
    label: string;
    x1: number;
    y1: number;
    x2: number;
    y2: number;
    locked?: boolean;
    hidden?: boolean;
}

export interface Polygon {
    id: number;
    label: string;
    points: { x: number; y: number }[];
    locked?: boolean;
    hidden?: boolean;
}

export interface Polyline {
    id: number;
    label: string;
    points: { x: number; y: number }[];
    locked?: boolean;
    hidden?: boolean;
}

export interface Keypoint {
    id: number;
    label: string;
    x: number;
    y: number;
    locked?: boolean;
    hidden?: boolean;
}

export interface LabelMeShape {
    label: string;
    points: number[][];
    shape_type: string;
}

export interface LabelMeJSON {
    shapes: LabelMeShape[];
    imageHeight: number;
    imageWidth: number;
}

export interface ImageEntry {
    name: string;
    path: string;
    hasJson: boolean;
}

type EventCallback = () => void;

interface HistoryState {
    bboxes: BBox[];
    polygons: Polygon[];
    polylines: Polyline[];
    keypoints: Keypoint[];
    nextBBoxId: number;
    nextPolygonId: number;
    nextPolylineId: number;
    nextKeypointId: number;
}

export class FabricManager {
    private canvas: fabric.Canvas | null = null;
    private imageScale = 1;
    private imageOffset = { x: 0, y: 0 };
    private mode: Mode = "select";
    private listeners: { [key: string]: EventCallback[] } = {};

    // Drawing State
    private isDrawing = false;
    private startPoint: fabric.Point | null = null;
    private currentRect: fabric.Rect | null = null;

    // Panning State
    private isPanning = false;
    private lastPosX = 0;
    private lastPosY = 0;

    // Polygon State
    private polygonPoints: { x: number; y: number }[] = [];
    private currentCanvasPolygonPoints: fabric.Point[] = [];
    private currentPolygonLine: fabric.Line | null = null;
    private currentPolygonPointsVisual: fabric.Circle[] = [];
    private currentPolygonSegments: fabric.Line[] = [];
    private isStartPointHighlighted = false;
    private readonly START_POINT_RADIUS = 3;
    private readonly START_POINT_HIGHLIGHT_RADIUS = 7;

    // Stroke width configuration
    private baseStrokeWidth = 2;
    private static readonly KEYPOINT_STROKE_RATIO = 0.75;
    private static readonly GUIDE_LINE_STROKE_RATIO = 0.5;

    // Selection
    private selectedObject: any = null;
    private selectedObjects: any[] = [];

    // Data
    private completedBBoxes: BBox[] = [];
    private completedPolygons: Polygon[] = [];
    private completedPolylines: Polyline[] = [];
    private completedKeypoints: Keypoint[] = [];
    private nextBBoxId = 1;
    private nextPolygonId = 1;
    private nextPolylineId = 1;
    private nextKeypointId = 1;

    // Polyline drawing state
    private polylinePoints: { x: number; y: number }[] = [];
    private currentCanvasPolylinePoints: fabric.Point[] = [];
    private currentPolylineSegments: fabric.Line[] = [];
    private currentPolylinePointsVisual: fabric.Circle[] = [];

    // Polygon Edit Mode
    private editingPolygonId: number | null = null;
    private editingPolygonObject: fabric.Polygon | null = null;
    private vertexHandles: fabric.Circle[] = [];

    // Undo / Redo
    private undoStack: HistoryState[] = [];
    private redoStack: HistoryState[] = [];
    private static readonly MAX_HISTORY = 50;

    constructor(private canvasEl: HTMLCanvasElement) {
        this.initCanvas();
    }

    private initCanvas() {
        this.canvas = new fabric.Canvas(this.canvasEl, {
            backgroundColor: "#f8fafc",
            selection: true,
        });

        this.canvas.on("mouse:down", this.handleMouseDown.bind(this));
        this.canvas.on("mouse:move", this.handleMouseMove.bind(this));
        this.canvas.on("mouse:up", this.handleMouseUp.bind(this));
        this.canvas.on("mouse:wheel", this.handleMouseWheel.bind(this));
        this.canvas.on("object:modified", this.handleObjectModified.bind(this));
        this.canvas.on("mouse:dblclick", this.handleDoubleClick.bind(this));

        this.canvas.on("selection:created", this.updateSelection.bind(this));
        this.canvas.on("selection:updated", this.updateSelection.bind(this));
        this.canvas.on("selection:cleared", this.updateSelection.bind(this));
    }

    public dispose() {
        if (this.canvas) {
            this.canvas.off("mouse:down");
            this.canvas.off("mouse:move");
            this.canvas.off("mouse:up");
            this.canvas.off("mouse:wheel");
            this.canvas.off("object:modified");
            this.canvas.off("mouse:dblclick");
            this.canvas.dispose();
            this.canvas = null;
        }
    }

    public resize(width: number, height: number) {
        if (this.canvas) {
            this.canvas.setWidth(width);
            this.canvas.setHeight(height);
            this.canvas.renderAll();
        }
    }

    public loadImage(url: string): Promise<void> {
        return new Promise((resolve, reject) => {
            if (!this.canvas) {
                reject(new Error("Canvas not initialized"));
                return;
            }

            this.canvas.clear();
            this.clearAnnotations();
            this.resetPolygonDrawing();

            this.canvas.setBackgroundColor("#f8fafc", this.canvas.renderAll.bind(this.canvas));

            fabric.Image.fromURL(url, (img: fabric.Image) => {
                if (!this.canvas || !img || !img.width || !img.height) {
                    reject(new Error("Failed to load image"));
                    return;
                }

                const canvasWidth = this.canvas.getWidth();
                const canvasHeight = this.canvas.getHeight();

                if (canvasWidth === 0 || canvasHeight === 0) {
                    reject(new Error("Canvas has zero dimensions"));
                    return;
                }

                const imgWidth = img.width;
                const imgHeight = img.height;

                const scaleX = canvasWidth / imgWidth;
                const scaleY = canvasHeight / imgHeight;
                const scale = Math.min(scaleX, scaleY);

                this.imageScale = scale;
                this.imageOffset = {
                    x: (canvasWidth - imgWidth * scale) / 2,
                    y: (canvasHeight - imgHeight * scale) / 2,
                };

                img.scale(scale);
                img.set({
                    left: this.imageOffset.x,
                    top: this.imageOffset.y,
                    selectable: false,
                    evented: false,
                });

                this.canvas.setBackgroundImage(img, () => {
                    this.canvas?.renderAll();
                    resolve();
                });
            });
        });
    }

    public setMode(mode: Mode) {
        this.exitPolygonEditMode();
        this.mode = mode;
        if (this.canvas) {
            this.canvas.selection = mode === "select";

            if (mode === "pan") {
                this.canvas.defaultCursor = "grab";
                this.canvas.hoverCursor = "grab";
            } else if (mode === "select") {
                this.canvas.defaultCursor = "default";
                this.canvas.hoverCursor = "move";
            } else {
                this.canvas.defaultCursor = "crosshair";
                this.canvas.hoverCursor = "crosshair";
            }

            this.canvas.discardActiveObject();
            this.canvas.renderAll();
            this.resetPolygonDrawing();
        }
        this.emit("modeChange");
    }

    public getMode(): Mode {
        return this.mode;
    }

    public getBBoxes(): BBox[] {
        return this.completedBBoxes;
    }

    public getPolygons(): Polygon[] {
        return this.completedPolygons;
    }

    public getPolylines(): Polyline[] {
        return this.completedPolylines;
    }

    public getKeypoints(): Keypoint[] {
        return this.completedKeypoints;
    }

    public isPolygonDrawing(): boolean {
        return this.mode === "polygon" && this.polygonPoints.length > 0;
    }

    public updateBBoxLabel(id: number, label: string) {
        const index = this.completedBBoxes.findIndex(box => box.id === id);
        if (index !== -1) {
            this.completedBBoxes[index].label = label;
            this.emit("update");
        }
    }

    public updatePolygonLabel(id: number, label: string) {
        const index = this.completedPolygons.findIndex(poly => poly.id === id);
        if (index !== -1) {
            this.completedPolygons[index].label = label;
            this.emit("update");
        }
    }

    public deleteBBox(id: number) {
        this.completedBBoxes = this.completedBBoxes.filter(b => b.id !== id);
        if (this.canvas) {
            const obj = this.canvas.getObjects().find((o: any) => o.annotationId === id && o.annotationType === "bbox");
            if (obj) this.canvas.remove(obj);
        }
        this.saveState();
        this.emit("update");
    }

    public deletePolygon(id: number) {
        this.completedPolygons = this.completedPolygons.filter(p => p.id !== id);
        if (this.canvas) {
            const obj = this.canvas.getObjects().find((o: any) => o.annotationId === id && o.annotationType === "polygon");
            if (obj) this.canvas.remove(obj);
        }
        this.saveState();
        this.emit("update");
    }

    // --- Private Drawing Logic ---

    private canvasToOriginalCoords(point: { x: number; y: number }): { x: number; y: number } {
        if (this.imageScale === 0) return { x: 0, y: 0 };
        return {
            x: (point.x - this.imageOffset.x) / this.imageScale,
            y: (point.y - this.imageOffset.y) / this.imageScale
        };
    }

    private originalCoordsToCanvas(point: { x: number; y: number }): { x: number; y: number } {
        return {
            x: point.x * this.imageScale + this.imageOffset.x,
            y: point.y * this.imageScale + this.imageOffset.y,
        };
    }

    public loadAnnotations(shapes: LabelMeShape[]) {
        if (!this.canvas) return;

        for (const shape of shapes) {
            const isBBox = (shape.shape_type === "rectangle" || shape.shape_type === "bounding_box") && shape.points.length >= 2;
            if (isBBox) {
                const p1 = this.originalCoordsToCanvas({ x: shape.points[0][0], y: shape.points[0][1] });
                const p2 = this.originalCoordsToCanvas({ x: shape.points[1][0], y: shape.points[1][1] });

                const left = Math.min(p1.x, p2.x);
                const top = Math.min(p1.y, p2.y);
                const width = Math.abs(p2.x - p1.x);
                const height = Math.abs(p2.y - p1.y);

                const bboxId = this.nextBBoxId;
                const rect = new fabric.Rect({
                    left, top, width, height,
                    fill: "rgba(255, 0, 0, 0.1)",
                    stroke: "red",
                    strokeWidth: this.getZoomAdaptiveStroke(),
                    selectable: true,
                    hasControls: true,
                });
                (rect as any).annotationId = bboxId;
                (rect as any).annotationType = "bbox";
                this.canvas.add(rect);

                this.completedBBoxes = [...this.completedBBoxes, {
                    id: this.nextBBoxId++,
                    label: shape.label,
                    x1: shape.points[0][0], y1: shape.points[0][1],
                    x2: shape.points[1][0], y2: shape.points[1][1],
                }];
            } else if (shape.shape_type === "polygon" && shape.points.length >= 3) {
                const canvasPoints = shape.points.map(p => {
                    const cp = this.originalCoordsToCanvas({ x: p[0], y: p[1] });
                    return new fabric.Point(cp.x, cp.y);
                });

                const polyId = this.nextPolygonId;
                const polygon = new fabric.Polygon(canvasPoints, {
                    fill: "rgba(0, 0, 255, 0.3)",
                    stroke: "blue",
                    strokeWidth: this.getZoomAdaptiveStroke(),
                    selectable: true,
                    hasControls: true,
                    objectCaching: false,
                });
                (polygon as any).annotationId = polyId;
                (polygon as any).annotationType = "polygon";
                this.canvas.add(polygon);

                this.completedPolygons = [...this.completedPolygons, {
                    id: this.nextPolygonId++,
                    label: shape.label,
                    points: shape.points.map(p => ({ x: p[0], y: p[1] })),
                }];
            } else if ((shape.shape_type === "linestrip" || shape.shape_type === "polyline") && shape.points.length >= 2) {
                // Polyline (open path)
                const canvasPoints = shape.points.map(p => {
                    const cp = this.originalCoordsToCanvas({ x: p[0], y: p[1] });
                    return { x: cp.x, y: cp.y };
                });

                const lineId = this.nextPolylineId;
                const polyline = new fabric.Polyline(canvasPoints, {
                    fill: "transparent",
                    stroke: "#f59e0b",
                    strokeWidth: this.getZoomAdaptiveStroke(),
                    selectable: true,
                    hasControls: true,
                    objectCaching: false,
                });
                (polyline as any).annotationId = lineId;
                (polyline as any).annotationType = "polyline";
                this.canvas.add(polyline);

                this.completedPolylines = [...this.completedPolylines, {
                    id: this.nextPolylineId++,
                    label: shape.label,
                    points: shape.points.map(p => ({ x: p[0], y: p[1] })),
                }];
            } else if (shape.shape_type === "point" && shape.points.length >= 1) {
                // Keypoint
                const cp = this.originalCoordsToCanvas({ x: shape.points[0][0], y: shape.points[0][1] });
                const kpId = this.nextKeypointId;
                const circle = new fabric.Circle({
                    left: cp.x,
                    top: cp.y,
                    radius: 5,
                    fill: "#ec4899",
                    stroke: "#be185d",
                    strokeWidth: this.getZoomAdaptiveStroke(FabricManager.KEYPOINT_STROKE_RATIO),
                    originX: "center",
                    originY: "center",
                    selectable: true,
                    hasControls: false,
                    hasBorders: false,
                });
                (circle as any).annotationId = kpId;
                (circle as any).annotationType = "keypoint";
                this.canvas.add(circle);

                this.completedKeypoints = [...this.completedKeypoints, {
                    id: this.nextKeypointId++,
                    label: shape.label,
                    x: shape.points[0][0],
                    y: shape.points[0][1],
                }];
            }
        }

        this.canvas.renderAll();
        this.saveState();
        this.emit("update");
    }

    private handleMouseWheel(opt: fabric.IEvent) {
        if (!this.canvas) return;
        const delta = (opt.e as WheelEvent).deltaY;
        let zoom = this.canvas.getZoom();
        zoom *= 0.999 ** delta;
        if (zoom > 20) zoom = 20;
        if (zoom < 0.01) zoom = 0.01;

        this.canvas.zoomToPoint({ x: (opt.e as WheelEvent).offsetX, y: (opt.e as WheelEvent).offsetY }, zoom);
        this.updateAllStrokes();
        opt.e.preventDefault();
        opt.e.stopPropagation();
    }

    private handleMouseDown(options: fabric.IEvent) {
        if (!this.canvas || this.mode === "select") return;

        // Panning Logic
        if (this.mode === "pan") {
            this.isPanning = true;
            this.isDrawing = false; // Pan is not drawing
            const e = options.e as MouseEvent;
            this.lastPosX = e.clientX;
            this.lastPosY = e.clientY;
            this.canvas.selection = false;
            return;
        }

        // Use getPointer for correct coordinates (accounting for zoom/pan)
        // If getPointer somehow fails (e.g. no event), fallback to options.pointer or {0,0} if desperate, but getPointer(e) is standard.
        const pointerObj = this.canvas.getPointer(options.e);
        const pointer = new fabric.Point(pointerObj.x, pointerObj.y);

        this.isDrawing = true;
        this.startPoint = pointer;

        if (this.mode === "bbox") {
            this.currentRect = new fabric.Rect({
                left: this.startPoint.x,
                top: this.startPoint.y,
                width: 0,
                height: 0,
                fill: "rgba(255, 0, 0, 0.1)",
                stroke: "red",
                strokeWidth: this.getZoomAdaptiveStroke(),
                selectable: true,
                hasControls: true,
            });
            this.canvas.add(this.currentRect);
        } else if (this.mode === "keypoint") {
            // Keypoint: place a single point immediately
            const origPt = this.canvasToOriginalCoords({ x: pointer.x, y: pointer.y });
            const kpId = this.nextKeypointId;
            const circle = new fabric.Circle({
                left: pointer.x,
                top: pointer.y,
                radius: 5,
                fill: "#ec4899",
                stroke: "#be185d",
                strokeWidth: this.getZoomAdaptiveStroke(FabricManager.KEYPOINT_STROKE_RATIO),
                originX: "center",
                originY: "center",
                selectable: true,
                hasControls: false,
                hasBorders: false,
            });
            (circle as any).annotationId = kpId;
            (circle as any).annotationType = "keypoint";
            this.canvas.add(circle);

            this.completedKeypoints = [...this.completedKeypoints, {
                id: this.nextKeypointId++,
                label: "",
                x: origPt.x,
                y: origPt.y,
            }];
            this.isDrawing = false;
            this.startPoint = null;
            this.saveState();
            this.emit("update");
        }
    }

    private handleMouseMove(options: fabric.IEvent) {
        if (!this.canvas) return;

        // Panning Logic (Must be before drawing checks)
        if (this.isPanning && this.canvas) {
            const e = options.e as MouseEvent;
            const vpt = this.canvas.viewportTransform;
            if (vpt) {
                vpt[4] += e.clientX - this.lastPosX;
                vpt[5] += e.clientY - this.lastPosY;
                this.canvas.requestRenderAll();
                this.lastPosX = e.clientX;
                this.lastPosY = e.clientY;
            }
            return;
        }

        if (!this.isDrawing || !this.startPoint) return;

        const pointerObj = this.canvas.getPointer(options.e);
        const pointer = new fabric.Point(pointerObj.x, pointerObj.y);

        if (this.mode === "bbox" && this.currentRect) {
            const width = pointer.x - this.startPoint.x;
            const height = pointer.y - this.startPoint.y;

            this.currentRect.set({
                left: Math.min(this.startPoint.x, pointer.x),
                top: Math.min(this.startPoint.y, pointer.y),
                width: Math.abs(width),
                height: Math.abs(height),
                strokeWidth: this.getZoomAdaptiveStroke(),
            });
            this.canvas.requestRenderAll(); // Optimization: use requestRenderAll
        } else if (this.mode === "polygon" && this.currentCanvasPolygonPoints.length > 0) {
            this.handlePolygonMouseMove(pointer);
        }
    }

    private handleMouseUp(options: fabric.IEvent) {
        if (!this.canvas) return;
        const pointerObj = this.canvas.getPointer(options.e);
        const pointer = new fabric.Point(pointerObj.x, pointerObj.y);

        if (this.mode === "pan") {
            this.isPanning = false;
            this.canvas.selection = true; // Re-enable selection after pan
            return;
        }

        if (this.mode === "bbox") {
            this.finishBBox();
        } else if (this.mode === "polygon") {
            this.addPolygonPoint(pointer);
            if (this.currentCanvasPolygonPoints.length > 2 && this.isNearStart(pointer)) {
                this.finishPolygon();
            }
        } else if (this.mode === "polyline") {
            this.addPolylinePoint(pointer);
        }

        if (this.mode !== "polygon" && this.mode !== "polyline") {
            this.isDrawing = false;
            this.startPoint = null;
        }
    }

    private handlePolygonMouseMove(pointer: fabric.Point) {
        if (!this.canvas) return;

        if (this.currentPolygonLine) {
            this.currentPolygonLine.set({ x2: pointer.x, y2: pointer.y });
        }

        const firstPointVisual = this.currentPolygonPointsVisual[0];
        if (this.currentCanvasPolygonPoints.length >= 3 && firstPointVisual) {
            const nearStart = this.isNearStart(pointer);

            if (nearStart && !this.isStartPointHighlighted) {
                firstPointVisual.set({
                    radius: this.START_POINT_HIGHLIGHT_RADIUS / this.canvas.getZoom(),
                    fill: "red",
                });
                this.isStartPointHighlighted = true;
            } else if (!nearStart && this.isStartPointHighlighted) {
                firstPointVisual.set({
                    radius: this.START_POINT_RADIUS / this.canvas.getZoom(),
                    fill: "blue",
                });
                this.isStartPointHighlighted = false;
            }
            this.canvas.requestRenderAll(); // Optimization
        } else {
            if (this.currentPolygonLine) this.canvas.requestRenderAll(); // Optimization
        }
    }

    private finishBBox() {
        if (this.isDrawing && this.currentRect) {
            const width = this.currentRect.width ?? 0;
            const height = this.currentRect.height ?? 0;
            const left = this.currentRect.left ?? 0;
            const top = this.currentRect.top ?? 0;

            if (width > 0 && height > 0) {
                const p1 = this.canvasToOriginalCoords({ x: left, y: top });
                const p2 = this.canvasToOriginalCoords({ x: left + width, y: top + height });

                const bboxId = this.nextBBoxId++;
                this.completedBBoxes = [...this.completedBBoxes, {
                    id: bboxId,
                    label: "",
                    x1: p1.x, y1: p1.y,
                    x2: p2.x, y2: p2.y
                }];

                (this.currentRect as any).annotationId = bboxId;
                (this.currentRect as any).annotationType = "bbox";
                this.currentRect.set({ selectable: true, hasControls: true });
                this.canvas?.setActiveObject(this.currentRect);
                this.setMode("select");
                this.saveState();
                this.emit("update");
            } else {
                this.canvas?.remove(this.currentRect);
            }
            this.currentRect = null;
        }
    }

    public addPolygonPoint(canvasPoint: fabric.Point) {
        if (!this.canvas) return;

        const originalPoint = this.canvasToOriginalCoords(canvasPoint);
        this.polygonPoints.push(originalPoint);
        this.currentCanvasPolygonPoints.push(canvasPoint);

        const pointVisual = new fabric.Circle({
            radius: this.START_POINT_RADIUS,
            fill: "blue",
            left: canvasPoint.x,
            top: canvasPoint.y,
            selectable: false,
            evented: false,
            originX: "center",
            originY: "center",
        });
        this.currentPolygonPointsVisual.push(pointVisual);
        this.canvas.add(pointVisual);

        if (this.currentCanvasPolygonPoints.length > 1) {
            const prev = this.currentCanvasPolygonPoints[this.currentCanvasPolygonPoints.length - 2];
            const segment = new fabric.Line([prev.x, prev.y, canvasPoint.x, canvasPoint.y], {
                stroke: "blue", strokeWidth: this.getZoomAdaptiveStroke(), selectable: false, evented: false
            });
            this.currentPolygonSegments.push(segment);
            this.canvas.add(segment);
        }

        if (this.currentPolygonLine) {
            this.canvas.remove(this.currentPolygonLine);
            this.currentPolygonLine = null;
        }

        // Setup new temp line
        this.currentPolygonLine = new fabric.Line(
            [canvasPoint.x, canvasPoint.y, canvasPoint.x, canvasPoint.y],
            { stroke: "rgba(0, 0, 255, 0.5)", strokeWidth: this.getZoomAdaptiveStroke(FabricManager.GUIDE_LINE_STROKE_RATIO), selectable: false, evented: false }
        );
        this.canvas.add(this.currentPolygonLine);
        this.canvas.renderAll();
        this.emit("update");
    }

    public finishPolygon() {
        if (!this.canvas || this.currentCanvasPolygonPoints.length < 3) return;

        const polyId = this.nextPolygonId++;
        const visualPolygon = new fabric.Polygon([...this.currentCanvasPolygonPoints], {
            fill: "rgba(0,0,255,0.3)",
            stroke: "blue",
            strokeWidth: this.getZoomAdaptiveStroke(),
            selectable: true,
            hasControls: true,
            objectCaching: false,
        });
        (visualPolygon as any).annotationId = polyId;
        (visualPolygon as any).annotationType = "polygon";
        this.canvas.add(visualPolygon);

        this.completedPolygons = [...this.completedPolygons, {
            id: polyId,
            label: "",
            points: [...this.polygonPoints]
        }];

        this.resetPolygonDrawing(true);
        this.saveState();
        this.emit("update");
    }

    public resetPolygonDrawing(clearOriginalPoints = true) {
        if (!this.canvas) return;

        if (this.currentPolygonLine) {
            this.canvas.remove(this.currentPolygonLine);
            this.currentPolygonLine = null;
        }
        this.currentPolygonPointsVisual.forEach(p => this.canvas?.remove(p));
        this.currentPolygonPointsVisual = [];
        this.currentPolygonSegments.forEach(s => this.canvas?.remove(s));
        this.currentPolygonSegments = [];
        this.currentCanvasPolygonPoints = [];

        if (clearOriginalPoints) {
            this.polygonPoints = [];
        }
        this.isStartPointHighlighted = false;
        this.canvas.renderAll();
        this.emit("update");
    }

    // --- Polyline Drawing ---

    public addPolylinePoint(canvasPoint: fabric.Point) {
        if (!this.canvas) return;

        const originalPoint = this.canvasToOriginalCoords(canvasPoint);
        this.polylinePoints.push(originalPoint);
        this.currentCanvasPolylinePoints.push(canvasPoint);

        const pointVisual = new fabric.Circle({
            radius: 3,
            fill: "#f59e0b",
            left: canvasPoint.x,
            top: canvasPoint.y,
            selectable: false,
            evented: false,
            originX: "center",
            originY: "center",
        });
        this.currentPolylinePointsVisual.push(pointVisual);
        this.canvas.add(pointVisual);

        // Draw segment from previous point
        if (this.currentCanvasPolylinePoints.length > 1) {
            const prev = this.currentCanvasPolylinePoints[this.currentCanvasPolylinePoints.length - 2];
            const segment = new fabric.Line(
                [prev.x, prev.y, canvasPoint.x, canvasPoint.y],
                { stroke: "#f59e0b", strokeWidth: this.getZoomAdaptiveStroke(), selectable: false, evented: false }
            );
            this.currentPolylineSegments.push(segment);
            this.canvas.add(segment);
        }

        this.canvas.renderAll();
        this.emit("update");
    }

    public finishPolyline() {
        if (!this.canvas || this.currentCanvasPolylinePoints.length < 2) return;

        const lineId = this.nextPolylineId++;
        const polyline = new fabric.Polyline(
            this.currentCanvasPolylinePoints.map(p => ({ x: p.x, y: p.y })),
            {
                fill: "transparent",
                stroke: "#f59e0b",
                strokeWidth: this.getZoomAdaptiveStroke(),
                selectable: true,
                hasControls: true,
                objectCaching: false,
            }
        );
        (polyline as any).annotationId = lineId;
        (polyline as any).annotationType = "polyline";
        this.canvas.add(polyline);

        this.completedPolylines = [...this.completedPolylines, {
            id: lineId,
            label: "",
            points: [...this.polylinePoints],
        }];

        this.resetPolylineDrawing();
        this.saveState();
        this.emit("update");
    }

    public resetPolylineDrawing() {
        if (!this.canvas) return;
        this.currentPolylinePointsVisual.forEach(p => this.canvas?.remove(p));
        this.currentPolylinePointsVisual = [];
        this.currentPolylineSegments.forEach(s => this.canvas?.remove(s));
        this.currentPolylineSegments = [];
        this.currentCanvasPolylinePoints = [];
        this.polylinePoints = [];
        this.canvas.renderAll();
    }

    public deletePolyline(id: number) {
        this.completedPolylines = this.completedPolylines.filter(l => l.id !== id);
        if (this.canvas) {
            const obj = this.canvas.getObjects().find((o: any) => o.annotationId === id && o.annotationType === "polyline");
            if (obj) this.canvas.remove(obj);
        }
        this.saveState();
        this.emit("update");
    }

    public deleteKeypoint(id: number) {
        this.completedKeypoints = this.completedKeypoints.filter(k => k.id !== id);
        if (this.canvas) {
            const obj = this.canvas.getObjects().find((o: any) => o.annotationId === id && o.annotationType === "keypoint");
            if (obj) this.canvas.remove(obj);
        }
        this.saveState();
        this.emit("update");
    }

    public updatePolylineLabel(id: number, label: string) {
        const index = this.completedPolylines.findIndex(l => l.id === id);
        if (index !== -1) {
            this.completedPolylines[index].label = label;
            this.emit("update");
        }
    }

    public updateKeypointLabel(id: number, label: string) {
        const index = this.completedKeypoints.findIndex(k => k.id === id);
        if (index !== -1) {
            this.completedKeypoints[index].label = label;
            this.emit("update");
        }
    }

    private getDataById(id: number, type: string): any {
        if (type === 'bbox') return this.completedBBoxes.find(b => b.id === id);
        if (type === 'polygon') return this.completedPolygons.find(p => p.id === id);
        if (type === 'polyline') return this.completedPolylines.find(l => l.id === id);
        if (type === 'keypoint') return this.completedKeypoints.find(k => k.id === id);
        return null;
    }

    private updateSelection() {
        if (!this.canvas) return;
        const active = this.canvas.getActiveObject();

        this.selectedObjects = [];
        this.selectedObject = null;

        if (!active) {
            this.emit("update");
            return;
        }

        if (active.type === 'activeSelection') {
            const group = active as fabric.ActiveSelection;
            group.getObjects().forEach(obj => {
                if ((obj as any).annotationId !== undefined) {
                    const id = (obj as any).annotationId;
                    const type = (obj as any).annotationType;
                    const data = this.getDataById(id, type);
                    if (data) {
                        this.selectedObjects.push({ ...data, type });
                    }
                }
            });
            this.selectedObject = this.selectedObjects.length > 0 ? this.selectedObjects[0] : null;
        } else if ((active as any).annotationId !== undefined) {
            const id = (active as any).annotationId;
            const type = (active as any).annotationType;
            const data = this.getDataById(id, type);
            if (data) {
                this.selectedObject = { ...data, type };
                this.selectedObjects = [this.selectedObject];
            }
        }

        this.emit("update");
    }

    public getSelectedObjects() {
        return this.selectedObjects;
    }

    public getSelectedObject() {
        return this.selectedObject;
    }

    public batchUpdateLabel(label: string) {
        if (this.selectedObjects.length === 0) return;

        this.selectedObjects.forEach(item => {
            const type = item.type;
            const id = item.id;

            if (type === 'bbox') this.updateBBoxLabel(id, label);
            else if (type === 'polygon') this.updatePolygonLabel(id, label);
            else if (type === 'polyline') this.updatePolylineLabel(id, label);
            else if (type === 'keypoint') this.updateKeypointLabel(id, label);
        });

        this.emit("update");
    }

    public updateProperty(id: number, type: string, prop: string, value: any) {
        if (!this.canvas) return;

        if (type === 'bbox') {
            const box = this.completedBBoxes.find(b => b.id === id);
            if (box) {
                (box as any)[prop] = value;
                const obj = this.canvas.getObjects().find(o => (o as any).annotationId === id && (o as any).annotationType === 'bbox');
                if (obj) {
                    const p1 = this.originalCoordsToCanvas({ x: box.x1, y: box.y1 });
                    const p2 = this.originalCoordsToCanvas({ x: box.x2, y: box.y2 });
                    obj.set({
                        left: Math.min(p1.x, p2.x),
                        top: Math.min(p1.y, p2.y),
                        width: Math.abs(p2.x - p1.x),
                        height: Math.abs(p2.y - p1.y)
                    });
                }
            }
        } else if (type === 'keypoint') {
            const kp = this.completedKeypoints.find(k => k.id === id);
            if (kp) {
                (kp as any)[prop] = value;
                const obj = this.canvas.getObjects().find(o => (o as any).annotationId === id && (o as any).annotationType === 'keypoint');
                if (obj) {
                    const cp = this.originalCoordsToCanvas({ x: kp.x, y: kp.y });
                    obj.set({ left: cp.x, top: cp.y });
                }
            }
        }

        this.canvas.renderAll();
        this.updateSelection();
        this.saveState();
        this.emit("update");
    }

    private isNearStart(pointer: fabric.Point, threshold = 10): boolean {
        if (this.currentCanvasPolygonPoints.length < 3) return false;
        const start = this.currentCanvasPolygonPoints[0];
        const dist = Math.sqrt(Math.pow(pointer.x - start.x, 2) + Math.pow(pointer.y - start.y, 2));
        return dist < threshold;
    }

    private clearAnnotations() {
        this.completedBBoxes = [];
        this.completedPolygons = [];
        this.completedPolylines = [];
        this.completedKeypoints = [];
        this.nextBBoxId = 1;
        this.nextPolygonId = 1;
        this.nextPolylineId = 1;
        this.nextKeypointId = 1;
        this.emit("update");
    }

    public toggleLock(id: number, type: string) {
        if (!this.canvas) return;
        const list = this.getListByType(type);
        const item = list.find((i: any) => i.id === id);
        if (item) {
            item.locked = !item.locked;
            const obj = this.canvas.getObjects().find(o => (o as any).annotationId === id && (o as any).annotationType === type);
            if (obj) {
                obj.set({
                    selectable: !item.locked,
                    evented: !item.locked,
                    lockMovementX: item.locked,
                    lockMovementY: item.locked,
                    lockScalingX: item.locked,
                    lockScalingY: item.locked,
                    lockRotation: item.locked,
                });
            }
            this.emit("update");
            this.saveState();
        }
    }

    public toggleVisibility(id: number, type: string) {
        if (!this.canvas) return;
        const list = this.getListByType(type);
        const item = list.find((i: any) => i.id === id);
        if (item) {
            item.hidden = !item.hidden;
            const obj = this.canvas.getObjects().find(o => (o as any).annotationId === id && (o as any).annotationType === type);
            if (obj) {
                obj.set("visible", !item.hidden);
            }
            this.canvas.renderAll();
            this.emit("update");
            this.saveState();
        }
    }

    private getListByType(type: string): any[] {
        if (type === 'bbox') return this.completedBBoxes;
        if (type === 'polygon') return this.completedPolygons;
        if (type === 'polyline') return this.completedPolylines;
        if (type === 'keypoint') return this.completedKeypoints;
        return [];
    }

    private saveState() {
        this.undoStack.push({
            bboxes: this.completedBBoxes.map(b => ({ ...b })),
            polygons: this.completedPolygons.map(p => ({ ...p, points: p.points.map(pt => ({ ...pt })) })),
            polylines: this.completedPolylines.map(l => ({ ...l, points: l.points.map(pt => ({ ...pt })) })),
            keypoints: this.completedKeypoints.map(k => ({ ...k })),
            nextBBoxId: this.nextBBoxId,
            nextPolygonId: this.nextPolygonId,
            nextPolylineId: this.nextPolylineId,
            nextKeypointId: this.nextKeypointId,
        });
        if (this.undoStack.length > FabricManager.MAX_HISTORY) {
            this.undoStack.shift();
        }
        this.redoStack = [];
    }

    private createSnapshot(): HistoryState {
        return {
            bboxes: this.completedBBoxes.map(b => ({ ...b })),
            polygons: this.completedPolygons.map(p => ({ ...p, points: p.points.map(pt => ({ ...pt })) })),
            polylines: this.completedPolylines.map(l => ({ ...l, points: l.points.map(pt => ({ ...pt })) })),
            keypoints: this.completedKeypoints.map(k => ({ ...k })),
            nextBBoxId: this.nextBBoxId,
            nextPolygonId: this.nextPolygonId,
            nextPolylineId: this.nextPolylineId,
            nextKeypointId: this.nextKeypointId,
        };
    }

    public undo() {
        if (this.undoStack.length === 0) return;
        this.redoStack.push(this.createSnapshot());
        const prev = this.undoStack.pop()!;
        this.restoreState(prev);
    }

    public redo() {
        if (this.redoStack.length === 0) return;
        this.undoStack.push(this.createSnapshot());
        const next = this.redoStack.pop()!;
        this.restoreState(next);
    }

    private restoreState(state: HistoryState) {
        this.completedBBoxes = state.bboxes;
        this.completedPolygons = state.polygons;
        this.completedPolylines = state.polylines;
        this.completedKeypoints = state.keypoints;
        this.nextBBoxId = state.nextBBoxId;
        this.nextPolygonId = state.nextPolygonId;
        this.nextPolylineId = state.nextPolylineId;
        this.nextKeypointId = state.nextKeypointId;
        this.redrawFromState();
        this.emit("update");
    }

    private redrawFromState() {
        if (!this.canvas) return;

        // Remove all annotation objects (keep background image)
        const objects = this.canvas.getObjects().slice();
        for (const obj of objects) {
            if ((obj as any).annotationId !== undefined) {
                this.canvas.remove(obj);
            }
        }

        // Redraw bboxes
        for (const box of this.completedBBoxes) {
            const p1 = this.originalCoordsToCanvas({ x: box.x1, y: box.y1 });
            const p2 = this.originalCoordsToCanvas({ x: box.x2, y: box.y2 });
            const rect = new fabric.Rect({
                left: Math.min(p1.x, p2.x),
                top: Math.min(p1.y, p2.y),
                width: Math.abs(p2.x - p1.x),
                height: Math.abs(p2.y - p1.y),
                fill: "rgba(255, 0, 0, 0.1)",
                stroke: "red",
                strokeWidth: this.getZoomAdaptiveStroke(),
                selectable: true,
                hasControls: true,
            });
            (rect as any).annotationId = box.id;
            (rect as any).annotationType = "bbox";
            this.canvas.add(rect);
        }

        // Redraw polygons
        for (const poly of this.completedPolygons) {
            const canvasPoints = poly.points.map(p => {
                const cp = this.originalCoordsToCanvas(p);
                return new fabric.Point(cp.x, cp.y);
            });
            const polygon = new fabric.Polygon(canvasPoints, {
                fill: "rgba(0, 0, 255, 0.3)",
                stroke: "blue",
                strokeWidth: this.getZoomAdaptiveStroke(),
                selectable: true,
                hasControls: true,
                objectCaching: false,
            });
            (polygon as any).annotationId = poly.id;
            (polygon as any).annotationType = "polygon";
            this.canvas.add(polygon);
        }

        // Redraw polylines
        for (const line of this.completedPolylines) {
            const canvasPoints = line.points.map(p => this.originalCoordsToCanvas(p));
            const polyline = new fabric.Polyline(canvasPoints.map(cp => ({ x: cp.x, y: cp.y })), {
                fill: "transparent",
                stroke: "#f59e0b",
                strokeWidth: this.getZoomAdaptiveStroke(),
                selectable: true,
                hasControls: true,
                objectCaching: false,
            });
            (polyline as any).annotationId = line.id;
            (polyline as any).annotationType = "polyline";
            this.canvas.add(polyline);
        }

        // Redraw keypoints
        for (const kp of this.completedKeypoints) {
            const cp = this.originalCoordsToCanvas({ x: kp.x, y: kp.y });
            const circle = new fabric.Circle({
                left: cp.x,
                top: cp.y,
                radius: 5,
                fill: "#ec4899",
                stroke: "#be185d",
                strokeWidth: this.getZoomAdaptiveStroke(FabricManager.KEYPOINT_STROKE_RATIO),
                originX: "center",
                originY: "center",
                selectable: true,
                hasControls: false,
                hasBorders: false,
            });
            (circle as any).annotationId = kp.id;
            (circle as any).annotationType = "keypoint";
            this.canvas.add(circle);
        }

        this.canvas.renderAll();
    }

    // --- Object Modification Sync ---

    private handleObjectModified(opt: fabric.IEvent) {
        const target = opt.target;
        if (!target) return;

        const id = (target as any).annotationId;
        const type = (target as any).annotationType;
        if (id === undefined || !type) return;

        if (type === "bbox") {
            const bound = target.getBoundingRect(true);
            const p1 = this.canvasToOriginalCoords({ x: bound.left, y: bound.top });
            const p2 = this.canvasToOriginalCoords({ x: bound.left + bound.width, y: bound.top + bound.height });

            const idx = this.completedBBoxes.findIndex(b => b.id === id);
            if (idx !== -1) {
                this.completedBBoxes[idx] = {
                    ...this.completedBBoxes[idx],
                    x1: p1.x, y1: p1.y,
                    x2: p2.x, y2: p2.y,
                };
                this.saveState();
                this.emit("update");
            }
        } else if (type === "polygon") {
            // For polygons, recalculate from the transformed points
            const fabricPoly = target as fabric.Polygon;
            if (fabricPoly.points) {
                const matrix = fabricPoly.calcTransformMatrix();
                const newPoints = fabricPoly.points.map(p => {
                    const transformed = fabric.util.transformPoint(
                        new fabric.Point(p.x - (fabricPoly.pathOffset?.x || 0), p.y - (fabricPoly.pathOffset?.y || 0)),
                        matrix
                    );
                    return this.canvasToOriginalCoords({ x: transformed.x, y: transformed.y });
                });

                const idx = this.completedPolygons.findIndex(poly => poly.id === id);
                if (idx !== -1) {
                    this.completedPolygons[idx] = {
                        ...this.completedPolygons[idx],
                        points: newPoints,
                    };
                    this.saveState();
                    this.emit("update");
                }
            }
        }
    }

    // --- Polygon Node Editing ---

    private handleDoubleClick(opt: fabric.IEvent) {
        if (!this.canvas || this.mode !== "select") return;
        const target = opt.target;
        if (!target) {
            this.exitPolygonEditMode();
            return;
        }

        if ((target as any).annotationType === "polygon" && (target as any).annotationId !== undefined) {
            this.enterPolygonEditMode(target as fabric.Polygon, (target as any).annotationId);
        }
    }

    private enterPolygonEditMode(polygon: fabric.Polygon, id: number) {
        if (!this.canvas) return;

        // Exit any previous edit mode
        this.exitPolygonEditMode();

        this.editingPolygonId = id;
        this.editingPolygonObject = polygon;

        // Make the polygon non-selectable while editing vertices
        polygon.set({
            selectable: false,
            evented: false,
            stroke: "cyan",
            strokeWidth: this.getZoomAdaptiveStroke(),
        });

        // Create draggable handles for each vertex
        if (!polygon.points) return;
        const matrix = polygon.calcTransformMatrix();

        for (let i = 0; i < polygon.points.length; i++) {
            const point = polygon.points[i];
            const transformed = fabric.util.transformPoint(
                new fabric.Point(
                    point.x - (polygon.pathOffset?.x || 0),
                    point.y - (polygon.pathOffset?.y || 0)
                ),
                matrix
            );
            this.createVertexHandle(i, transformed.x, transformed.y);
        }

        this.canvas.renderAll();
    }

    private createVertexHandle(index: number, x: number, y: number) {
        if (!this.canvas) return;

        const handle = new fabric.Circle({
            left: x,
            top: y,
            radius: 5,
            fill: "cyan",
            stroke: "#0e7490",
            strokeWidth: this.getZoomAdaptiveStroke(FabricManager.KEYPOINT_STROKE_RATIO),
            originX: "center",
            originY: "center",
            selectable: true,
            hasBorders: false,
            hasControls: false,
        });

        (handle as any).isVertexHandle = true;
        (handle as any).vertexIndex = index;

        // Update polygon shape as handle is dragged
        handle.on("moving", () => {
            if (!this.editingPolygonObject || !this.editingPolygonObject.points || !this.canvas) return;

            const pts = this.editingPolygonObject.points;
            const matrix = this.editingPolygonObject.calcTransformMatrix();
            const invMatrix = fabric.util.invertTransform(matrix);

            // Convert handle position back to polygon-local coords
            const localPt = fabric.util.transformPoint(
                new fabric.Point(handle.left || 0, handle.top || 0),
                invMatrix
            );

            pts[index] = new fabric.Point(
                localPt.x + (this.editingPolygonObject.pathOffset?.x || 0),
                localPt.y + (this.editingPolygonObject.pathOffset?.y || 0)
            );

            this.editingPolygonObject.dirty = true;
            this.canvas.requestRenderAll();
        });

        this.vertexHandles.push(handle);
        this.canvas.add(handle);
    }

    public exitPolygonEditMode() {
        if (!this.canvas) return;
        if (this.editingPolygonId === null) return;

        // Sync edited polygon back to state
        if (this.editingPolygonObject && this.editingPolygonObject.points) {
            const matrix = this.editingPolygonObject.calcTransformMatrix();
            const newPoints = this.editingPolygonObject.points.map(p => {
                const transformed = fabric.util.transformPoint(
                    new fabric.Point(
                        p.x - (this.editingPolygonObject!.pathOffset?.x || 0),
                        p.y - (this.editingPolygonObject!.pathOffset?.y || 0)
                    ),
                    matrix
                );
                return this.canvasToOriginalCoords({ x: transformed.x, y: transformed.y });
            });

            const idx = this.completedPolygons.findIndex(poly => poly.id === this.editingPolygonId);
            if (idx !== -1) {
                this.completedPolygons[idx] = {
                    ...this.completedPolygons[idx],
                    points: newPoints,
                };
            }

            // Restore polygon style
            this.editingPolygonObject.set({
                selectable: true,
                evented: true,
                stroke: "blue",
                strokeWidth: this.getZoomAdaptiveStroke(),
            });
        }

        // Remove vertex handles
        for (const handle of this.vertexHandles) {
            this.canvas.remove(handle);
        }
        this.vertexHandles = [];
        this.editingPolygonId = null;
        this.editingPolygonObject = null;

        this.saveState();
        this.canvas.renderAll();
        this.emit("update");
    }

    // --- Stroke Width Management ---

    /** Returns the effective stroke width for the current zoom level */
    private getZoomAdaptiveStroke(ratio = 1): number {
        const zoom = this.canvas?.getZoom() ?? 1;
        return (this.baseStrokeWidth * ratio) / zoom;
    }

    /** Update base stroke width and refresh all canvas objects */
    public setBaseStrokeWidth(value: number): void {
        this.baseStrokeWidth = value;
        this.updateAllStrokes();
    }

    /** Recalculate stroke width for all annotation objects on canvas */
    private updateAllStrokes(): void {
        if (!this.canvas) return;
        const zoom = this.canvas.getZoom();
        const standardStroke = this.baseStrokeWidth / zoom;
        const keypointStroke = (this.baseStrokeWidth * FabricManager.KEYPOINT_STROKE_RATIO) / zoom;

        for (const obj of this.canvas.getObjects()) {
            const type = (obj as any).annotationType;
            if (type === "bbox" || type === "polygon" || type === "polyline") {
                obj.set("strokeWidth", standardStroke);
            } else if (type === "keypoint") {
                obj.set("strokeWidth", keypointStroke);
            }
            if ((obj as any).isVertexHandle) {
                obj.set("strokeWidth", keypointStroke);
            }
        }
        this.canvas.requestRenderAll();
    }

    // --- Event System ---

    public on(event: string, callback: EventCallback) {
        if (!this.listeners[event]) this.listeners[event] = [];
        this.listeners[event].push(callback);
        // Return unsubscribe function
        return () => {
            this.listeners[event] = this.listeners[event].filter(cb => cb !== callback);
        };
    }

    private emit(event: string) {
        if (this.listeners[event]) {
            this.listeners[event].forEach(cb => cb());
        }
    }
}

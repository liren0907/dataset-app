/**
 * Tauri 拖放功能 Composable
 *
 * 封裝了 Tauri 的拖放監聽器設置、座標轉換、以及事件處理邏輯
 */
import { isDraggingOver, activeDropZone, type DropZoneType } from '../stores/exportStore';

/** 取消監聽函數的型別 */
type UnlistenFn = () => void;

/** 拖放回調配置 */
export interface DragDropCallbacks {
	onSourceDrop?: (path: string) => void;
	onOutputDrop?: (path: string) => void;
}

/** Drop Zone 元素參照 */
export interface DropZoneRefs {
	sourceDropZone: HTMLElement | null;
	outputDropZone: HTMLElement | null;
}

/**
 * 檢查物理座標是否在元素範圍內
 *
 * Tauri 給的是 PhysicalPosition（考慮 DPI 縮放），需要轉換為 CSS 座標
 *
 * @param physicalX 物理 X 座標
 * @param physicalY 物理 Y 座標
 * @param element 目標 HTML 元素
 * @returns 座標是否在元素範圍內
 */
export function isPointInElement(physicalX: number, physicalY: number, element: HTMLElement | null): boolean {
	if (!element) return false;

	// 將物理座標轉換為 CSS 座標（除以 DPI 縮放比例）
	const scaleFactor = window.devicePixelRatio || 1;
	const cssX = physicalX / scaleFactor;
	const cssY = physicalY / scaleFactor;

	const rect = element.getBoundingClientRect();
	return cssX >= rect.left && cssX <= rect.right && cssY >= rect.top && cssY <= rect.bottom;
}

/**
 * 根據座標判斷目前在哪個 drop zone
 */
function detectDropZone(
	position: { x: number; y: number },
	refs: DropZoneRefs
): DropZoneType {
	if (position && refs.sourceDropZone && isPointInElement(position.x, position.y, refs.sourceDropZone)) {
		return 'source';
	} else if (position && refs.outputDropZone && isPointInElement(position.x, position.y, refs.outputDropZone)) {
		return 'output';
	}
	return null;
}

/**
 * 建立拖放事件處理器
 */
function createDragDropHandler(refs: DropZoneRefs, callbacks: DragDropCallbacks) {
	return function handleDragDropEvent(payload: any) {
		const eventType = payload.type;

		if (eventType === 'enter' || eventType === 'over') {
			isDraggingOver.set(true);
			const position = payload.position;
			const zone = detectDropZone(position, refs);
			activeDropZone.set(zone);
		} else if (eventType === 'drop') {
			const paths = payload.paths;
			const dropPosition = payload.position;

			// 用 drop 事件的座標計算目標區域
			let dropZone: DropZoneType = null;
			if (dropPosition && refs.sourceDropZone && refs.outputDropZone) {
				dropZone = detectDropZone(dropPosition, refs);
			} else {
				// Fallback 到目前 activeDropZone 的值
				// 注意：這裡需要同步取得 store 值，但在事件處理中我們使用 closure
				let currentActiveZone: DropZoneType = null;
				const unsubscribe = activeDropZone.subscribe(v => { currentActiveZone = v; });
				unsubscribe();
				dropZone = currentActiveZone;
			}

			if (paths && paths.length > 0 && dropZone) {
				const droppedPath = paths[0];
				if (dropZone === 'source' && callbacks.onSourceDrop) {
					callbacks.onSourceDrop(droppedPath);
				} else if (dropZone === 'output' && callbacks.onOutputDrop) {
					callbacks.onOutputDrop(droppedPath);
				}
			}

			isDraggingOver.set(false);
			activeDropZone.set(null);
		} else if (eventType === 'leave' || eventType === 'cancel') {
			isDraggingOver.set(false);
			activeDropZone.set(null);
		}
	};
}

/**
 * 設置 Tauri 拖放監聽器
 *
 * @param refs Drop zone 元素參照
 * @param callbacks 拖放事件回調
 * @returns 用於清理的 unlisten 函數陣列
 */
export async function setupDragDropListener(
	refs: DropZoneRefs,
	callbacks: DragDropCallbacks
): Promise<UnlistenFn[]> {
	const unlistenFns: UnlistenFn[] = [];

	if (typeof window === 'undefined' || !(window as any).__TAURI__) {
		return unlistenFns;
	}

	try {
		const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
		const appWindow = getCurrentWebviewWindow();
		const handleDragDropEvent = createDragDropHandler(refs, callbacks);

		const unlisten = await appWindow.onDragDropEvent((event) => {
			handleDragDropEvent(event.payload);
		});

		unlistenFns.push(unlisten);
	} catch (error) {
		console.error('❌ 拖放監聯器設置失敗:', error);
	}

	return unlistenFns;
}

/**
 * 清理所有拖放監聽器
 */
export function cleanupDragDropListeners(unlistenFns: UnlistenFn[]): void {
	unlistenFns.forEach((fn) => fn());
}

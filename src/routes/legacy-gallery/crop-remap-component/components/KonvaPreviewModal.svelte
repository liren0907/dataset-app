<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { KonvaManager, type KonvaState } from "../services/konvaService";
    import type { PreviewImage } from "../services/dataService";

    let { image, onclose }: {
        image: PreviewImage;
        onclose?: () => void;
    } = $props();

    let konvaContainer: HTMLDivElement;
    let manager: KonvaManager;

    let state: KonvaState = $state({
        scale: 1,
        stageX: 0,
        stageY: 0,
        selectedAnnotationCount: 0,
        totalAnnotations: 0,
    });

    onMount(() => {
        // Initialize KonvaManager
        manager = new KonvaManager((newState) => {
            state = { ...state, ...newState };
        });

        // Small timeout to ensure container is ready
        setTimeout(() => {
            if (konvaContainer) {
                manager.initialize(konvaContainer, image);
            }
        }, 100);
    });

    onDestroy(() => {
        if (manager) {
            manager.destroy();
        }
    });

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") {
            onclose?.();
        }

        // Forward keys to manager or handle locally
        switch (e.key.toLowerCase()) {
            case "delete":
            case "backspace":
                manager.deleteSelectedAnnotation();
                break;
            case "a":
                if (e.ctrlKey || e.metaKey) {
                    e.preventDefault();
                    manager.selectAllAnnotations();
                }
                break;
            case "=":
            case "+":
                manager.zoomIn();
                break;
            case "-":
            case "_":
                manager.zoomOut();
                break;
            case "0":
                manager.resetZoom();
                break;
            case "r":
                manager.fitToScreen();
                break;
        }
    }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    role="dialog"
    aria-modal="true"
    onclick={() => onclose?.()}
    tabindex="-1"
>
    <div
        class="bg-white rounded-lg shadow-xl max-w-6xl max-h-[95vh] overflow-hidden w-full flex flex-col"
        role="document"
        onclick={(e) => e.stopPropagation()}
    >
        <div
            class="p-4 border-b border-gray-200 flex justify-between items-center"
        >
            <h3 class="text-lg font-medium text-gray-900">{image.name}</h3>
            <button
                onclick={() => onclose?.()}
                class="text-gray-400 hover:text-gray-600 text-2xl leading-none px-2"
            >
                ×
            </button>
        </div>

        <div class="p-4 flex-1 overflow-hidden flex flex-col">
            <!-- Control Panel -->
            <div
                class="flex flex-wrap items-center gap-2 mb-4 p-3 bg-gray-50 rounded-lg"
            >
                <div class="text-sm text-gray-600 mr-4">Controls:</div>

                <div class="flex gap-1">
                    <button
                        onclick={() => manager.zoomOut()}
                        class="btn-tool"
                        title="Zoom Out (-)">🔍-</button
                    >
                    <button
                        onclick={() => manager.resetZoom()}
                        class="btn-tool"
                        title="Reset (0)">100%</button
                    >
                    <button
                        onclick={() => manager.zoomIn()}
                        class="btn-tool"
                        title="Zoom In (+)">🔍+</button
                    >
                    <button
                        onclick={() => manager.fitToScreen()}
                        class="btn-tool ml-2 bg-green-500 hover:bg-green-600"
                        title="Fit (R)">📐 Fit</button
                    >
                </div>

                <!-- Annotation Controls -->
                <div
                    class="ml-4 flex items-center gap-2 border-l pl-4 border-gray-300"
                >
                    <span class="text-sm text-gray-600">Annotations:</span>
                    <button
                        onclick={() => manager.selectAllAnnotations()}
                        class="btn-tool bg-purple-500 hover:bg-purple-600"
                        >Select All</button
                    >
                    <button
                        onclick={() => manager.deselectAnnotation()}
                        class="btn-tool bg-gray-500 hover:bg-gray-600"
                        >Deselect</button
                    >
                    <button
                        onclick={() => manager.deleteSelectedAnnotation()}
                        class="btn-tool bg-red-500 hover:bg-red-600"
                        >🗑️ Delete</button
                    >
                </div>

                <!-- Status -->
                <div class="ml-auto text-sm text-gray-600">
                    Zoom: {Math.round(state.scale * 100)}% | Total: {state.totalAnnotations}
                    | Selected: {state.selectedAnnotationCount}
                </div>
            </div>

            <!-- Konva Container -->
            <div
                class="relative flex-1 bg-gray-100 rounded-lg overflow-hidden border-2 border-gray-300"
            >
                <div
                    bind:this={konvaContainer}
                    class="w-full h-full bg-gray-50 flex items-center justify-center min-h-[500px]"
                ></div>

                <div
                    class="absolute top-2 left-2 bg-green-600 text-white text-xs px-2 py-1 rounded shadow z-10 pointer-events-none"
                >
                    Enhanced KonvaJS
                </div>
            </div>

            <div class="mt-2 text-xs text-gray-500 text-center">
                Shortcuts: Zoom (+/-), Reset (0), Fit (R), Select All (Ctrl+A),
                Delete (Del), Deselect (Esc)
            </div>
        </div>
    </div>
</div>

<style>
    .btn-tool {
        @apply px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white text-sm rounded transition-colors;
    }
</style>

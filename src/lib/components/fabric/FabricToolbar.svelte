<script lang="ts">
    import type { Mode } from "$lib/logic/FabricManager";
    import { IconButton, ToggleButtonGroup } from "$lib/components/ui";

    let {
        mode = "select" as Mode,
        isPolygonDrawing = false,
        isPolylineDrawing = false,
        strokeWidth = 2,
        showVertexPoints = true,
        onsetmode,
        ontriggerfileinput,
        ontriggerdirectoryinput,
        ontriggermockload,
        onfinishpolygon,
        onresetpolygon,
        onfinishpolyline,
        onresetpolyline,
        onstrokewidthchange,
        ontogglevertexpoints,
    }: {
        mode?: Mode;
        isPolygonDrawing?: boolean;
        isPolylineDrawing?: boolean;
        strokeWidth?: number;
        showVertexPoints?: boolean;
        onsetmode?: (mode: Mode) => void;
        ontriggerfileinput?: () => void;
        ontriggerdirectoryinput?: () => void;
        ontriggermockload?: () => void;
        onfinishpolygon?: () => void;
        onresetpolygon?: () => void;
        onfinishpolyline?: () => void;
        onresetpolyline?: () => void;
        onstrokewidthchange?: (width: number) => void;
        ontogglevertexpoints?: () => void;
    } = $props();

    const toolOptions = [
        { value: "select", icon: "arrow_selector_tool", tooltip: "Select (V)" },
        { value: "bbox", icon: "crop_square", tooltip: "Bounding Box (R)" },
        { value: "polygon", icon: "pentagon", tooltip: "Polygon (P)" },
        { value: "polyline", icon: "polyline", tooltip: "Polyline (L)" },
        { value: "keypoint", icon: "location_on", tooltip: "Keypoint (K)" },
        { value: "pan", icon: "pan_tool", tooltip: "Pan (H)" },
    ];

    function handleModeChange(value: Mode) {
        onsetmode?.(value);
    }
</script>

<div
    class="relative z-10 flex items-center gap-3 px-4 py-2 border-b border-base-200 bg-base-100 shrink-0"
>
    <!-- Title -->
    <div class="flex items-center gap-2">
        <span class="material-symbols-rounded text-primary text-[20px]"
            >brush</span
        >
        <h1 class="text-base font-bold text-base-content whitespace-nowrap">
            Fabric Annotator
        </h1>
    </div>

    <div class="w-px h-6 bg-base-300"></div>

    <!-- Tool Mode Selector -->
    <ToggleButtonGroup
        options={toolOptions}
        value={mode}
        onchange={handleModeChange}
    />

    <div class="w-px h-6 bg-base-300"></div>

    <!-- Load Image / Directory -->
    <IconButton
        icon="image"
        label="Load Image"
        bordered
        onclick={() => ontriggerfileinput?.()}
    />
    <IconButton
        icon="folder_open"
        label="Load Directory"
        bordered
        onclick={() => ontriggerdirectoryinput?.()}
    />
    <IconButton
        icon="science"
        label="Mock"
        bordered
        class="btn-ghost text-primary hover:bg-primary/10"
        onclick={() => ontriggermockload?.()}
    />

    <div class="w-px h-6 bg-base-300"></div>

    <!-- Stroke Width Control -->
    <div class="flex items-center gap-1.5" title="Stroke Width">
        <span class="material-symbols-rounded text-base-content/60 text-[18px]">line_weight</span>
        <input
            type="range"
            min="0.5"
            max="5"
            step="0.5"
            value={strokeWidth}
            class="range range-xs range-primary w-20"
            oninput={(e) => onstrokewidthchange?.(parseFloat(e.currentTarget.value))}
        />
        <span class="text-[10px] font-mono text-base-content/50 w-6 text-center">
            {strokeWidth}
        </span>
    </div>

    <!-- Vertex Points Toggle -->
    <button
        class="btn btn-ghost btn-xs gap-1"
        class:btn-active={showVertexPoints}
        title={showVertexPoints ? "Hide Vertex Points" : "Show Vertex Points"}
        onclick={() => ontogglevertexpoints?.()}
    >
        <span class="material-symbols-rounded text-[18px]">scatter_plot</span>
    </button>

    <!-- Polygon Actions (contextual) -->
    {#if mode === "polygon" && isPolygonDrawing}
        <div class="w-px h-6 bg-base-300"></div>
        <div class="flex items-center gap-2">
            <IconButton
                icon="check_circle"
                label="Finish"
                bordered
                class="btn-success text-success-content hover:bg-success hover:text-success-content"
                onclick={() => onfinishpolygon?.()}
            />
            <IconButton
                icon="cancel"
                label="Cancel"
                bordered
                class="btn-error text-error-content hover:bg-error hover:text-error-content"
                onclick={() => onresetpolygon?.()}
            />
        </div>
    {/if}

    <!-- Polyline Actions (contextual) -->
    {#if mode === "polyline" && isPolylineDrawing}
        <div class="w-px h-6 bg-base-300"></div>
        <div class="flex items-center gap-2">
            <IconButton
                icon="check_circle"
                label="Finish"
                bordered
                class="btn-success text-success-content hover:bg-success hover:text-success-content"
                onclick={() => onfinishpolyline?.()}
            />
            <IconButton
                icon="cancel"
                label="Cancel"
                bordered
                class="btn-error text-error-content hover:bg-error hover:text-error-content"
                onclick={() => onresetpolyline?.()}
            />
        </div>
    {/if}
</div>

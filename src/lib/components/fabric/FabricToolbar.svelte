<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { Mode } from "$lib/logic/FabricManager";
    import RawButton from "$lib/components/ui/RawButton.svelte";
    import IconSegmentedControl from "$lib/components/ui/IconSegmentedControl.svelte";

    export let mode: Mode = "select";
    export let isPolygonDrawing = false;
    export let isPolylineDrawing = false;
    export let strokeWidth = 2;
    export let showVertexPoints = true;

    const dispatch = createEventDispatcher<{
        setMode: Mode;
        triggerFileInput: void;
        triggerDirectoryInput: void;
        triggerMockLoad: void;
        finishPolygon: void;
        resetPolygon: void;
        finishPolyline: void;
        resetPolyline: void;
        strokeWidthChange: number;
        toggleVertexPoints: void;
    }>();

    const toolOptions = [
        { value: "select", icon: "arrow_selector_tool", tooltip: "Select (V)" },
        { value: "bbox", icon: "crop_square", tooltip: "Bounding Box (R)" },
        { value: "polygon", icon: "pentagon", tooltip: "Polygon (P)" },
        { value: "polyline", icon: "polyline", tooltip: "Polyline (L)" },
        { value: "keypoint", icon: "location_on", tooltip: "Keypoint (K)" },
        { value: "pan", icon: "pan_tool", tooltip: "Pan (H)" },
    ];

    function handleModeChange(event: CustomEvent<Mode>) {
        dispatch("setMode", event.detail);
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
    <IconSegmentedControl
        options={toolOptions}
        value={mode}
        on:change={handleModeChange}
    />

    <div class="w-px h-6 bg-base-300"></div>

    <!-- Load Image / Directory -->
    <RawButton
        icon="image"
        label="Load Image"
        on:click={() => dispatch("triggerFileInput")}
    />
    <RawButton
        icon="folder_open"
        label="Load Directory"
        on:click={() => dispatch("triggerDirectoryInput")}
    />
    <RawButton
        icon="science"
        label="Mock"
        class="btn-ghost text-primary hover:bg-primary/10"
        on:click={() => dispatch("triggerMockLoad")}
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
            on:input={(e) => dispatch('strokeWidthChange', parseFloat(e.currentTarget.value))}
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
        on:click={() => dispatch('toggleVertexPoints')}
    >
        <span class="material-symbols-rounded text-[18px]">scatter_plot</span>
    </button>

    <!-- Polygon Actions (contextual) -->
    {#if mode === "polygon" && isPolygonDrawing}
        <div class="w-px h-6 bg-base-300"></div>
        <div class="flex items-center gap-2">
            <RawButton
                icon="check_circle"
                label="Finish"
                class="btn-success text-success-content hover:bg-success hover:text-success-content"
                on:click={() => dispatch("finishPolygon")}
            />
            <RawButton
                icon="cancel"
                label="Cancel"
                class="btn-error text-error-content hover:bg-error hover:text-error-content"
                on:click={() => dispatch("resetPolygon")}
            />
        </div>
    {/if}

    <!-- Polyline Actions (contextual) -->
    {#if mode === "polyline" && isPolylineDrawing}
        <div class="w-px h-6 bg-base-300"></div>
        <div class="flex items-center gap-2">
            <RawButton
                icon="check_circle"
                label="Finish"
                class="btn-success text-success-content hover:bg-success hover:text-success-content"
                on:click={() => dispatch("finishPolyline")}
            />
            <RawButton
                icon="cancel"
                label="Cancel"
                class="btn-error text-error-content hover:bg-error hover:text-error-content"
                on:click={() => dispatch("resetPolyline")}
            />
        </div>
    {/if}
</div>

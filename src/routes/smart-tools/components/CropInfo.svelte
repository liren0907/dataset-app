<script lang="ts">
    export let cropArea: {
        x: number;
        y: number;
        width: number;
        height: number;
    };
</script>

<div class="stats stats-vertical shadow w-full">
    <div class="stat py-2">
        <div class="stat-title text-xs">Position</div>
        <div class="stat-value text-sm">
            {Math.round(cropArea.x)}, {Math.round(cropArea.y)}
        </div>
    </div>
    <div class="stat py-2">
        <div class="stat-title text-xs">Size</div>
        <div class="stat-value text-sm">
            {Math.round(cropArea.width)} × {Math.round(cropArea.height)}
        </div>
    </div>
    <div class="stat py-2">
        <div class="stat-title text-xs">Ratio</div>
        <div class="stat-value text-sm">
            {(cropArea.width / cropArea.height).toFixed(2)}
        </div>
    </div>
</div>

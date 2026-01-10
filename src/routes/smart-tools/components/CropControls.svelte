<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let aspectRatio: string;
    export let zoom: number;
    export let rotation: number;

    const dispatch = createEventDispatcher();

    function setAspectRatio(e: Event) {
        const target = e.target as HTMLSelectElement;
        dispatch("ratioChange", target.value);
    }
</script>

<div class="card bg-base-200 shadow-sm">
    <div class="card-body p-4">
        <h3 class="card-title text-base flex items-center gap-2">
            <span class="material-symbols-rounded text-primary">tune</span>
            Crop Controls
        </h3>

        <!-- Aspect Ratio -->
        <div class="form-control">
            <div class="label">
                <span class="label-text">Aspect Ratio</span>
            </div>
            <select
                value={aspectRatio}
                on:change={setAspectRatio}
                class="select select-bordered select-sm w-full"
            >
                <option value="free">Free Form</option>
                <option value="square">Square (1:1)</option>
                <option value="4:3">Landscape (4:3)</option>
                <option value="16:9">Wide (16:9)</option>
                <option value="3:2">Photo (3:2)</option>
                <option value="2:3">Portrait (2:3)</option>
                <option value="9:16">Tall (9:16)</option>
            </select>
        </div>

        <!-- Zoom Controls -->
        <div class="form-control">
            <div class="label"><span class="label-text">Zoom</span></div>
            <div class="join">
                <button
                    on:click={() => dispatch("zoomOut")}
                    class="btn btn-sm join-item">-</button
                >
                <span class="btn btn-sm join-item btn-disabled"
                    >{Math.round(zoom * 100)}%</span
                >
                <button
                    on:click={() => dispatch("zoomIn")}
                    class="btn btn-sm join-item">+</button
                >
                <button
                    on:click={() => dispatch("zoomReset")}
                    class="btn btn-sm join-item">Reset</button
                >
            </div>
        </div>

        <!-- Rotation Controls -->
        <div class="form-control">
            <div class="label"><span class="label-text">Rotation</span></div>
            <div class="join">
                <button
                    on:click={() => dispatch("rotateCCW")}
                    class="btn btn-sm join-item"
                >
                    <span class="material-symbols-rounded icon-sm"
                        >rotate_left</span
                    >
                </button>
                <span class="btn btn-sm join-item btn-disabled"
                    >{rotation}°</span
                >
                <button
                    on:click={() => dispatch("rotateCW")}
                    class="btn btn-sm join-item"
                >
                    <span class="material-symbols-rounded icon-sm"
                        >rotate_right</span
                    >
                </button>
            </div>
        </div>

        <!-- Action Buttons -->
        <div class="space-y-2 mt-4">
            <button
                on:click={() => dispatch("preview")}
                class="btn btn-primary btn-block"
            >
                <span class="material-symbols-rounded">visibility</span>
                Preview Crop
            </button>

            <div class="grid grid-cols-2 gap-2">
                <button
                    on:click={() => dispatch("reset")}
                    class="btn btn-ghost btn-sm"
                >
                    <span class="material-symbols-rounded icon-sm"
                        >restart_alt</span
                    >
                    Reset
                </button>
                <button
                    on:click={() => dispatch("clear")}
                    class="btn btn-error btn-sm"
                >
                    <span class="material-symbols-rounded icon-sm">delete</span>
                    Clear
                </button>
            </div>
        </div>
    </div>
</div>

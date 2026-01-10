<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let label: string;
    export let value: string | null = null;
    export let placeholder: string = "Select directory...";

    const dispatch = createEventDispatcher();

    function onBrowse() {
        dispatch("browse");
    }
</script>

<div>
    <label class="block text-sm font-medium text-gray-700 mb-1">{label}</label>
    <div class="flex items-center gap-2">
        <input
            type="text"
            readonly
            {placeholder}
            value={value || ""}
            class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-600 text-sm truncate"
        />
        <button
            on:click={onBrowse}
            class="px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md border border-gray-300 text-sm"
        >
            Browse...
        </button>
    </div>
</div>

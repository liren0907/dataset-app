<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { listen } from "@tauri-apps/api/event";

    let isDragHover: boolean = false;

    let unlistenDrop: (() => void) | null = null;
    let unlistenHover: (() => void) | null = null;
    let unlistenCancel: (() => void) | null = null;

    onMount(async () => {
        unlistenDrop = await listen("tauri://file-drop", async () => {
            isDragHover = false;
        });
        unlistenHover = await listen("tauri://file-drop-hover", async () => {
            isDragHover = true;
        });
        unlistenCancel = await listen("tauri://file-drop-cancelled", () => {
            isDragHover = false;
        });
    });

    onDestroy(() => {
        unlistenDrop?.();
        unlistenHover?.();
        unlistenCancel?.();
    });
</script>

<svelte:head>
    <title>Dataset App - Home</title>
    <meta name="description" content="A modern dataset application" />
</svelte:head>

<div class="container mx-auto px-6 py-8 md:py-12 max-w-6xl">
    <!-- Welcome Header -->
    <div
        class="flex flex-col items-center md:items-start mb-12 animate-in fade-in slide-in-from-bottom-2 duration-500"
    >
        <h1 class="text-3xl font-bold text-base-content mb-2">Welcome Back</h1>
        <p class="text-base text-base-content/60">
            Manage and process your datasets efficiently.
        </p>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8 items-start">
        <!-- Left: Quick Import Zone -->
        <div class="lg:col-span-2 space-y-8">
            <!-- Drop Zone Panel (Flat) -->
            <div
                class="card bg-base-100 border transition-all duration-200 relative group overflow-hidden
                {isDragHover
                    ? 'border-primary bg-primary/5'
                    : 'border-base-300 hover:border-primary/50'}"
            >
                <div class="card-body items-center text-center py-16">
                    {#if isDragHover}
                        <div
                            class="w-16 h-16 rounded-lg bg-primary/10 flex items-center justify-center mb-4 border border-primary/20"
                        >
                            <span
                                class="material-symbols-rounded text-4xl text-primary"
                                >folder_open</span
                            >
                        </div>
                        <h2 class="text-xl font-bold text-primary">
                            Drop to Import
                        </h2>
                        <p class="text-sm text-base-content/60">
                            Release your files to start processing
                        </p>
                    {:else}
                        <div
                            class="w-16 h-16 rounded-lg bg-base-200 flex items-center justify-center mb-4 group-hover:bg-base-300 transition-colors border border-base-300"
                        >
                            <span
                                class="material-symbols-rounded text-4xl text-base-content/40 group-hover:text-base-content/60 transition-colors"
                                >cloud_upload</span
                            >
                        </div>
                        <h2 class="text-lg font-semibold text-base-content">
                            Import Dataset
                        </h2>
                        <p
                            class="text-sm text-base-content/50 max-w-xs mx-auto mt-1"
                        >
                            Drag and drop folders containing images, videos, or
                            label files here to get started.
                        </p>

                        <!-- Manual Action -->
                        <div class="mt-6">
                            <button
                                class="btn btn-primary btn-sm gap-2 normal-case no-animation rounded-md"
                            >
                                <span class="material-symbols-rounded">add</span
                                >
                                Select Folder
                            </button>
                        </div>
                    {/if}
                </div>
            </div>

            <!-- Recent Activity or Stats Placeholder (Optional, for dashboard feel) -->
            <!-- 
            <div class="border border-base-200 rounded-xl p-6 bg-base-100">
                <h3 class="font-semibold text-sm text-base-content/70 uppercase tracking-wide mb-4">Recent Activity</h3>
                <div class="text-sm text-base-content/40 italic">No recent datasets found.</div>
            </div> 
            -->
        </div>

        <!-- Right: Tools Navigation (Flat) -->
        <div
            class="space-y-3 animate-in fade-in slide-in-from-right-4 duration-500 delay-100"
        >
            <div class="flex items-center justify-between mb-2">
                <span
                    class="text-xs font-bold text-base-content/40 uppercase tracking-wider"
                    >Quick Access</span
                >
            </div>

            <!-- Card 1: Gallery -->
            <a
                href="/gallery"
                class="card bg-base-100 border border-base-200 hover:border-primary/40 hover:bg-base-50 transition-all duration-200 group rounded-lg"
            >
                <div class="card-body p-4 flex-row items-center gap-4">
                    <div
                        class="w-10 h-10 rounded-md bg-primary/10 flex items-center justify-center text-primary group-hover:bg-primary group-hover:text-primary-content transition-colors border border-primary/10"
                    >
                        <span class="material-symbols-rounded text-xl"
                            >photo_library</span
                        >
                    </div>
                    <div class="flex-1 min-w-0">
                        <h2
                            class="font-bold text-base-content text-sm group-hover:text-primary transition-colors"
                        >
                            Unified Gallery
                        </h2>
                        <p class="text-xs text-base-content/50 truncate">
                            Browse and manage image datasets
                        </p>
                    </div>
                </div>
            </a>

            <!-- Card 2: Turbo Export -->
            <a
                href="/turbo-export"
                class="card bg-base-100 border border-base-200 hover:border-secondary/40 hover:bg-base-50 transition-all duration-200 group rounded-lg"
            >
                <div class="card-body p-4 flex-row items-center gap-4">
                    <div
                        class="w-10 h-10 rounded-md bg-secondary/10 flex items-center justify-center text-secondary group-hover:bg-secondary group-hover:text-secondary-content transition-colors border border-secondary/10"
                    >
                        <span class="material-symbols-rounded text-xl"
                            >rocket_launch</span
                        >
                    </div>
                    <div class="flex-1 min-w-0">
                        <h2
                            class="font-bold text-base-content text-sm group-hover:text-secondary transition-colors"
                        >
                            Turbo Export
                        </h2>
                        <p class="text-xs text-base-content/50 truncate">
                            High-speed model export tools
                        </p>
                    </div>
                </div>
            </a>

            <!-- Card 3: Smart Tools -->
            <a
                href="/smart-tools"
                class="card bg-base-100 border border-base-200 hover:border-accent/40 hover:bg-base-50 transition-all duration-200 group rounded-lg"
            >
                <div class="card-body p-4 flex-row items-center gap-4">
                    <div
                        class="w-10 h-10 rounded-md bg-accent/10 flex items-center justify-center text-accent group-hover:bg-accent group-hover:text-accent-content transition-colors border border-accent/10"
                    >
                        <span class="material-symbols-rounded text-xl"
                            >auto_awesome</span
                        >
                    </div>
                    <div class="flex-1 min-w-0">
                        <h2
                            class="font-bold text-base-content text-sm group-hover:text-accent transition-colors"
                        >
                            Smart Tools
                        </h2>
                        <p class="text-xs text-base-content/50 truncate">
                            AI auto-labeling utilities
                        </p>
                    </div>
                </div>
            </a>
        </div>
    </div>
</div>

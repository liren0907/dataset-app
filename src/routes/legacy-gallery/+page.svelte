<script lang="ts">
    // Legacy Gallery Hub - Navigation to older gallery implementations

    const legacyPages = [
        {
            href: "/legacy-gallery/dataset-gallery",
            title: "Dataset Gallery",
            icon: "photo_library",
            description:
                "Basic dataset browsing with pagination and export functionality.",
            features: [
                "Directory selection",
                "Grid/List view",
                "Export to YOLO",
                "Crop & Remap tool",
            ],
        },
        {
            href: "/legacy-gallery/dataset-gallery-advanced",
            title: "Advanced Gallery",
            icon: "palette",
            description:
                "Enhanced gallery with modal annotation viewer and auto-loading.",
            features: [
                "Modal annotation editor",
                "KonvaJS canvas",
                "Auto-load annotations",
                "Annotation type toggle",
            ],
        },
        {
            href: "/legacy-gallery/imageViewer3",
            title: "Image Viewer",
            icon: "visibility",
            description:
                "Image viewer with integrated crop, remap, and extraction tools.",
            features: [
                "Crop & Remap",
                "Extract labels",
                "Export dataset",
                "Dataset summary",
            ],
        },
        {
            href: "/legacy-gallery/crop-remap",
            title: "Standalone Crop Tool",
            icon: "crop",
            description:
                "Dedicated page for advanced crop and remap operations (Legacy).",
            features: [
                "Parent/Child Label",
                "Padding Control",
                "Batch Processing",
                "Dynamic Analysis",
            ],
        },
        {
            href: "/legacy-gallery/crop-remap-component",
            title: "Standalone Crop Component",
            icon: "straighten",
            description:
                "A test page for the standalone crop and remap component UI (Legacy).",
            features: ["Dataset Analysis", "Label Config", "Konva Preview"],
        },
    ];
</script>

<svelte:head>
    <title>Legacy Gallery | Dataset App</title>
    <meta name="description" content="Access older gallery implementations" />
</svelte:head>

<div class="container mx-auto px-4 py-8">
    <div class="max-w-4xl mx-auto">
        <!-- Header -->
        <div class="mb-8">
            <div class="flex items-center gap-3 mb-2">
                <span
                    class="material-symbols-rounded text-3xl text-base-content/50"
                    >history</span
                >
                <h1 class="text-3xl font-bold text-base-content">
                    Legacy Gallery
                </h1>
            </div>
            <p class="text-base-content/60 ml-12">
                These are older gallery implementations preserved for reference.
                For the best experience, use the <a
                    href="/gallery"
                    class="link link-primary font-medium">Unified Gallery</a
                >.
            </p>
        </div>

        <!-- Info Alert -->
        <div class="alert alert-info mb-8 shadow-sm">
            <span class="material-symbols-rounded">info</span>
            <div>
                <h3 class="font-bold">Why Legacy?</h3>
                <p class="text-sm">
                    These pages represent earlier development iterations.
                    Features from these pages have been consolidated into the
                    main <strong>Unified Gallery</strong> page.
                </p>
            </div>
        </div>

        <!-- Navigation Cards -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            {#each legacyPages as page}
                <a
                    href={page.href}
                    class="card bg-base-100 border border-base-200 shadow-sm hover:shadow-lg hover:border-primary/30 transition-all duration-300 group"
                >
                    <div class="card-body">
                        <!-- Icon & Title -->
                        <div class="flex items-center gap-3 mb-2">
                            <div
                                class="w-12 h-12 rounded-xl bg-base-200 group-hover:bg-primary/10 flex items-center justify-center transition-colors"
                            >
                                <span
                                    class="material-symbols-rounded text-2xl text-base-content/60 group-hover:text-primary transition-colors"
                                >
                                    {page.icon}
                                </span>
                            </div>
                            <h2 class="card-title text-lg">{page.title}</h2>
                        </div>

                        <!-- Description -->
                        <p class="text-sm text-base-content/60 mb-4">
                            {page.description}
                        </p>

                        <!-- Features -->
                        <div class="flex flex-wrap gap-1.5">
                            {#each page.features as feature}
                                <span
                                    class="badge badge-ghost badge-sm text-xs"
                                >
                                    {feature}
                                </span>
                            {/each}
                        </div>

                        <!-- Arrow indicator -->
                        <div class="card-actions justify-end mt-4">
                            <span
                                class="material-symbols-rounded text-base-content/30 group-hover:text-primary group-hover:translate-x-1 transition-all"
                            >
                                arrow_forward
                            </span>
                        </div>
                    </div>
                </a>
            {/each}
        </div>

        <!-- Footer Note -->
        <div class="mt-8 text-center">
            <p class="text-sm text-base-content/40">
                <span class="material-symbols-rounded text-sm align-middle mr-1"
                    >lightbulb</span
                >
                Tip: The Unified Gallery combines all features from these legacy
                pages.
            </p>
        </div>
    </div>
</div>

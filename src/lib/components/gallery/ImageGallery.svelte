<script lang="ts">
    import { onMount } from "svelte";

    let {
        images = [],
        totalImages = 0,
        currentPage = 1,
        totalPages = 0,
        pageSize = 30,
        viewMode = "grid",
        annotationType = "bounding_box",
        loading = false,
        loadingMore = false,
        selectedImage = null,
        onimageclick,
        onloadpage,
    }: {
        images?: any[];
        totalImages?: number;
        currentPage?: number;
        totalPages?: number;
        pageSize?: number;
        viewMode?: "grid" | "column";
        annotationType?: string;
        loading?: boolean;
        loadingMore?: boolean;
        selectedImage?: any;
        onimageclick?: (data: { image: any; index: number }) => void;
        onloadpage?: (page: number) => void;
    } = $props();

    // Image preloading for faster annotation editor loading
    let preloadCache = new Map<string, boolean>();
    let preloadTimeout: number | null = null;

    function preloadImage(imagePath: string): void {
        if (preloadCache.has(imagePath)) return;

        const img = new Image();
        img.onload = () => {
            preloadCache.set(imagePath, true);
            console.log(`Preloaded image: ${imagePath}`);
        };
        img.onerror = () => {
            console.warn(`Failed to preload image: ${imagePath}`);
        };
        img.src = imagePath;
    }

    // Preload annotated preview for faster modal loading
    async function preloadAnnotatedPreview(image: any): Promise<void> {
        if (!image?.path || preloadCache.has(`annotated_${image.path}`)) return;

        try {
            console.log(`Preloading annotated preview for: ${image.name}`);
            if (image.previewUrl) {
                preloadImage(image.previewUrl);
            }
            preloadCache.set(`annotated_${image.path}`, true);
        } catch (error) {
            console.warn(
                `Failed to preload annotated preview for ${image.name}:`,
                error,
            );
        }
    }

    function handleImageHover(image: any): void {
        if (preloadTimeout) {
            clearTimeout(preloadTimeout);
        }
        preloadTimeout = setTimeout(() => {
            preloadAnnotatedPreview(image);
        }, 500);
    }

    function handleImageLeave(): void {
        if (preloadTimeout) {
            clearTimeout(preloadTimeout);
            preloadTimeout = null;
        }
    }

    // --- Helper Functions ---
    function formatFileSize(bytes: number | null | undefined) {
        if (bytes === null || bytes === undefined) return "";
        if (bytes < 1024) return bytes + " B";
        if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
        return (bytes / 1048576).toFixed(1) + " MB";
    }

    function generatePageNumbers(current: number, total: number) {
        let pages: (number | string)[] = [];
        if (total <= 1) return [1];
        pages.push(1);
        if (current > 3) pages.push("ellipsis_start");
        if (current > 2 && current - 1 !== 1) pages.push(current - 1);
        if (current !== 1 && current !== total) pages.push(current);
        if (current < total - 1 && current + 1 !== total)
            pages.push(current + 1);
        if (current < total - 2) pages.push("ellipsis_end");
        if (total > 1) pages.push(total);

        let uniquePages: (number | string)[] = [];
        let lastPushed: number | string | null = null;
        for (const p of pages) {
            if (
                (p === "ellipsis_start" || p === "ellipsis_end") &&
                (lastPushed === "ellipsis_start" ||
                    lastPushed === "ellipsis_end")
            )
                continue;
            uniquePages.push(p);
            lastPushed = p;
        }
        return uniquePages;
    }

    function handleImageClick(image: any, index: number) {
        onimageclick?.({ image, index });
    }

    function handleLoadPage(page: number | string) {
        if (
            typeof page === "number" &&
            page >= 1 &&
            page <= totalPages &&
            page !== currentPage
        ) {
            onloadpage?.(page);
        }
    }

    // --- Enhanced Lazy Loading Strategy ---
    function setupLazyLoading() {
        if (typeof document === "undefined") return;

        const imageElements = document.querySelectorAll(
            `.gallery-lazy-image:not(.observed):not(.loaded)`,
        );

        if (imageElements.length === 0) return;

        const imgObserver = new IntersectionObserver(
            (entries) => {
                entries.forEach((entry) => {
                    if (entry.isIntersecting) {
                        const img = entry.target as HTMLImageElement;
                        const imageSrc = img.getAttribute("data-src");

                        if (
                            imageSrc &&
                            img.src !== imageSrc &&
                            !img.classList.contains("loaded")
                        ) {
                            img.src = imageSrc;
                            img.classList.add("observed");
                        }

                        imgObserver.unobserve(img);
                    }
                });
            },
            {
                rootMargin: "300px 0px",
                threshold: 0.01,
            },
        );

        imageElements.forEach((img) => {
            imgObserver.observe(img);
        });
    }

    // Performance optimization: track if we've already processed this page
    let lastProcessedPage = -1;
    let lastImagesLength = 0;

    onMount(() => {
        setupLazyLoading();
        lastProcessedPage = currentPage;
        lastImagesLength = images.length;
    });

    // Replace afterUpdate with $effect
    $effect(() => {
        // Track dependencies
        const _page = currentPage;
        const _len = images.length;

        if (_page !== lastProcessedPage || _len !== lastImagesLength) {
            setupLazyLoading();
            lastProcessedPage = _page;
            lastImagesLength = _len;
        }
    });
</script>

{#if images.length > 0}
    <div class="image-gallery-container">
        <!-- Grid View -->
        {#if viewMode === "grid"}
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
                {#each images as image, i (image.path)}
                    <button
                        type="button"
                        class={`group relative card bg-base-100 shadow-sm hover:shadow-md transition-all duration-200 cursor-pointer overflow-hidden border ${selectedImage?.path === image.path ? "border-primary ring-2 ring-primary ring-offset-2" : "border-transparent"}`}
                        onclick={() => handleImageClick(image, i)}
                        onmouseenter={() => handleImageHover(image)}
                        onmouseleave={handleImageLeave}
                        aria-label={`View details for ${image.name}`}
                        title={`${image.name}\nSize: ${formatFileSize(image.size)}\nDimensions: ${image.dimensions ? `${image.dimensions.width}x${image.dimensions.height}` : "Unknown"}`}
                    >
                        <figure class="relative w-full pb-[75%] bg-base-200">
                            <!-- Image -->
                            <img
                                class={`gallery-lazy-image absolute inset-0 w-full h-full object-cover transition-transform duration-500 group-hover:scale-105 ${image.displayIndex < pageSize ? "opacity-100" : "opacity-0"}`}
                                data-src={image.previewUrl}
                                alt={image.name}
                                src={image.displayIndex < pageSize
                                    ? image.previewUrl
                                    : "data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 1 1%27%3E%3C/svg%3E"}
                                onload={(e) => {
                                    const target = e.target;
                                    if (
                                        target instanceof HTMLImageElement &&
                                        target.src !==
                                            "data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 1 1%27%3E%3C/svg%3E"
                                    ) {
                                        target.classList.remove("opacity-0");
                                        target.classList.add("loaded");
                                        target.classList.add("observed");
                                    }
                                }}
                                onerror={(e) => {
                                    const target = e.target;
                                    if (target instanceof HTMLImageElement) {
                                        target.classList.add("observed");
                                        target.classList.add("loaded");
                                    }
                                }}
                            />

                            <!-- Overlay Gradient -->
                            <div
                                class="absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-black/80 via-black/40 to-transparent opacity-60 group-hover:opacity-80 transition-opacity duration-300"
                            ></div>

                            <!-- Content Overlay -->
                            <div
                                class="absolute inset-x-0 bottom-0 p-3 flex flex-col justify-end text-left"
                            >
                                <div
                                    class="flex items-end justify-between gap-2"
                                >
                                    <span
                                        class="text-xs font-medium text-white/90 truncate drop-shadow-md select-none"
                                    >
                                        {image.name}
                                    </span>

                                    <!-- Status Badges -->
                                    <div class="flex gap-1 flex-shrink-0">
                                        {#if image.hasJson}
                                            <span
                                                class="w-2 h-2 rounded-full bg-info shadow-sm"
                                                title="Has JSON"
                                            ></span>
                                        {/if}
                                        {#if image.annotated}
                                            <span
                                                class={`badge badge-sm border-0 h-5 px-1.5 text-[10px] font-bold shadow-sm ${annotationType === "bounding_box" ? "bg-success text-success-content" : "bg-warning text-warning-content"}`}
                                            >
                                                {image.annotations.length}
                                            </span>
                                        {/if}
                                    </div>
                                </div>
                            </div>
                        </figure>
                    </button>
                {/each}
            </div>
        {:else}
            <!-- Column View -->
            <div class="space-y-4">
                {#each images as image, i (image.path)}
                    <button
                        type="button"
                        class={`card card-side bg-base-100 shadow-sm hover:shadow-md transition-all duration-200 cursor-pointer border w-full hover:-translate-y-0.5 ${selectedImage?.path === image.path ? "border-primary ring-2 ring-primary ring-offset-2" : "border-base-300"}`}
                        onclick={() => handleImageClick(image, i)}
                        onmouseenter={() => handleImageHover(image)}
                        onmouseleave={handleImageLeave}
                        aria-label={`View details for ${image.name}`}
                    >
                        <figure class="sm:w-48 h-48 relative">
                            <img
                                class={`gallery-lazy-image w-full h-full object-cover transition-opacity duration-300 ${image.displayIndex < pageSize ? "opacity-100" : "opacity-0"}`}
                                data-src={image.previewUrl}
                                alt={image.name}
                                src={image.displayIndex < pageSize
                                    ? image.previewUrl
                                    : "data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 1 1%27%3E%3C/svg%3E"}
                                onload={(e) => {
                                    const target = e.target;
                                    if (
                                        target instanceof HTMLImageElement &&
                                        target.src !==
                                            "data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 1 1%27%3E%3C/svg%3E"
                                    ) {
                                        target.classList.remove("opacity-0");
                                        target.classList.add("loaded");
                                        target.classList.add("observed");
                                    }
                                }}
                                onerror={(e) => {
                                    const target = e.target;
                                    if (target instanceof HTMLImageElement) {
                                        target.classList.add("observed");
                                        target.classList.add("loaded");
                                    }
                                }}
                            />
                            <div
                                class="absolute top-2 right-2 flex flex-col gap-1"
                            >
                                {#if image.hasJson}
                                    <span class="badge badge-info badge-sm"
                                        >JSON</span
                                    >
                                {/if}
                                {#if image.annotated}
                                    <span class="badge badge-success badge-sm">
                                        {annotationType === "bounding_box"
                                            ? "Boxes"
                                            : "Polygons"}
                                    </span>
                                {/if}
                            </div>
                        </figure>
                        <div class="card-body p-4">
                            <h3 class="card-title text-base text-base-content">
                                {image.name}
                            </h3>
                            {#if image.size != null}
                                <p class="text-sm text-base-content/60">
                                    Size: {formatFileSize(image.size)}
                                </p>
                            {/if}
                            {#if image.dimensions}
                                <p class="text-sm text-base-content/60">
                                    Dimensions: {image.dimensions.width} x {image
                                        .dimensions.height}
                                </p>
                            {/if}
                            {#if image.created}
                                <p class="text-sm text-base-content/60">
                                    Created: {new Date(
                                        image.created,
                                    ).toLocaleString()}
                                </p>
                            {/if}
                            {#if image.annotations && image.annotations.length > 0}
                                <div class="mt-2">
                                    <p class="text-sm font-medium text-success">
                                        {image.annotations.length}
                                        {annotationType === "bounding_box"
                                            ? "bounding box"
                                            : "polygon"}{image.annotations
                                            .length !== 1
                                            ? "es"
                                            : ""}:
                                    </p>
                                    <ul
                                        class="mt-1 list-disc list-inside text-sm text-base-content/70"
                                    >
                                        {#each image.annotations.slice(0, 3) as annotation, annotationIndex (annotationIndex)}
                                            <li class="truncate">
                                                {annotation.label}
                                            </li>
                                        {/each}
                                        {#if image.annotations.length > 3}
                                            <li class="text-base-content/50">
                                                +{image.annotations.length - 3} more
                                            </li>
                                        {/if}
                                    </ul>
                                </div>
                            {/if}
                        </div>
                    </button>
                {/each}
            </div>
        {/if}

        <!-- Pagination -->
        {#if !loading && images.length > 0 && totalPages > 1}
            <div class="flex items-center justify-center my-8">
                <div class="join" aria-label="Pagination">
                    <button
                        type="button"
                        class="join-item btn btn-sm"
                        onclick={() => handleLoadPage(currentPage - 1)}
                        disabled={currentPage === 1 || loading || loadingMore}
                    >
                        &laquo;
                    </button>
                    {#each generatePageNumbers(currentPage, totalPages) as pageNum (typeof pageNum === "number" ? pageNum : pageNum)}
                        {#if typeof pageNum === "number"}
                            <button
                                type="button"
                                class={`join-item btn btn-sm ${currentPage === pageNum ? "btn-active" : "btn-ghost border border-neutral"}`}
                                onclick={() => handleLoadPage(pageNum)}
                                disabled={loading ||
                                    loadingMore ||
                                    pageNum === currentPage}
                            >
                                {pageNum}
                            </button>
                        {:else}
                            <button class="join-item btn btn-sm btn-disabled"
                                >...</button
                            >
                        {/if}
                    {/each}
                    <button
                        type="button"
                        class="join-item btn btn-sm"
                        onclick={() => handleLoadPage(currentPage + 1)}
                        disabled={currentPage === totalPages ||
                            loading ||
                            loadingMore}
                    >
                        &raquo;
                    </button>
                </div>
            </div>
        {/if}

        {#if loadingMore}
            <div class="h-10 my-8 flex justify-center">
                <span class="loading loading-spinner loading-md text-primary"
                ></span>
            </div>
        {/if}
    </div>
{:else if !loading && totalImages === 0}
    <!-- No images placeholder handled by parent page -->
{/if}

<style>
    /* Enhanced shimmer effect - optimized version from Optimized Gallery */
    .gallery-lazy-image {
        transition: opacity 0.3s ease-in-out;
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .gallery-lazy-image.loaded {
        opacity: 1;
    }

    .gallery-lazy-image.opacity-0 {
        opacity: 0;
    }

    /* Enhanced shimmer effect with better performance and transitions */
    .gallery-lazy-image:not(.loaded) {
        background: linear-gradient(
            90deg,
            rgba(229, 232, 235, 0.6) 25%,
            rgba(215, 219, 223, 0.6) 37%,
            rgba(229, 232, 235, 0.6) 63%
        );
        background-size: 400% 100%;
        animation: shimmer 1.4s ease infinite;
    }

    /* Enhanced loading states for smoother transitions */
    .gallery-lazy-image.loaded {
        animation: none;
        background: none;
    }

    /* Ensure smooth transitions between states */
    .gallery-lazy-image.observed:not(.loaded) {
        animation: shimmer 1.4s ease infinite;
    }

    @keyframes shimmer {
        0% {
            background-position: 100% 50%;
        }
        100% {
            background-position: 0 50%;
        }
    }
</style>

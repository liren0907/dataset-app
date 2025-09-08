<script lang="ts">
    import { createEventDispatcher, tick, onMount, afterUpdate } from 'svelte';
    // import { convertFileSrc } from "@tauri-apps/api/core"; // Not strictly needed if previewUrl is always pre-converted by parent

    export let images: any[] = [];
    export let totalImages: number = 0;
    export let currentPage: number = 1;
    export let totalPages: number = 0;
    export let pageSize: number = 30; // Used for smart loading strategy
    export let viewMode: 'grid' | 'column' = 'grid';
    export let annotationType: string = 'bounding_box';
    export let loading: boolean = false;
    export let loadingMore: boolean = false;

    const dispatch = createEventDispatcher();

    // Image preloading for faster annotation editor loading
    let preloadCache = new Map<string, boolean>();
    let preloadTimeout: number | null = null;

    function preloadImage(imagePath: string): void {
        if (preloadCache.has(imagePath)) return;

        const img = new Image();
        img.onload = () => {
            preloadCache.set(imagePath, true);
            console.log(`✅ Preloaded image: ${imagePath}`);
        };
        img.onerror = () => {
            console.warn(`❌ Failed to preload image: ${imagePath}`);
        };
        img.src = imagePath;
    }

    // Preload annotated preview for faster modal loading
    async function preloadAnnotatedPreview(image: any): Promise<void> {
        if (!image?.path || preloadCache.has(`annotated_${image.path}`)) return;

        try {
            // Simulate backend call for preloading (in real implementation, this would call the backend)
            console.log(`🔄 Preloading annotated preview for: ${image.name}`);

            // For now, just preload the regular image
            // In full implementation, this would preload the annotated version
            if (image.previewUrl) {
                preloadImage(image.previewUrl);
            }

            preloadCache.set(`annotated_${image.path}`, true);

        } catch (error) {
            console.warn(`⚠️ Failed to preload annotated preview for ${image.name}:`, error);
        }
    }

    function handleImageHover(image: any): void {
        // Clear any existing timeout
        if (preloadTimeout) {
            clearTimeout(preloadTimeout);
        }

        // Start preloading after a short delay (500ms) to avoid unnecessary preloading
        preloadTimeout = setTimeout(() => {
            preloadAnnotatedPreview(image);
        }, 500);
    }

    function handleImageLeave(): void {
        // Clear preload timeout if user moves away quickly
        if (preloadTimeout) {
            clearTimeout(preloadTimeout);
            preloadTimeout = null;
        }
    }

    // --- Helper Functions (can be moved from +page.svelte or kept if only used here) ---
    function formatFileSize(bytes) {
        if (bytes === null || bytes === undefined) return '';
        if (bytes < 1024) return bytes + " B";
        if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
        return (bytes / 1048576).toFixed(1) + " MB";
    }

    function generatePageNumbers(current, total) {
        let pages: (number | string)[] = [];
        if (total <= 1) return [1];
        pages.push(1);
        if (current > 3) pages.push("ellipsis_start"); // Use unique string for ellipsis
        if (current > 2 && current -1 !== 1) pages.push(current - 1);
        if (current !== 1 && current !== total) pages.push(current);
        if (current < total - 1 && current + 1 !== total) pages.push(current + 1);
        if (current < total - 2) pages.push("ellipsis_end"); // Use unique string for ellipsis
        if (total > 1) pages.push(total);

        let uniquePages: (number | string)[] = [];
        let lastPushed: (number | string | null) = null;
        for (const p of pages) {
            if ((p === "ellipsis_start" || p === "ellipsis_end") && (lastPushed === "ellipsis_start" || lastPushed === "ellipsis_end")) continue;
            uniquePages.push(p);
            lastPushed = p;
        }
        return uniquePages;
    }

    function handleImageClick(image: any, index: number) {
        dispatch('selectImage', { image, index });
    }

    function handleLoadPage(page: number | string) {
        if (typeof page === 'number' && page >= 1 && page <= totalPages && page !== currentPage) {
            dispatch('loadPage', page);
        }
    }

    // --- Enhanced Lazy Loading Strategy --- (Optimized version)
    // Improved intersection observer with better preloading and state management
    function setupLazyLoading() {
        if (typeof document === 'undefined') return; // Guard for SSR

        // Only select images that need lazy loading (not first page, not already processed)
        const imageElements = document.querySelectorAll(`.gallery-lazy-image:not(.observed):not(.loaded)`);

        if (imageElements.length === 0) return; // No images to process

        const imgObserver = new IntersectionObserver(
            (entries) => {
                entries.forEach((entry) => {
                    if (entry.isIntersecting) {
                        const img = entry.target as HTMLImageElement;
                        const imageSrc = img.getAttribute("data-src");

                        // Enhanced loading logic with better state management
                        if (imageSrc && img.src !== imageSrc && !img.classList.contains('loaded')) {
                            img.src = imageSrc;
                            img.classList.add("observed"); // Mark as observed to prevent re-processing
                        }

                        imgObserver.unobserve(img); // Stop observing this image
                    }
                });
            },
            {
                rootMargin: "300px 0px", // Increased for better preloading distance
                threshold: 0.01
            }
        );

        // Observe all eligible images
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

    afterUpdate(() => {
        // Only re-run lazy loading if page changed or new images were added
        if (currentPage !== lastProcessedPage || images.length !== lastImagesLength) {
            setupLazyLoading();
            lastProcessedPage = currentPage;
            lastImagesLength = images.length;
        }
    });

</script>

{#if images.length > 0}
    <div class="image-gallery-container">
        <!-- Grid View -->
        {#if viewMode === "grid"}
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
                {#each images as image, i (image.path)}
                    <button type="button"
                        class="bg-white/80 backdrop-blur rounded-xl shadow-sm overflow-hidden hover:shadow-md transition-all duration-200 cursor-pointer relative text-left p-0 border border-slate-200/60 block w-full hover:-translate-y-0.5"
                        on:click={() => handleImageClick(image, i)}
                        on:mouseenter={() => handleImageHover(image)}
                        on:mouseleave={handleImageLeave}
                        aria-label={`View details for ${image.name}`}
                    >
                        <div class="relative pb-[75%]">
                            <img
                                class={`gallery-lazy-image absolute inset-0 w-full h-full object-cover transition-opacity duration-300 ${image.displayIndex < pageSize ? 'opacity-100' : 'opacity-0'}`}
                                data-src={image.previewUrl}
                                alt={image.name}
                                src={image.displayIndex < pageSize ? image.previewUrl : "data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 1 1%27%3E%3C/svg%3E"}
                                on:load={(e) => {
                                    const target = e.target;
                                    if (target instanceof HTMLImageElement && target.src !== "data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 1 1%27%3E%3C/svg%3E") {
                                        // Enhanced state management for smooth transitions
                                        target.classList.remove('opacity-0');
                                        target.classList.add('loaded');
                                        target.classList.add('observed'); // Ensure observed state is set
                                    }
                                }}
                                on:error={(e) => {
                                    // Enhanced error handling - still mark as observed to prevent retries
                                    const target = e.target;
                                    if (target instanceof HTMLImageElement) {
                                        target.classList.add('observed');
                                        target.classList.add('loaded'); // Prevent shimmer on failed loads
                                    }
                                }}
                            />
                            <div class="absolute top-2 right-2 flex flex-col gap-1">
                                {#if image.hasJson}
                                    <div class="bg-blue-600/90 backdrop-blur text-white text-[10px] px-2 py-0.5 rounded-full shadow">JSON</div>
                                {/if}
                                {#if image.annotated}
                                    <div class="bg-green-600/90 backdrop-blur text-white text-[10px] px-2 py-0.5 rounded-full shadow">
                                        {annotationType === "bounding_box" ? "Boxes" : "Polygons"}
                                    </div>
                                {/if}
                            </div>
                        </div>
                        <div class="p-3">
                            <p class="text-sm text-slate-800 truncate">{image.name}</p>
                            {#if image.size != null} <p class="text-xs text-slate-500 mt-1">{formatFileSize(image.size)}</p> {/if}
                            {#if image.annotations && image.annotations.length > 0}
                                <p class="text-xs text-green-700 mt-1">
                                    {image.annotations.length} {annotationType === "bounding_box" ? "box" : "polygon"}{image.annotations.length !== 1 ? "es" : ""}
                                </p>
                            {/if}
                        </div>
                    </button>
                {/each}
            </div>
        {:else}
            <!-- Column View -->
            <div class="space-y-4">
                {#each images as image, i (image.path)}
                    <button type="button"
                        class="bg-white/80 backdrop-blur rounded-xl shadow-sm overflow-hidden hover:shadow-md transition-all duration-200 cursor-pointer relative w-full text-left p-0 border border-slate-200/60 block hover:-translate-y-0.5"
                        on:click={() => handleImageClick(image, i)}
                        on:mouseenter={() => handleImageHover(image)}
                        on:mouseleave={handleImageLeave}
                        aria-label={`View details for ${image.name}`}
                    >
                        <div class="flex flex-col sm:flex-row">
                            <div class="sm:w-48 h-48 relative">
                                <img
                                    class={`gallery-lazy-image w-full h-full object-cover transition-opacity duration-300 ${image.displayIndex < pageSize ? 'opacity-100' : 'opacity-0'}`}
                                    data-src={image.previewUrl}
                                    alt={image.name}
                                    src={image.displayIndex < pageSize ? image.previewUrl : "data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 1 1%27%3E%3C/svg%3E"}
                                     on:load={(e) => {
                                        const target = e.target;
                                        if (target instanceof HTMLImageElement && target.src !== "data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 1 1%27%3E%3C/svg%3E") {
                                            // Enhanced state management for smooth transitions
                                            target.classList.remove('opacity-0');
                                            target.classList.add('loaded');
                                            target.classList.add('observed'); // Ensure observed state is set
                                        }
                                    }}
                                    on:error={(e) => {
                                        // Enhanced error handling - still mark as observed to prevent retries
                                        const target = e.target;
                                        if (target instanceof HTMLImageElement) {
                                            target.classList.add('observed');
                                            target.classList.add('loaded'); // Prevent shimmer on failed loads
                                        }
                                    }}
                                />
                                <div class="absolute top-2 right-2 flex flex-col gap-1">
                                    {#if image.hasJson}
                                        <div class="bg-blue-600/90 backdrop-blur text-white text-[10px] px-2 py-0.5 rounded-full shadow">JSON</div>
                                    {/if}
                                    {#if image.annotated}
                                        <div class="bg-green-600/90 backdrop-blur text-white text-[10px] px-2 py-0.5 rounded-full shadow">
                                            {annotationType === "bounding_box" ? "Boxes" : "Polygons"}
                                        </div>
                                    {/if}
                                </div>
                            </div>
                            <div class="p-4 flex-1">
                                <h3 class="text-lg font-medium text-slate-800">{image.name}</h3>
                                {#if image.size != null} <p class="text-sm text-slate-500 mt-1">Size: {formatFileSize(image.size)}</p> {/if}
                                {#if image.dimensions} <p class="text-sm text-slate-500 mt-1">Dimensions: {image.dimensions.width} × {image.dimensions.height}</p> {/if}
                                {#if image.created} <p class="text-sm text-slate-500 mt-1">Created: {new Date(image.created).toLocaleString()}</p> {/if}
                                {#if image.annotations && image.annotations.length > 0}
                                    <div class="mt-2">
                                        <p class="text-sm font-medium text-green-700">
                                            {image.annotations.length} {annotationType === "bounding_box" ? "bounding box" : "polygon"}{image.annotations.length !== 1 ? "es" : ""}:
                                        </p>
                                        <ul class="mt-1 list-disc list-inside text-sm text-slate-600">
                                            {#each image.annotations.slice(0, 3) as annotation, annotationIndex (annotationIndex)}
                                                <li class="truncate">{annotation.label}</li>
                                            {/each}
                                            {#if image.annotations.length > 3}
                                                <li class="text-slate-500">+{image.annotations.length - 3} more</li>
                                            {/if}
                                        </ul>
                                    </div>
                                {/if}
                            </div>
                        </div>
                    </button>
                {/each}
            </div>
        {/if}

        <!-- Pagination -->
        {#if !loading && images.length > 0 && totalPages > 1}
            <div class="flex items-center justify-center my-8">
                <nav class="relative z-0 inline-flex rounded-md shadow-sm -space-x-px" aria-label="Pagination">
                    <button
                        type="button"
                        class="relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50"
                        on:click={() => handleLoadPage(currentPage - 1)}
                        disabled={currentPage === 1 || loading || loadingMore}
                    >
                        <span class="sr-only">Previous</span>
                        <svg class="h-5 w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true"><path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" /></svg>
                    </button>
                    {#each generatePageNumbers(currentPage, totalPages) as pageNum (typeof pageNum === 'number' ? pageNum : pageNum)}
                        {#if typeof pageNum === 'number'}
                            <button
                                type="button"
                                class="relative inline-flex items-center px-4 py-2 border text-sm font-medium disabled:opacity-50 {currentPage === pageNum ? 'z-10 bg-indigo-50 border-indigo-500 text-indigo-600' : 'bg-white border-gray-300 text-gray-500 hover:bg-gray-50'}"
                                on:click={() => handleLoadPage(pageNum)}
                                disabled={loading || loadingMore || pageNum === currentPage}
                            >
                                {pageNum}
                            </button>
                        {:else}
                            <span class="relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium text-gray-700">...</span>
                        {/if}
                    {/each}
                    <button
                        type="button"
                        class="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50"
                        on:click={() => handleLoadPage(currentPage + 1)}
                        disabled={currentPage === totalPages || loading || loadingMore}
                    >
                        <span class="sr-only">Next</span>
                        <svg class="h-5 w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true"><path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" /></svg>
                    </button>
                </nav>
            </div>
        {/if}

        {#if loadingMore}
            <div class="h-10 my-8 flex justify-center">
                <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-600"></div>
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
        animation: none; /* Stop shimmer animation */
        background: none; /* Remove shimmer background */
    }

    /* Ensure smooth transitions between states */
    .gallery-lazy-image.observed:not(.loaded) {
        /* Keep shimmer while loading */
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

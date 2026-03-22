<script lang="ts">
    import { onMount } from "svelte";

    let {
        images = [],
        viewMode = "grid",
        loading = false,
        loadingMore = false,
        annotationType = "bounding_box",
        currentPage,
        totalPages,
        pageSize = 30,
        onselectimage,
        onloadpage,
    }: {
        images: any[];
        viewMode: "grid" | "column";
        loading: boolean;
        loadingMore: boolean;
        annotationType: string;
        currentPage: number;
        totalPages: number;
        pageSize?: number;
        onselectimage?: (detail: { image: any; index: number }) => void;
        onloadpage?: (page: number) => void;
    } = $props();

    function setupLazyLoading() {
        const imageElements = document.querySelectorAll(".lazy-image:not(.observed)");

        const imgObserver = new IntersectionObserver(
            (entries) => {
                entries.forEach((entry) => {
                    if (entry.isIntersecting) {
                        const img = entry.target as HTMLImageElement;
                        const imageSrc = img.getAttribute("data-src");
                        if (imageSrc) { img.src = imageSrc; }
                        imgObserver.unobserve(img);
                    }
                });
            },
            { rootMargin: "200px 0px", threshold: 0.1 },
        );

        imageElements.forEach((img) => {
            imgObserver.observe(img);
            img.classList.add("observed");
        });
    }

    onMount(() => { setupLazyLoading(); });
    $effect(() => {
        // Re-run lazy loading when images or viewMode change
        images;
        viewMode;
        setupLazyLoading();
    });

    function formatFileSize(bytes: number) {
        if (typeof bytes !== "number") return "";
        if (bytes < 1024) return bytes + " B";
        if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
        return (bytes / 1048576).toFixed(1) + " MB";
    }

    function getVisibleImages() { return images; }

    function generatePageNumbers(current: number, total: number) {
        let pages = [];
        pages.push(1);
        if (current > 3) { pages.push("..."); }
        if (current > 2) { pages.push(current - 1); }
        if (current !== 1 && current !== total) { pages.push(current); }
        if (current < total - 1) { pages.push(current + 1); }
        if (current < total - 2) { pages.push("..."); }
        if (total > 1) { pages.push(total); }
        return [...new Set(pages)];
    }
</script>

<div class="mt-8">
    {#if images.length > 0}
        {#if viewMode === "grid"}
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
                {#each getVisibleImages() as image, index (image.path)}
                    <button
                        type="button"
                        class="bg-white rounded-lg shadow-md overflow-hidden hover:shadow-lg transition-shadow duration-200 cursor-pointer relative text-left p-0 border-none block w-full"
                        onclick={() => onselectimage?.({ image, index })}
                        aria-label={`View details for ${image.name}`}
                    >
                        <div class="relative pb-[75%]">
                            <img
                                class={`absolute inset-0 w-full h-full object-cover transition-opacity duration-300 ${image.displayIndex < pageSize ? "opacity-100" : "lazy-image opacity-0"}`}
                                data-src={image.previewUrl}
                                data-index={index}
                                alt={image.name}
                                src={image.displayIndex < pageSize ? image.previewUrl : "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1 1'%3E%3C/svg%3E"}
                                onload={(e) => {
                                    const target = e.target;
                                    if (target instanceof HTMLImageElement && target.src !== "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1 1'%3E%3C/svg%3E") {
                                        target.classList.add("loaded");
                                    }
                                }}
                            />
                            <div class="absolute top-2 right-2 flex flex-col gap-1">
                                {#if image.hasJson}<div class="bg-blue-500 text-white text-xs px-2 py-1 rounded-full shadow">JSON</div>{/if}
                                {#if image.annotated}<div class="bg-green-500 text-white text-xs px-2 py-1 rounded-full shadow">{annotationType === "bounding_box" ? "Boxes" : "Polygons"}</div>{/if}
                            </div>
                        </div>
                        <div class="p-3">
                            <p class="text-sm text-gray-800 truncate">{image.name}</p>
                            {#if image.size}<p class="text-xs text-gray-500 mt-1">{formatFileSize(image.size)}</p>{/if}
                            {#if image.annotations && image.annotations.length > 0}
                                <p class="text-xs text-green-600 mt-1">{image.annotations.length} {annotationType === "bounding_box" ? "box" : "polygon"}{image.annotations.length !== 1 ? "es" : ""}</p>
                            {/if}
                        </div>
                    </button>
                {/each}
            </div>
        {:else}
            <div class="space-y-4">
                {#each getVisibleImages() as image, index (image.path)}
                    <button
                        type="button"
                        class="bg-white rounded-lg shadow-md overflow-hidden hover:shadow-lg transition-shadow duration-200 cursor-pointer relative w-full text-left p-0 border-none block"
                        onclick={() => onselectimage?.({ image, index })}
                        aria-label={`View details for ${image.name}`}
                    >
                        <div class="flex flex-col sm:flex-row">
                            <div class="sm:w-48 h-48 relative">
                                <img
                                    class={`w-full h-full object-cover transition-opacity duration-300 ${image.displayIndex < pageSize ? "opacity-100" : "lazy-image opacity-0"}`}
                                    data-src={image.previewUrl}
                                    data-index={index}
                                    alt={image.name}
                                    src={image.displayIndex < pageSize ? image.previewUrl : "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1 1'%3E%3C/svg%3E"}
                                    onload={(e) => {
                                        const target = e.target;
                                        if (target instanceof HTMLImageElement && target.src !== "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1 1'%3E%3C/svg%3E") {
                                            target.classList.add("loaded");
                                        }
                                    }}
                                />
                                <div class="absolute top-2 right-2 flex flex-col gap-1">
                                    {#if image.hasJson}<div class="bg-blue-500 text-white text-xs px-2 py-1 rounded-full shadow">JSON</div>{/if}
                                    {#if image.annotated}<div class="bg-green-500 text-white text-xs px-2 py-1 rounded-full shadow">{annotationType === "bounding_box" ? "Boxes" : "Polygons"}</div>{/if}
                                </div>
                            </div>
                            <div class="p-4 flex-1">
                                <h3 class="text-lg font-medium text-gray-800">{image.name}</h3>
                                {#if image.size}<p class="text-sm text-gray-500 mt-1">Size: {formatFileSize(image.size)}</p>{/if}
                                {#if image.dimensions}<p class="text-sm text-gray-500 mt-1">Dimensions: {image.dimensions.width} × {image.dimensions.height}</p>{/if}
                                {#if image.created}<p class="text-sm text-gray-500 mt-1">Created: {new Date(image.created).toLocaleString()}</p>{/if}
                                {#if image.annotations && image.annotations.length > 0}
                                    <div class="mt-2">
                                        <p class="text-sm font-medium text-green-600">{image.annotations.length} {annotationType === "bounding_box" ? "bounding box" : "polygon"}{image.annotations.length !== 1 ? "es" : ""}:</p>
                                        <ul class="mt-1 list-disc list-inside text-sm text-gray-600">
                                            {#each image.annotations.slice(0, 3) as annotation}
                                                <li class="truncate">{annotation.label}</li>
                                            {/each}
                                            {#if image.annotations.length > 3}<li class="text-gray-500">+{image.annotations.length - 3} more</li>{/if}
                                        </ul>
                                    </div>
                                {/if}
                            </div>
                        </div>
                    </button>
                {/each}
            </div>
        {/if}

        {#if !loading && images.length > 0 && totalPages > 1}
            <div class="flex items-center justify-center my-8">
                <nav class="relative z-0 inline-flex rounded-md shadow-sm -space-x-px" aria-label="Pagination">
                    <button
                        class="relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50"
                        onclick={() => { if (currentPage > 1) onloadpage?.(currentPage - 1); }}
                        disabled={currentPage === 1 || loading || loadingMore}
                    >
                        <span class="sr-only">Previous</span>
                        <svg class="h-5 w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true"><path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" /></svg>
                    </button>
                    {#each generatePageNumbers(currentPage, totalPages) as pageNum}
                        {#if pageNum === "..."}
                            <span class="relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium">...</span>
                        {:else}
                            <button
                                class="relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium {currentPage === pageNum ? 'z-10 bg-indigo-50 border-indigo-500 text-indigo-600' : 'text-gray-500 hover:bg-gray-50'}"
                                onclick={() => { if (currentPage !== pageNum && typeof pageNum === 'number') onloadpage?.(pageNum); }}
                                disabled={loading || loadingMore}
                            >
                                {pageNum}
                            </button>
                        {/if}
                    {/each}
                    <button
                        class="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50"
                        onclick={() => { if (currentPage < totalPages) onloadpage?.(currentPage + 1); }}
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
    {/if}
</div>

<style>
    .lazy-image { opacity: 0; transition: opacity 0.3s ease-in-out; }
    .lazy-image.loaded { opacity: 1; }
    .lazy-image:not(.loaded) {
        background: linear-gradient(90deg, rgba(229,232,235,0.6) 25%, rgba(215,219,223,0.6) 37%, rgba(229,232,235,0.6) 63%);
        background-size: 400% 100%;
        animation: shimmer 1.4s ease infinite;
    }
    img.lazy-image { width: 100%; height: 100%; object-fit: cover; position: absolute; top: 0; left: 0; }
    @keyframes shimmer { 0% { background-position: 100% 50%; } 100% { background-position: 0 50%; } }
</style>

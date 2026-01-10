<script lang="ts">
    import { createEventDispatcher, onMount, afterUpdate } from "svelte";

    export let images: any[] = [];
    export let totalImages: number = 0;
    export let currentPage: number = 1;
    export let totalPages: number = 0;
    export let pageSize: number = 30;
    export let viewMode: "grid" | "column" = "grid";
    export let annotationType: string = "bounding_box";
    export let loading: boolean = false;
    export let loadingMore: boolean = false;

    const dispatch = createEventDispatcher();

    function formatFileSize(bytes) {
        if (bytes === null || bytes === undefined) return "";
        if (bytes < 1024) return bytes + " B";
        if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
        return (bytes / 1048576).toFixed(1) + " MB";
    }

    function generatePageNumbers(current, total) {
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
        dispatch("selectImage", { image, index });
    }

    function handleLoadPage(page: number | string) {
        if (
            typeof page === "number" &&
            page >= 1 &&
            page <= totalPages &&
            page !== currentPage
        ) {
            dispatch("loadPage", page);
        }
    }

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

    let lastProcessedPage = -1;
    let lastImagesLength = 0;

    onMount(() => {
        setupLazyLoading();
        lastProcessedPage = currentPage;
        lastImagesLength = images.length;
    });

    afterUpdate(() => {
        if (
            currentPage !== lastProcessedPage ||
            images.length !== lastImagesLength
        ) {
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
                    <button
                        type="button"
                        class="card bg-base-100 shadow-sm hover:shadow-lg transition-all duration-200 cursor-pointer overflow-hidden hover:-translate-y-0.5"
                        on:click={() => handleImageClick(image, i)}
                        aria-label={`View details for ${image.name}`}
                    >
                        <figure class="relative pb-[75%]">
                            <img
                                class={`gallery-lazy-image absolute inset-0 w-full h-full object-cover transition-opacity duration-300 ${image.displayIndex < pageSize ? "opacity-100" : "opacity-0"}`}
                                data-src={image.previewUrl}
                                alt={image.name}
                                src={image.displayIndex < pageSize
                                    ? image.previewUrl
                                    : "data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 1 1%27%3E%3C/svg%3E"}
                                on:load={(e) => {
                                    const target = e.target;
                                    if (
                                        target instanceof HTMLImageElement &&
                                        target.src !==
                                            "data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 1 1%27%3E%3C/svg%3E"
                                    ) {
                                        target.classList.remove("opacity-0");
                                        target.classList.add(
                                            "loaded",
                                            "observed",
                                        );
                                    }
                                }}
                                on:error={(e) => {
                                    const target = e.target;
                                    if (target instanceof HTMLImageElement) {
                                        target.classList.add(
                                            "observed",
                                            "loaded",
                                        );
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
                                            : "Poly"}
                                    </span>
                                {/if}
                            </div>
                        </figure>
                        <div class="card-body p-3">
                            <p class="text-sm font-medium truncate">
                                {image.name}
                            </p>
                            {#if image.size != null}
                                <p class="text-xs opacity-60">
                                    {formatFileSize(image.size)}
                                </p>
                            {/if}
                            {#if image.annotations && image.annotations.length > 0}
                                <p class="text-xs text-success">
                                    {image.annotations.length}
                                    {annotationType === "bounding_box"
                                        ? "box"
                                        : "polygon"}{image.annotations
                                        .length !== 1
                                        ? "es"
                                        : ""}
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
                    <button
                        type="button"
                        class="card card-side bg-base-100 shadow-sm hover:shadow-lg transition-all duration-200 cursor-pointer overflow-hidden hover:-translate-y-0.5 w-full"
                        on:click={() => handleImageClick(image, i)}
                        aria-label={`View details for ${image.name}`}
                    >
                        <figure class="w-48 h-48 relative flex-shrink-0">
                            <img
                                class={`gallery-lazy-image w-full h-full object-cover transition-opacity duration-300 ${image.displayIndex < pageSize ? "opacity-100" : "opacity-0"}`}
                                data-src={image.previewUrl}
                                alt={image.name}
                                src={image.displayIndex < pageSize
                                    ? image.previewUrl
                                    : "data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 1 1%27%3E%3C/svg%3E"}
                                on:load={(e) => {
                                    const target = e.target;
                                    if (
                                        target instanceof HTMLImageElement &&
                                        target.src !==
                                            "data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 1 1%27%3E%3C/svg%3E"
                                    ) {
                                        target.classList.remove("opacity-0");
                                        target.classList.add(
                                            "loaded",
                                            "observed",
                                        );
                                    }
                                }}
                                on:error={(e) => {
                                    const target = e.target;
                                    if (target instanceof HTMLImageElement) {
                                        target.classList.add(
                                            "observed",
                                            "loaded",
                                        );
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
                                            : "Poly"}
                                    </span>
                                {/if}
                            </div>
                        </figure>
                        <div class="card-body p-4 text-left">
                            <h3 class="card-title text-base">{image.name}</h3>
                            {#if image.size != null}
                                <p class="text-sm opacity-60">
                                    Size: {formatFileSize(image.size)}
                                </p>
                            {/if}
                            {#if image.dimensions}
                                <p class="text-sm opacity-60">
                                    Dimensions: {image.dimensions.width} × {image
                                        .dimensions.height}
                                </p>
                            {/if}
                            {#if image.created}
                                <p class="text-sm opacity-60">
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
                                        class="mt-1 list-disc list-inside text-sm opacity-70"
                                    >
                                        {#each image.annotations.slice(0, 3) as annotation, annotationIndex (annotationIndex)}
                                            <li class="truncate">
                                                {annotation.label}
                                            </li>
                                        {/each}
                                        {#if image.annotations.length > 3}
                                            <li class="opacity-50">
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
                <div class="join">
                    <button
                        class="join-item btn btn-sm"
                        on:click={() => handleLoadPage(currentPage - 1)}
                        disabled={currentPage === 1 || loading || loadingMore}
                    >
                        <span class="material-symbols-rounded icon-sm"
                            >chevron_left</span
                        >
                    </button>
                    {#each generatePageNumbers(currentPage, totalPages) as pageNum (typeof pageNum === "number" ? pageNum : pageNum)}
                        {#if typeof pageNum === "number"}
                            <button
                                class="join-item btn btn-sm {currentPage ===
                                pageNum
                                    ? 'btn-active'
                                    : ''}"
                                on:click={() => handleLoadPage(pageNum)}
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
                        class="join-item btn btn-sm"
                        on:click={() => handleLoadPage(currentPage + 1)}
                        disabled={currentPage === totalPages ||
                            loading ||
                            loadingMore}
                    >
                        <span class="material-symbols-rounded icon-sm"
                            >chevron_right</span
                        >
                    </button>
                </div>
            </div>
        {/if}

        {#if loadingMore}
            <div class="flex justify-center my-8">
                <span class="loading loading-spinner loading-md text-primary"
                ></span>
            </div>
        {/if}
    </div>
{/if}

<style>
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

    .gallery-lazy-image:not(.loaded) {
        background: linear-gradient(
            90deg,
            oklch(var(--b2)) 25%,
            oklch(var(--b3)) 37%,
            oklch(var(--b2)) 63%
        );
        background-size: 400% 100%;
        animation: shimmer 1.4s ease infinite;
    }

    .gallery-lazy-image.loaded {
        animation: none;
        background: none;
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

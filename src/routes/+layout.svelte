<script>
	import { onMount } from "svelte";
	import Sidebar from "./Sidebar.svelte";
	import "../app.css";

	let { children } = $props();

	let theme = $state("light");
	let isSidebarExpanded = $state(true);
	let sidebarWidth = $state(256);
	let isResizing = $state(false);

	const MIN_SIDEBAR_WIDTH = 200;
	const MAX_SIDEBAR_WIDTH = 480;

	function handleResizeStart(e) {
		e.preventDefault();
		isResizing = true;

		function onMouseMove(e) {
			const newWidth = Math.min(MAX_SIDEBAR_WIDTH, Math.max(MIN_SIDEBAR_WIDTH, e.clientX));
			sidebarWidth = newWidth;
		}

		function onMouseUp() {
			isResizing = false;
			window.removeEventListener("mousemove", onMouseMove);
			window.removeEventListener("mouseup", onMouseUp);
		}

		window.addEventListener("mousemove", onMouseMove);
		window.addEventListener("mouseup", onMouseUp);
	}

	onMount(() => {
		// Check for saved theme or system preference
		const savedTheme = localStorage.getItem("theme");
		if (savedTheme) {
			theme = savedTheme;
		} else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
			theme = "dark";
		}
		document.documentElement.setAttribute("data-theme", theme);

		// Restore sidebar state if we wanted to persist it, but user asked for default expanded.
		// let savedSidebar = localStorage.getItem("sidebarExpanded");
		// if(savedSidebar !== null) isSidebarExpanded = savedSidebar === "true";
	});

	function toggleTheme() {
		theme = theme === "light" ? "dark" : "light";
		document.documentElement.setAttribute("data-theme", theme);
		localStorage.setItem("theme", theme);
	}

	function toggleSidebar() {
		isSidebarExpanded = !isSidebarExpanded;
		// localStorage.setItem("sidebarExpanded", String(isSidebarExpanded));
	}
</script>

<!-- 
    DaisyUI Drawer Structure:
    - .drawer: Wrapper
    - .drawer-toggle: Hidden checkbox
    - .drawer-content: Main content area
    - .drawer-side: Sidebar area (overlay on mobile, fixed on desktop if drawer-open)
    
    Logic:
    - lg:drawer-open: Forces sidebar to be visible and pushes content.
    - We toggle this class based on isSidebarExpanded.
-->
<div
	class="app drawer"
	class:lg:drawer-open={isSidebarExpanded}
	class:resizing={isResizing}
	style:--sidebar-width="{sidebarWidth}px"
>
	<input id="sidebar-drawer" type="checkbox" class="drawer-toggle" />

	<div class="drawer-content flex flex-col transition-all duration-300">
		<!-- Main content -->
		<main class="flex-1 w-full bg-base-100 relative">
			<!-- Sidebar Toggle Button (Floating or Sticky) -->
			{#if !isSidebarExpanded}
				<div
					class="absolute top-4 left-4 z-30 print:hidden transition-all duration-300"
				>
					<button
						onclick={toggleSidebar}
						class="btn btn-circle btn-ghost btn-sm shadow-sm bg-base-100 hover:bg-base-200 border border-base-200"
						title="Open Sidebar"
					>
						<span
							class="material-symbols-rounded text-base-content/70"
						>
							menu
						</span>
					</button>
				</div>
			{/if}

			<div
				class="w-full px-6 py-8 pt-16 lg:pt-8 transition-all duration-300"
			>
				{@render children()}
			</div>
		</main>
	</div>

	<div class="drawer-side z-40">
		<label
			for="sidebar-drawer"
			aria-label="close sidebar"
			class="drawer-overlay"
		></label>
		<Sidebar {theme} {toggleTheme} {isSidebarExpanded} {toggleSidebar} width={sidebarWidth} />
	</div>

	<!-- Resize Handle (outside drawer-side to avoid overflow:hidden clipping) -->
	{#if isSidebarExpanded}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="resize-handle hidden lg:block"
			onmousedown={handleResizeStart}
			style="left: {sidebarWidth}px"
			class:active={isResizing}
		></div>
	{/if}
</div>

<style>
	.app {
		min-height: 100vh;
	}

	.app.resizing {
		user-select: none;
		cursor: col-resize;
	}

	/* Override DaisyUI drawer width — only at lg+ where drawer-open is active */
	@media (min-width: 1024px) {
		:global(.app.lg\:drawer-open > .drawer-side) {
			width: var(--sidebar-width, 256px);
		}
	}

	.resize-handle {
		position: fixed;
		top: 0;
		bottom: 0;
		width: 6px;
		transform: translateX(-3px);
		cursor: col-resize;
		z-index: 50;
		transition: background-color 0.15s;
	}

	.resize-handle:hover,
	.resize-handle.active {
		background-color: oklch(var(--p) / 0.3);
	}
</style>

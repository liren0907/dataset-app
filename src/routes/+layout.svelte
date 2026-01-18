<script>
	import { onMount } from "svelte";
	import Sidebar from "./Sidebar.svelte";
	import "../app.css";

	let theme = "light";
	let isSidebarExpanded = true;

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

	// Export for child components if needed (rarely used this way in SvelteKit layouts but keeping pattern)
	export { theme, toggleTheme };
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
<div class="app drawer" class:lg:drawer-open={isSidebarExpanded}>
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
						on:click={toggleSidebar}
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
				<slot />
			</div>
		</main>
	</div>

	<div class="drawer-side z-40">
		<label
			for="sidebar-drawer"
			aria-label="close sidebar"
			class="drawer-overlay"
		></label>
		<Sidebar {theme} {toggleTheme} {isSidebarExpanded} {toggleSidebar} />
	</div>
</div>

<style>
	.app {
		min-height: 100vh;
	}
</style>

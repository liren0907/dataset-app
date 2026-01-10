<script>
	import { onMount } from "svelte";
	import Sidebar from "./Sidebar.svelte";
	import "../app.css";

	let theme = "light";

	onMount(() => {
		// Check for saved theme or system preference
		const savedTheme = localStorage.getItem("theme");
		if (savedTheme) {
			theme = savedTheme;
		} else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
			theme = "dark";
		}
		document.documentElement.setAttribute("data-theme", theme);
	});

	function toggleTheme() {
		theme = theme === "light" ? "dark" : "light";
		document.documentElement.setAttribute("data-theme", theme);
		localStorage.setItem("theme", theme);
	}

	// Export for child components
	export { theme, toggleTheme };
</script>

<div class="app drawer lg:drawer-open">
	<input id="sidebar-drawer" type="checkbox" class="drawer-toggle" />

	<div class="drawer-content flex flex-col">
		<!-- Main content -->
		<main class="flex-1 w-full bg-base-100">
			<div class="w-full px-6 py-8">
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
		<Sidebar {theme} {toggleTheme} />
	</div>
</div>

<style>
	.app {
		min-height: 100vh;
	}
</style>

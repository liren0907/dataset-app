<script>
	import Sidebar from "./Sidebar.svelte";
	import { sidebarOpen } from '../store.js';
	import "../app.css";
</script>

<div class="app min-h-screen bg-gradient-to-br from-slate-50 via-white to-indigo-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900">
	<Sidebar />

	<main class="flex-1 w-full transition-all duration-300 {$sidebarOpen ? 'ml-64' : 'ml-16'}">
		<div class="max-w-7xl mx-auto px-6 py-8">
			<slot />
		</div>
	</main>
</div>

<style>
	.app {
		display: flex;
		min-height: 100vh;
	}

	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		box-sizing: border-box;
	}
</style>

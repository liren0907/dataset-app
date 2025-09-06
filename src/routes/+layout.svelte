<script>
	import Header from "./Header.svelte";
	import "../app.css";
</script>

<div class="app min-h-screen bg-gradient-to-br from-slate-50 via-white to-indigo-50">
	<Header />

	<main class="flex-1 w-full max-w-7xl mx-auto px-6 py-8">
		<slot />
	</main>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}

	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		box-sizing: border-box;
	}

	footer {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		padding: 12px;
	}

	footer a {
		font-weight: bold;
	}

	@media (min-width: 480px) {
		footer {
			padding: 12px 0;
		}
	}
</style>

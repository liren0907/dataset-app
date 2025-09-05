<script>
	import { page } from '$app/stores';
</script>

<header>
	<nav class="bg-white shadow-lg h-16">
		<div class="max-w-7xl mx-auto px-4 h-full">
			<div class="flex justify-between items-center h-full">
				<div class="flex-shrink-0 flex items-center">
					<span class="text-xl font-bold text-gray-800">Dataset Viewer</span>
				</div>
				<div class="hidden sm:flex sm:items-center sm:space-x-4">
					<!-- Home Link -->
					<a
						href="/"
						class="
							{$page.url.pathname === '/' ? 'bg-indigo-50 text-indigo-600' : 'text-gray-500'}
							px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150 ease-in-out
							hover:bg-gray-50 hover:text-gray-900 no-underline
						"
					>
						Home
					</a>
					<!-- Smart Tools Link -->
					<a
						href="/smart-tools"
						class="
							{$page.url.pathname.startsWith('/smart-tools') ? 'bg-indigo-50 text-indigo-600' : 'text-gray-500'}
							px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150 ease-in-out
							hover:bg-gray-50 hover:text-gray-900 no-underline
						"
					>
						Smart Tools
					</a>
					<!-- Dataset Gallery Link -->
					<a
						href="/dataset-gallery"
						class="
							{$page.url.pathname.startsWith('/dataset-gallery') ? 'bg-indigo-50 text-indigo-600' : 'text-gray-500'}
							px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150 ease-in-out
							hover:bg-gray-50 hover:text-gray-900 no-underline
						"
					>
						Dataset Gallery
					</a>
					<!-- Crop & Remap Tool Link -->
					<a
						href="/crop-remap"
						class="
							{$page.url.pathname.startsWith('/crop-remap') ? 'bg-indigo-50 text-indigo-600' : 'text-gray-500'}
							px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150 ease-in-out
							hover:bg-gray-50 hover:text-gray-900 no-underline
						"
					>
						Crop & Remap
					</a>
					<!-- Image Viewer 3 Link -->
					<a
						href="/imageViewer3"
						class="
							{$page.url.pathname.startsWith('/imageViewer3') ? 'bg-indigo-50 text-indigo-600' : 'text-gray-500'}
							px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150 ease-in-out
							hover:bg-gray-50 hover:text-gray-900 no-underline
						"
					>
						Image Viewer 3
					</a>
				</div>
			</div>
		</div>
	</nav>
</header>

<style>
	:global(a) {
		text-decoration: none !important;
	}
</style> 
<script>
	import { page } from '$app/stores';
</script>

<header class="sticky top-0 z-40">
	<nav class="backdrop-blur supports-[backdrop-filter]:bg-white/60 bg-white/80 border-b border-slate-200/60">
		<div class="max-w-7xl mx-auto px-4">
			<div class="h-16 flex justify-between items-center">
				<div class="flex items-center gap-3">
					<div class="h-8 w-8 rounded-lg bg-indigo-600 shadow-sm shadow-indigo-300"></div>
					<span class="text-lg font-semibold tracking-tight text-slate-800">Dataset Viewer</span>
				</div>
				<div class="hidden sm:flex sm:items-center sm:space-x-1">
					<a
						href="/"
						class="px-3 py-2 rounded-md text-sm font-medium transition-all duration-200 hover:text-slate-900 hover:bg-slate-100 no-underline {$page.url.pathname === '/' ? 'bg-indigo-600 text-white shadow-sm shadow-indigo-300' : 'text-slate-600'}"
					>
						Home
					</a>
					<a
						href="/smart-tools"
						class="px-3 py-2 rounded-md text-sm font-medium transition-all duration-200 hover:text-slate-900 hover:bg-slate-100 no-underline {$page.url.pathname.startsWith('/smart-tools') ? 'bg-indigo-600 text-white shadow-sm shadow-indigo-300' : 'text-slate-600'}"
					>
						Smart Tools
					</a>
					<a
						href="/dataset-gallery"
						class="px-3 py-2 rounded-md text-sm font-medium transition-all duration-200 hover:text-slate-900 hover:bg-slate-100 no-underline {$page.url.pathname.startsWith('/dataset-gallery') ? 'bg-indigo-600 text-white shadow-sm shadow-indigo-300' : 'text-slate-600'}"
					>
						Dataset Gallery
					</a>
					<a
						href="/crop-remap"
						class="px-3 py-2 rounded-md text-sm font-medium transition-all duration-200 hover:text-slate-900 hover:bg-slate-100 no-underline {$page.url.pathname.startsWith('/crop-remap') ? 'bg-indigo-600 text-white shadow-sm shadow-indigo-300' : 'text-slate-600'}"
					>
						Crop & Remap
					</a>
					<a
						href="/imageViewer3"
						class="px-3 py-2 rounded-md text-sm font-medium transition-all duration-200 hover:text-slate-900 hover:bg-slate-100 no-underline {$page.url.pathname.startsWith('/imageViewer3') ? 'bg-indigo-600 text-white shadow-sm shadow-indigo-300' : 'text-slate-600'}"
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
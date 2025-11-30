<script lang="ts">
	// @ts-ignore - SvelteKit 內建模組，運行時正常
	import { page } from '$app/stores';
	import { darkMode, toggleDarkMode, sidebarOpen, toggleSidebar } from '../store.js';

	// 工具列表
	const tools = [
		{ href: '/', label: 'Home', icon: '🏠', description: '首頁' },
		{ href: '/turbo-export', label: 'Turbo Export', icon: '⚡', description: '高效能格式轉換' },
		{ href: '/smart-tools', label: 'Smart Tools', icon: '🛠️', description: '智能工具集' },
		{ href: '/dataset-gallery', label: 'Dataset Gallery', icon: '🖼️', description: '資料集瀏覽' },
		{ href: '/crop-remap', label: 'Crop & Remap', icon: '✂️', description: '裁切與重映射' },
		{ href: '/crop-remap-component', label: 'Crop Component', icon: '📐', description: '裁切組件' },
		{ href: '/dataset-gallery-advanced', label: 'Advanced Gallery', icon: '🎨', description: '進階瀏覽器' },
		{ href: '/imageViewer3', label: 'Image Viewer', icon: '👁️', description: '圖片檢視器' },
	];

	// 點擊導航連結後關閉側邊欄
	function handleNavClick() {
		if ($sidebarOpen) {
			toggleSidebar();
		}
	}
</script>

<!-- 遮罩層 - 點擊關閉側邊欄（半透明背景） -->
{#if $sidebarOpen}
	<div
		class="fixed inset-0 bg-black/30 dark:bg-black/50 z-40 backdrop-blur-sm transition-opacity duration-120"
		on:click={toggleSidebar}
		on:keydown={(e) => e.key === 'Escape' && toggleSidebar()}
		role="button"
		tabindex="0"
		aria-label="Close sidebar"
	></div>
{/if}

<!-- 漢堡選單按鈕 - 側邊欄收起時顯示 -->
{#if !$sidebarOpen}
	<button
		on:click={toggleSidebar}
		class="fixed top-0 left-0 z-50 p-4 transition-colors duration-120
			text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100"
		aria-label="Open sidebar"
	>
		<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
			<path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h16" />
		</svg>
	</button>
{/if}

<!-- 側邊欄 -->
<aside
	class="fixed top-0 left-0 z-50 h-full transition-transform duration-120 ease-out
		{$sidebarOpen ? 'w-56 translate-x-0' : 'w-64 -translate-x-full'}
		bg-white/60 dark:bg-slate-900/60 backdrop-blur-lg
		border-r border-slate-200/60 dark:border-slate-700/60
		shadow-lg dark:shadow-slate-900/50"
>
	<div class="flex flex-col h-full">
	<!-- 頂部區域：展開/收起按鈕 -->
	<div class="h-16 flex items-center px-4 border-b border-slate-200/60 dark:border-slate-700/60">
		<button
			on:click={toggleSidebar}
			class="p-4 -ml-4 transition-colors duration-120
				text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100"
			aria-label="Close sidebar"
		>
			<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
				<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
			</svg>
		</button>
		{#if $sidebarOpen}
			<span class="ml-2 font-semibold text-slate-800 dark:text-slate-100 whitespace-nowrap overflow-hidden">
				Dataset Viewer
			</span>
		{/if}
	</div>		<!-- 導航區域 -->
		<nav class="flex-1 overflow-y-auto py-4 px-2">
			<div class="space-y-1">
				{#each tools as tool (tool.href)}
					{@const isActive = tool.href === '/'
						? $page.url.pathname === '/'
						: $page.url.pathname === tool.href || $page.url.pathname.startsWith(tool.href + '/')}
					<a
						href={tool.href}
						on:click={handleNavClick}
						class="flex items-center gap-3 px-3 py-2.5 rounded-lg transition-transform duration-120 no-underline group
							{isActive
								? 'bg-indigo-600 text-white shadow-md shadow-indigo-300/50 dark:shadow-indigo-900/50'
								: 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 hover:text-slate-900 dark:hover:text-slate-100'}"
						title={$sidebarOpen ? '' : tool.label}
					>
						<span class="text-lg flex-shrink-0 {isActive ? '' : 'group-hover:scale-110'} transition-transform">
							{tool.icon}
						</span>
						{#if $sidebarOpen}
							<div class="overflow-hidden whitespace-nowrap">
								<div class="font-medium text-sm">{tool.label}</div>
								<div class="text-xs {isActive ? 'text-indigo-200' : 'text-slate-400 dark:text-slate-500'}">
									{tool.description}
								</div>
							</div>
						{/if}
					</a>
				{/each}
			</div>
		</nav>

		<!-- 底部設定區域 -->
		<div class="border-t border-slate-200/60 dark:border-slate-700/60 p-2">
			<!-- 深色模式切換 -->
			<button
				on:click={toggleDarkMode}
				class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-transform duration-120
					text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 hover:text-slate-900 dark:hover:text-slate-100"
				title={$sidebarOpen ? '' : ($darkMode ? 'Light Mode' : 'Dark Mode')}
			>
				<span class="text-lg flex-shrink-0">
					{#if $darkMode}
						☀️
					{:else}
						🌙
					{/if}
				</span>
				{#if $sidebarOpen}
					<div class="overflow-hidden whitespace-nowrap">
						<div class="font-medium text-sm">{$darkMode ? '淺色模式' : '深色模式'}</div>
						<div class="text-xs text-slate-400 dark:text-slate-500">
							{$darkMode ? '切換到淺色主題' : '拯救你的眼睛'}
						</div>
					</div>
				{/if}
			</button>

			<!-- 設定按鈕（預留） -->
			<button
				class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-transform duration-120
					text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 hover:text-slate-900 dark:hover:text-slate-100"
				title={$sidebarOpen ? '' : 'Settings'}
			>
				<span class="text-lg flex-shrink-0">⚙️</span>
				{#if $sidebarOpen}
					<div class="overflow-hidden whitespace-nowrap">
						<div class="font-medium text-sm">設定</div>
						<div class="text-xs text-slate-400 dark:text-slate-500">更多選項</div>
					</div>
				{/if}
			</button>
		</div>
	</div>
</aside>

<style>
	aside {
		scrollbar-width: thin;
		scrollbar-color: rgba(100, 116, 139, 0.3) transparent;
	}

	aside::-webkit-scrollbar {
		width: 4px;
	}

	aside::-webkit-scrollbar-track {
		background: transparent;
	}

	aside::-webkit-scrollbar-thumb {
		background-color: rgba(100, 116, 139, 0.3);
		border-radius: 2px;
	}
</style>

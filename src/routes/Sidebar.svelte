<script lang="ts">
	// @ts-ignore - SvelteKit 內建模組，運行時正常
	import { page } from "$app/stores";

	// Props from parent layout
	export let theme = "light";
	export let toggleTheme = () => {};
    export let isSidebarExpanded = true;
    export let toggleSidebar = () => {};

	// 工具列表 - using Material Symbols icon names
	const tools = [
		{ href: "/", label: "Home", icon: "home", description: "首頁" },
		{
			href: "/turbo-export",
			label: "Turbo Export",
			icon: "bolt",
			description: "高效能格式轉換",
		},
		{
			href: "/smart-tools",
			label: "Smart Tools",
			icon: "construction",
			description: "智能工具集",
		},
		{
			href: "/dataset-gallery",
			label: "Dataset Gallery",
			icon: "photo_library",
			description: "資料集瀏覽",
		},
		{
			href: "/crop-remap",
			label: "Crop & Remap",
			icon: "crop",
			description: "裁切與重映射",
		},
		{
			href: "/crop-remap-component",
			label: "Crop Component",
			icon: "straighten",
			description: "裁切組件",
		},
		{
			href: "/dataset-gallery-advanced",
			label: "Advanced Gallery",
			icon: "palette",
			description: "進階瀏覽器",
		},
		{
			href: "/imageViewer3",
			label: "Image Viewer",
			icon: "visibility",
			description: "圖片檢視器",
		},
	];
</script>

<!-- DaisyUI Menu Sidebar -->
<ul class="menu bg-base-200 min-h-full w-64 p-4 gap-1 border-r border-base-300">
	<!-- Logo / Title -->
	<li class="menu-title flex flex-row items-center justify-between gap-2 mb-4">
        <div class="flex items-center gap-2">
            <span class="material-symbols-rounded text-primary">dataset</span>
            <span class="text-lg font-bold">Dataset App</span>
        </div>
        <!-- Collapse Button inside Sidebar (Optional, maybe redundant if we have the floating one, but user asked for "retract" button) -->
        <button on:click={toggleSidebar} class="btn btn-ghost btn-xs btn-circle opacity-50 hover:opacity-100">
            <span class="material-symbols-rounded log-icon">first_page</span>
        </button>
	</li>

	<!-- Navigation Items -->
	{#each tools as tool (tool.href)}
		{@const isActive =
			tool.href === "/"
				? $page.url.pathname === "/"
				: $page.url.pathname === tool.href ||
					$page.url.pathname.startsWith(tool.href + "/")}
		<li>
			<a
				href={tool.href}
				class="flex items-center gap-3 {isActive ? 'active' : ''}"
			>
				<span class="material-symbols-rounded">{tool.icon}</span>
				<div class="flex flex-col">
					<span class="font-medium">{tool.label}</span>
					<span class="text-xs opacity-60">{tool.description}</span>
				</div>
			</a>
		</li>
	{/each}

	<!-- Spacer -->
	<div class="flex-1"></div>

	<!-- Divider -->
	<li class="divider my-2"></li>

	<!-- Theme Toggle -->
	<li>
		<button on:click={toggleTheme} class="flex items-center gap-3">
			<span class="material-symbols-rounded">
				{theme === "dark" ? "light_mode" : "dark_mode"}
			</span>
			<div class="flex flex-col">
				<span class="font-medium"
					>{theme === "dark" ? "淺色模式" : "深色模式"}</span
				>
				<span class="text-xs opacity-60">
					{theme === "dark" ? "切換到淺色主題" : "拯救你的眼睛"}
				</span>
			</div>
		</button>
	</li>

	<!-- Settings -->
	<li>
		<a href="#settings" class="flex items-center gap-3">
			<span class="material-symbols-rounded">settings</span>
			<div class="flex flex-col">
				<span class="font-medium">設定</span>
				<span class="text-xs opacity-60">更多選項</span>
			</div>
		</a>
	</li>
</ul>

<style>
	.menu :global(.active) {
		@apply bg-primary text-primary-content;
	}

	.divider {
		height: 1px;
		@apply bg-base-300;
	}
</style>

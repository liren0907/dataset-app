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
		{ href: "/", label: "Home", icon: "home" },
		{
			href: "/turbo-export",
			label: "Turbo Export",
			icon: "bolt",
		},
		{
			href: "/smart-tools",
			label: "Smart Tools",
			icon: "construction",
		},
		{
			href: "/gallery",
			label: "Unified Gallery",
			icon: "collections_bookmark",
		},
		{
			href: "/legacy-gallery",
			label: "Legacy Gallery",
			icon: "history",
		},
		{
			href: "/crop-remap",
			label: "Crop & Remap",
			icon: "crop",
		},
		{
			href: "/crop-remap-component",
			label: "Crop Component",
			icon: "straighten",
		},
	];
</script>

<!-- DaisyUI Menu Sidebar -->
<ul class="menu bg-base-200 min-h-full w-64 p-4 gap-1 border-r border-base-300">
	<!-- Logo / Title -->
	<li
		class="menu-title flex flex-row items-center justify-between gap-2 mb-4"
	>
		<div class="flex items-center gap-2">
			<span class="material-symbols-rounded text-primary">dataset</span>
			<span class="text-lg font-bold">Dataset App</span>
		</div>
		<!-- Collapse Button inside Sidebar (Optional, maybe redundant if we have the floating one, but user asked for "retract" button) -->
		<button
			on:click={toggleSidebar}
			class="btn btn-ghost btn-xs btn-circle opacity-50 hover:opacity-100"
		>
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
				<span class="font-medium">{tool.label}</span>
			</a>
		</li>
	{/each}

	<!-- Spacer -->
	<div class="flex-1"></div>

	<!-- Settings (Pinned to Bottom) -->
	<li>
		<a
			href="/settings"
			class="flex items-center gap-3 {$page.url.pathname === '/settings'
				? 'active'
				: ''}"
		>
			<span class="material-symbols-rounded">settings</span>
			<span class="font-medium">Settings</span>
		</a>
	</li>

	<!-- Divider -->
	<li class="divider my-2"></li>

	<!-- Theme Toggle -->
	<li>
		<button on:click={toggleTheme} class="flex items-center gap-3">
			<span class="material-symbols-rounded">
				{theme === "dark" ? "light_mode" : "dark_mode"}
			</span>
			<span class="font-medium">
				{theme === "dark" ? "Light Mode" : "Dark Mode"}
			</span>
		</button>
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

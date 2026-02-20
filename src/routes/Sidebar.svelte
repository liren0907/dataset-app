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
	];
</script>

<!-- Modern Minimalist Sidebar (Gallery Style) -->
<ul
	class="menu bg-base-100 min-h-full w-64 p-4 gap-1 border-r border-base-200 text-base-content"
>
	<!-- Logo / Title -->
	<li
		class="menu-title flex flex-row items-center justify-between gap-2 mb-6 px-2"
	>
		<div class="flex items-center gap-2.5">
			<div
				class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center text-primary border border-primary/20"
			>
				<span class="material-symbols-rounded">dataset</span>
			</div>
			<span class="text-base font-bold text-base-content"
				>Dataset App</span
			>
		</div>
		<!-- Collapse Button inside Sidebar -->
		<button
			on:click={toggleSidebar}
			class="btn btn-ghost btn-xs btn-square text-base-content/40 hover:text-base-content"
		>
			<span class="material-symbols-rounded">first_page</span>
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
				class="flex items-center gap-3 px-3 py-2 rounded-lg transition-colors duration-200
                {isActive
					? 'bg-primary/10 text-primary font-medium'
					: 'text-base-content/70 hover:bg-base-200 hover:text-base-content'}"
			>
				<span
					class="material-symbols-rounded {isActive
						? 'filled'
						: ''} text-[20px]">{tool.icon}</span
				>
				<span class="text-sm">{tool.label}</span>
			</a>
		</li>
	{/each}

	<!-- Spacer -->
	<div class="flex-1"></div>

	<!-- Settings (Pinned to Bottom) -->
	<li>
		<a
			href="/settings"
			class="flex items-center gap-3 px-3 py-2 rounded-lg transition-colors duration-200
            {$page.url.pathname === '/settings'
				? 'bg-neutral/10 text-neutral font-medium'
				: 'text-base-content/70 hover:bg-base-200 hover:text-base-content'}"
		>
			<span class="material-symbols-rounded text-[20px]">settings</span>
			<span class="text-sm">Settings</span>
		</a>
	</li>

	<!-- Divider -->
	<li class="divider my-2"></li>

	<!-- Theme Toggle -->
	<li>
		<button
			on:click={toggleTheme}
			class="flex items-center gap-3 px-3 py-2 rounded-lg text-base-content/70 hover:bg-base-200 hover:text-base-content transition-colors"
		>
			<span class="material-symbols-rounded text-[20px]">
				{theme === "dark" ? "light_mode" : "dark_mode"}
			</span>
			<span class="text-sm">
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

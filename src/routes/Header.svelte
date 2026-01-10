<script lang="ts">
	// @ts-ignore - SvelteKit 內建模組，運行時正常
	import { page } from "$app/stores";

	const navItems = [
		{ href: "/", label: "Home", icon: "home" },
		{
			href: "/smart-tools",
			label: "Smart Tools",
			icon: "construction",
			prefix: true,
		},
		{
			href: "/dataset-gallery",
			label: "Dataset Gallery",
			icon: "photo_library",
		},
		{ href: "/crop-remap", label: "Crop & Remap", icon: "crop" },
		{
			href: "/crop-remap-component",
			label: "Crop Component",
			icon: "crop_free",
			prefix: true,
		},
		{
			href: "/dataset-gallery-advanced",
			label: "Gallery Advanced",
			icon: "auto_awesome",
		},
		{
			href: "/imageViewer3",
			label: "Image Viewer 3",
			icon: "visibility",
			prefix: true,
		},
	];

	function isActive(
		href: string,
		pathname: string,
		prefix?: boolean,
	): boolean {
		return prefix ? pathname.startsWith(href) : pathname === href;
	}
</script>

<header class="sticky top-0 z-40">
	<nav class="navbar bg-base-100/80 backdrop-blur border-b border-base-200">
		<div class="navbar-start">
			<span
				class="text-lg font-semibold tracking-tight flex items-center gap-2"
			>
				<span class="material-symbols-rounded text-primary"
					>database</span
				>
				Dataset App
			</span>
		</div>
		<div class="navbar-center hidden lg:flex">
			<ul class="menu menu-horizontal px-1">
				{#each navItems as item}
					{@const active = isActive(
						item.href,
						$page.url.pathname,
						item.prefix,
					)}
					<li>
						<a href={item.href} class={active ? "active" : ""}>
							<span class="material-symbols-rounded icon-sm"
								>{item.icon}</span
							>
							{item.label}
						</a>
					</li>
				{/each}
			</ul>
		</div>
		<div class="navbar-end lg:hidden">
			<div class="dropdown dropdown-end">
				<div
					tabindex="0"
					role="button"
					class="btn btn-ghost btn-circle"
				>
					<span class="material-symbols-rounded">menu</span>
				</div>
				<ul
					tabindex="0"
					class="menu menu-sm dropdown-content mt-3 z-50 p-2 shadow bg-base-100 rounded-box w-52"
				>
					{#each navItems as item}
						{@const active = isActive(
							item.href,
							$page.url.pathname,
							item.prefix,
						)}
						<li>
							<a href={item.href} class={active ? "active" : ""}>
								<span class="material-symbols-rounded icon-sm"
									>{item.icon}</span
								>
								{item.label}
							</a>
						</li>
					{/each}
				</ul>
			</div>
		</div>
	</nav>
</header>

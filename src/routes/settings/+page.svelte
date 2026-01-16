<script lang="ts">
    import { onMount } from "svelte";

    // Theme state
    type ThemeMode = "light" | "dark" | "auto";
    let themeMode: ThemeMode = "auto";
    let currentAppliedTheme: "light" | "dark" = "light";

    // Language state (placeholder only)
    let selectedLanguage = "zh-TW";
    const languages = [
        { code: "zh-TW", label: "繁體中文", flag: "🇹🇼" },
        { code: "en", label: "English", flag: "🇺🇸" },
        { code: "ja", label: "日本語", flag: "🇯🇵" },
    ];

    onMount(() => {
        // Load saved theme mode
        const savedMode = localStorage.getItem("themeMode") as ThemeMode | null;
        if (savedMode && ["light", "dark", "auto"].includes(savedMode)) {
            themeMode = savedMode;
        }
        applyTheme();

        // Listen for system theme changes when in auto mode
        const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
        mediaQuery.addEventListener("change", handleSystemThemeChange);

        return () => {
            mediaQuery.removeEventListener("change", handleSystemThemeChange);
        };
    });

    function handleSystemThemeChange() {
        if (themeMode === "auto") {
            applyTheme();
        }
    }

    function applyTheme() {
        if (typeof window === "undefined") return;

        let theme: "light" | "dark";

        if (themeMode === "auto") {
            theme = window.matchMedia("(prefers-color-scheme: dark)").matches
                ? "dark"
                : "light";
        } else {
            theme = themeMode;
        }

        currentAppliedTheme = theme;
        document.documentElement.setAttribute("data-theme", theme);
        localStorage.setItem("theme", theme);
        localStorage.setItem("themeMode", themeMode);
    }

    function setThemeMode(mode: ThemeMode) {
        themeMode = mode;
        applyTheme();
    }
</script>

<div class="container mx-auto max-w-2xl py-8 px-4">
    <!-- Header -->
    <div class="mb-8">
        <h1
            class="text-3xl font-bold text-base-content flex items-center gap-3"
        >
            <span class="material-symbols-rounded text-primary">settings</span>
            設定
        </h1>
        <p class="text-base-content/60 mt-2">自訂您的應用程式偏好設定</p>
    </div>

    <!-- Theme Section -->
    <div class="card bg-base-200 shadow-sm mb-6">
        <div class="card-body">
            <h2 class="card-title text-lg flex items-center gap-2">
                <span class="material-symbols-rounded text-xl">palette</span>
                主題設定
            </h2>
            <p class="text-sm text-base-content/60 mb-4">
                選擇您偏好的顯示模式
            </p>

            <div class="join w-full">
                <!-- Light Mode -->
                <button
                    class={`btn btn-sm join-item flex-1 gap-2 border-0 px-4 ${themeMode === "light" ? "bg-base-100 text-base-content shadow-inner" : "btn-ghost text-base-content/70"}`}
                    on:click={() => setThemeMode("light")}
                    title="明亮清晰"
                >
                    <span class="material-symbols-rounded text-lg"
                        >light_mode</span
                    >
                    <span class="font-medium">淺色模式</span>
                </button>

                <!-- Dark Mode -->
                <button
                    class={`btn btn-sm join-item flex-1 gap-2 border-0 px-4 ${themeMode === "dark" ? "bg-base-100 text-base-content shadow-inner" : "btn-ghost text-base-content/70"}`}
                    on:click={() => setThemeMode("dark")}
                    title="保護眼睛"
                >
                    <span class="material-symbols-rounded text-lg"
                        >dark_mode</span
                    >
                    <span class="font-medium">深色模式</span>
                </button>

                <!-- Auto Mode -->
                <button
                    class={`btn btn-sm join-item flex-1 gap-2 border-0 px-4 ${themeMode === "auto" ? "bg-base-100 text-base-content shadow-inner" : "btn-ghost text-base-content/70"}`}
                    on:click={() => setThemeMode("auto")}
                    title="跟隨系統"
                >
                    <span class="material-symbols-rounded text-lg"
                        >brightness_auto</span
                    >
                    <span class="font-medium">自動</span>
                </button>
            </div>

            <!-- Current theme indicator -->
            <div
                class="mt-4 p-3 bg-base-100 rounded-lg flex items-center gap-3 text-sm"
            >
                <span class="material-symbols-rounded text-primary">info</span>
                <span>
                    目前套用主題：<span class="font-semibold"
                        >{currentAppliedTheme === "dark"
                            ? "深色"
                            : "淺色"}</span
                    >
                    {#if themeMode === "auto"}
                        <span class="text-base-content/60">（由系統決定）</span>
                    {/if}
                </span>
            </div>
        </div>
    </div>

    <!-- Language Section (Placeholder) -->
    <div class="card bg-base-200 shadow-sm mb-6">
        <div class="card-body">
            <h2 class="card-title text-lg flex items-center gap-2">
                <span class="material-symbols-rounded text-xl">translate</span>
                語言設定
                <span class="badge badge-ghost badge-sm">即將推出</span>
            </h2>
            <p class="text-sm text-base-content/60 mb-4">
                選擇介面顯示語言（此功能尚未實作）
            </p>

            <div class="form-control">
                <select
                    class="select select-bordered w-full max-w-xs"
                    bind:value={selectedLanguage}
                    disabled
                >
                    {#each languages as lang}
                        <option value={lang.code}>
                            {lang.flag}
                            {lang.label}
                        </option>
                    {/each}
                </select>
                <div class="label">
                    <span class="label-text-alt text-base-content/50">
                        多語言支援將在未來版本中推出
                    </span>
                </div>
            </div>
        </div>
    </div>

    <!-- About Section -->
    <div class="card bg-base-200 shadow-sm">
        <div class="card-body">
            <h2 class="card-title text-lg flex items-center gap-2">
                <span class="material-symbols-rounded text-xl">info</span>
                關於
            </h2>
            <div class="text-sm space-y-2 text-base-content/70">
                <p>
                    <span class="font-medium">版本：</span>
                    <span class="badge badge-ghost">0.0.1</span>
                </p>
                <p>
                    <span class="font-medium">技術棧：</span>
                    SvelteKit + Tauri v2
                </p>
            </div>
        </div>
    </div>
</div>

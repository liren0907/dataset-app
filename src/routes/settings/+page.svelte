<script lang="ts">
    import { Card, Badge } from "$lib/components/ui";
    import { onMount } from "svelte";

    // Theme state
    type ThemeMode = "light" | "dark" | "auto";
    let themeMode: ThemeMode = $state("auto");
    let currentAppliedTheme: "light" | "dark" = $state("light");

    // Language state (placeholder only)
    let selectedLanguage = $state("zh-TW");
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
    <Card variant="base-200" shadow="sm" icon="palette" title="主題設定" class="mb-6">
            <p class="text-sm text-base-content/60 mb-4">
                選擇您偏好的顯示模式
            </p>

            <div class="join w-full">
                <!-- Light Mode -->
                <button
                    class={`btn btn-sm join-item flex-1 gap-2 border-0 px-4 ${themeMode === "light" ? "bg-base-100 text-base-content shadow-inner" : "btn-ghost text-base-content/70"}`}
                    onclick={() => setThemeMode("light")}
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
                    onclick={() => setThemeMode("dark")}
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
                    onclick={() => setThemeMode("auto")}
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
    </Card>

    <!-- Language Section (Placeholder) -->
    <Card variant="base-200" shadow="sm" icon="translate" title="語言設定" class="mb-6">
            <Badge variant="ghost" size="sm">即將推出</Badge>
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
    </Card>

    <!-- About Section -->
    <Card variant="base-200" shadow="sm" icon="info" title="關於">
            <div class="text-sm space-y-2 text-base-content/70">
                <p>
                    <span class="font-medium">版本：</span>
                    <Badge variant="ghost">0.0.1</Badge>
                </p>
                <p>
                    <span class="font-medium">技術棧：</span>
                    SvelteKit + Tauri v2
                </p>
            </div>
    </Card>
</div>

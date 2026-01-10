<script>
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { hide, show } from "@tauri-apps/api/app";
  import { onMount } from "svelte";
  import { guiStore } from "../store";

  const appWindow = getCurrentWebviewWindow();
  let isMaximized = false;

  onMount(async () => {
    isMaximized = await appWindow.isMaximized();
  });

  async function handleMinimize() {
    if ($guiStore.os === "Darwin") {
      await hide();
    } else {
      await appWindow.minimize();
    }
  }

  async function handleMaximize() {
    if ($guiStore.os === "Darwin") {
      await show();
    } else {
      await appWindow.toggleMaximize();
      isMaximized = !isMaximized;
    }
  }

  async function handleClose() {
    await appWindow.close();
  }
</script>

<div class="flex items-center gap-1">
  <!-- Spacer for macOS traffic lights -->
  <div class="w-6"></div>

  <!-- Minimize -->
  <button
    on:click={handleMinimize}
    class="btn btn-ghost btn-xs btn-square"
    aria-label="Minimize"
  >
    <span class="material-symbols-rounded icon-sm">remove</span>
  </button>

  <!-- Maximize/Restore (not on macOS) -->
  {#if $guiStore.os !== "Darwin"}
    <button
      on:click={handleMaximize}
      class="btn btn-ghost btn-xs btn-square"
      aria-label={isMaximized ? "Restore" : "Maximize"}
    >
      <span class="material-symbols-rounded icon-sm">
        {isMaximized ? "filter_none" : "crop_square"}
      </span>
    </button>
  {/if}

  <!-- Close -->
  <button
    on:click={handleClose}
    class="btn btn-ghost btn-xs btn-square hover:btn-error"
    aria-label="Close"
  >
    <span class="material-symbols-rounded icon-sm">close</span>
  </button>
</div>

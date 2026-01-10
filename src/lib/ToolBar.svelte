<script>
  import { message, open } from "@tauri-apps/plugin-dialog";
  import { fitSize } from "../funcs/image";
  import {
    clearImage,
    imageStore,
    guiStore,
    updateData,
    updateImage,
    showModal,
  } from "../store";
  import { _, locale } from "svelte-i18n";
  import * as EXIF from "exif-js";
  import { openFile, openFolder } from "../funcs/biz";
  import Donate from "./Donate.svelte";

  export let img;
  export let containerW, containerH;
  let isMenuShow = false;

  function handleZoomIn() {
    updateImage({
      scaleX: $imageStore.scaleX * 1.05,
      scaleY: $imageStore.scaleY * 1.05,
    });
  }

  function handleZoomOut() {
    updateImage({
      scaleX: $imageStore.scaleX * 0.95,
      scaleY: $imageStore.scaleY * 0.95,
    });
  }

  function handleRotateCCW() {
    if ($imageStore.rotation === -270) {
      updateImage({ rotation: 0 });
    } else {
      updateImage({ rotation: $imageStore.rotation - 90 });
    }
  }

  function handleRotateCW() {
    if ($imageStore.rotation === 270) {
      updateImage({ rotation: 0 });
    } else {
      updateImage({ rotation: $imageStore.rotation + 90 });
    }
  }

  function handleFitToScreen() {
    const result = fitSize(img, containerW, containerH, 36);
    updateImage({
      scaleX: result.ratio,
      scaleY: result.ratio,
      pointX: result.offsetX,
      pointY: result.offsetY,
    });
  }

  function handleResetView() {
    if (img.naturalWidth <= containerW && img.naturalHeight <= containerH) {
      const result = fitSize(img, containerW, containerH, 36);
      updateImage({
        scaleX: result.ratio,
        scaleY: result.ratio,
        pointX: result.offsetX,
        pointY: result.offsetY,
      });
    } else {
      updateImage({
        scaleX: 1.0,
        scaleY: 1.0,
        pointX: 0,
        pointY: 0,
      });
    }
  }

  function handleFlipH() {
    updateImage({
      scaleX: $imageStore.scaleX * -1,
      scaleY: $imageStore.scaleY,
    });
  }

  function handleFlipV() {
    updateImage({
      scaleX: $imageStore.scaleX,
      scaleY: $imageStore.scaleY * -1,
    });
  }

  function handleShowExif() {
    EXIF.getData(img, function () {
      const allMetaData = EXIF.getAllTags(this);
      $imageStore.exif = allMetaData;
    });
    isMenuShow = false;
  }

  function handleToggleLocale() {
    if ($guiStore.locale === "en-US") {
      locale.set("zh-CN");
      $guiStore.locale = "zh-CN";
    } else {
      locale.set("en-US");
      $guiStore.locale = "en-US";
    }
  }

  function handleDonate() {
    showModal(Donate);
    isMenuShow = false;
  }

  function handleClearImage() {
    clearImage();
    isMenuShow = false;
  }
</script>

<div class="flex items-center gap-1">
  <!-- Open File -->
  <div class="tooltip tooltip-bottom" data-tip={$_("open_image_file")}>
    <button
      class="btn btn-ghost btn-sm btn-square"
      on:click={async () => await openFile()}
    >
      <span class="material-symbols-rounded">add_photo_alternate</span>
    </button>
  </div>

  <!-- Open Folder -->
  <div class="tooltip tooltip-bottom" data-tip={$_("open_folder")}>
    <button
      class="btn btn-ghost btn-sm btn-square"
      on:click={async () => await openFolder()}
    >
      <span class="material-symbols-rounded">folder_open</span>
    </button>
  </div>

  <div class="divider divider-horizontal mx-1 h-6"></div>

  <!-- Zoom In -->
  <div class="tooltip tooltip-bottom" data-tip={$_("zoom_in")}>
    <button class="btn btn-ghost btn-sm btn-square" on:click={handleZoomIn}>
      <span class="material-symbols-rounded">zoom_in</span>
    </button>
  </div>

  <!-- Zoom Out -->
  <div class="tooltip tooltip-bottom" data-tip={$_("zoom_out")}>
    <button class="btn btn-ghost btn-sm btn-square" on:click={handleZoomOut}>
      <span class="material-symbols-rounded">zoom_out</span>
    </button>
  </div>

  <div class="divider divider-horizontal mx-1 h-6"></div>

  <!-- Rotate Counter-Clockwise -->
  <div class="tooltip tooltip-bottom" data-tip={$_("rotate_anti-clockwise")}>
    <button class="btn btn-ghost btn-sm btn-square" on:click={handleRotateCCW}>
      <span class="material-symbols-rounded">rotate_left</span>
    </button>
  </div>

  <!-- Rotate Clockwise -->
  <div class="tooltip tooltip-bottom" data-tip={$_("rotate_clockwise")}>
    <button class="btn btn-ghost btn-sm btn-square" on:click={handleRotateCW}>
      <span class="material-symbols-rounded">rotate_right</span>
    </button>
  </div>

  <div class="divider divider-horizontal mx-1 h-6"></div>

  <!-- Fit to Screen -->
  <div class="tooltip tooltip-bottom" data-tip={$_("fit_to")}>
    <button
      class="btn btn-ghost btn-sm btn-square"
      on:click={handleFitToScreen}
    >
      <span class="material-symbols-rounded">fit_screen</span>
    </button>
  </div>

  <!-- Reset View -->
  <div class="tooltip tooltip-bottom" data-tip={$_("restore_to_origin")}>
    <button class="btn btn-ghost btn-sm btn-square" on:click={handleResetView}>
      <span class="material-symbols-rounded">aspect_ratio</span>
    </button>
  </div>

  <div class="divider divider-horizontal mx-1 h-6"></div>

  <!-- Flip Horizontal -->
  <div class="tooltip tooltip-bottom" data-tip={$_("flip_h")}>
    <button class="btn btn-ghost btn-sm btn-square" on:click={handleFlipH}>
      <span class="material-symbols-rounded">flip</span>
    </button>
  </div>

  <!-- Flip Vertical -->
  <div class="tooltip tooltip-bottom" data-tip={$_("flip_v")}>
    <button class="btn btn-ghost btn-sm btn-square" on:click={handleFlipV}>
      <span class="material-symbols-rounded" style="transform: rotate(90deg);"
        >flip</span
      >
    </button>
  </div>

  <!-- More Options Dropdown -->
  <div class="dropdown dropdown-end">
    <div tabindex="0" role="button" class="btn btn-ghost btn-sm btn-square">
      <span class="material-symbols-rounded">more_vert</span>
    </div>
    <ul
      tabindex="0"
      class="dropdown-content menu bg-base-200 rounded-box z-50 w-52 p-2 shadow-lg"
    >
      <li>
        <button on:click={handleShowExif}>
          <span class="material-symbols-rounded">info</span>
          {$_("show_exif")}
        </button>
      </li>
      <li>
        <button on:click={handleToggleLocale}>
          <span class="material-symbols-rounded">language</span>
          {$_("change_locale")}
          {$guiStore.locale === "en-US" ? "中文" : "English"}
        </button>
      </li>
      <li>
        <button on:click={handleDonate}>
          <span class="material-symbols-rounded">volunteer_activism</span>
          {$_("donate")}
        </button>
      </li>
      <li>
        <a
          href="https://github.com/ChqJourney/ImageProc"
          target="_blank"
          rel="noopener"
        >
          <span class="material-symbols-rounded">help</span>
          {$_("about")}
        </a>
      </li>
      <div class="divider my-1"></div>
      <li>
        <button on:click={handleClearImage} class="text-error">
          <span class="material-symbols-rounded">close</span>
          {$_("close_img")}
        </button>
      </li>
    </ul>
  </div>
</div>

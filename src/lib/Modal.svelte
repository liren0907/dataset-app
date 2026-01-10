<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();
  export let isOpen;
  export let component;
  export let data = {};
  export let width = "max-w-2xl";
  export let title = "";

  function handleClose() {
    dispatch("negitive");
  }

  function handleBackdropClick(e) {
    if (e.target === e.currentTarget) {
      handleClose();
    }
  }
</script>

<!-- DaisyUI Modal -->
<dialog class="modal" class:modal-open={isOpen}>
  <div class="modal-box {width}">
    <!-- Header with close button -->
    <div class="flex items-center justify-between mb-4">
      {#if title}
        <h3 class="font-bold text-lg">{title}</h3>
      {:else}
        <div></div>
      {/if}
      <button class="btn btn-sm btn-circle btn-ghost" on:click={handleClose}>
        <span class="material-symbols-rounded">close</span>
      </button>
    </div>

    <!-- Modal content -->
    <div class="py-4">
      <svelte:component this={component} {...data} />
    </div>
  </div>

  <!-- Backdrop -->
  <form method="dialog" class="modal-backdrop">
    <button on:click={handleClose}>close</button>
  </form>
</dialog>

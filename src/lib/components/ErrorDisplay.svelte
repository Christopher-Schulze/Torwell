<script lang="ts">
  import { uiStore } from '$lib/stores/uiStore';
  function close() {
    uiStore.actions.clearError();
  }
</script>
{#if $uiStore.error}
  <div role="alert" class="fixed top-4 right-4 bg-red-700 text-white px-4 py-2 rounded shadow-lg z-50 flex gap-2 items-start">
    <span class="flex-1">{$uiStore.error}</span>
    <button on:click={close} aria-label="Close error" class="font-bold">✕</button>
  </div>
{/if}

<script lang="ts">
  import { torStore } from "$lib/stores/torStore";
  import { X } from "lucide-svelte";

  function dismiss() {
    torStore.update((s) => ({ ...s, securityWarning: null }));
  }
</script>

{#if $torStore.securityWarning}
<div class="bg-yellow-900/80 text-yellow-200 p-2 rounded-lg mb-3 flex items-center justify-between" role="alert" tabindex="0">
  <span class="text-sm">{$torStore.securityWarning}</span>
  <button on:click={dismiss} aria-label="Dismiss warning" class="ml-2">
    <X size={14} />
  </button>
</div>
{/if}

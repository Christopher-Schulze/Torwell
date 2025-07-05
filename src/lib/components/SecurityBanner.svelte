<script lang="ts">
  import { torStore } from "$lib/stores/torStore";
  import { X } from "lucide-svelte";

  function dismiss() {
    torStore.update((s) => ({ ...s, securityWarning: null }));
  }
</script>

{#if $torStore.securityWarning}
  <div
    class="glass-sm p-2 rounded-lg mb-3 flex items-center justify-between"
    role="alert"
    tabindex="0"
    aria-label="Security warning"
  >
    <span class="text-sm">{$torStore.securityWarning}</span>
    <button
      on:click={dismiss}
      aria-label="Dismiss warning"
      class="ml-2 p-1 w-6 h-6 flex items-center justify-center"
    >
      <X size={14} />
    </button>
  </div>
{/if}

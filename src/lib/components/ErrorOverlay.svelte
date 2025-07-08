<script lang="ts">
import { errorStore } from '$lib/stores/errorStore';
import { onDestroy } from 'svelte';

let error: Error | null = null;
const unsub = errorStore.subscribe((e: Error | null) => (error = e));
onDestroy(unsub);
</script>

{#if error}
  <div class="fixed inset-0 bg-black/70 flex items-center justify-center z-50" role="alertdialog" aria-modal="true" aria-labelledby="error-overlay-title">
    <div class="glass-sm p-4 rounded max-w-md">
      <p id="error-overlay-title" class="font-semibold mb-2">An unexpected error occurred.</p>
      <pre class="text-xs mb-4">{error.message}</pre>
      <button class="px-3 py-1 bg-black rounded" on:click={() => errorStore.set(null)} aria-label="Dismiss error">Close</button>
    </div>
  </div>
{/if}

<script lang="ts">
import { errorStore } from './AppErrorBoundary.svelte';
import { onDestroy } from 'svelte';

let error: Error | null = null;
const unsub = errorStore.subscribe((e) => (error = e));
onDestroy(unsub);
</script>

{#if error}
  <div class="fixed inset-0 bg-black/70 flex items-center justify-center z-50" role="alertdialog" aria-modal="true">
    <div class="bg-red-800 text-white p-4 rounded max-w-md">
      <p class="font-semibold mb-2">An unexpected error occurred.</p>
      <pre class="text-xs mb-4">{error.message}</pre>
      <button class="px-3 py-1 bg-black rounded" on:click={() => errorStore.set(null)} aria-label="Dismiss error">Close</button>
    </div>
  </div>
{/if}

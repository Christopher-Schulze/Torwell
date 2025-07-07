<script lang="ts">
  import { toasts } from '$lib/stores/toastStore';
</script>

<div class="fixed top-2 right-2 flex flex-col gap-2 z-50" aria-live="assertive">
  {#each $toasts as t (t.id)}
    <div class="glass-sm px-3 py-2 rounded text-sm text-white {t.type === 'error' ? 'bg-red-600/80' : 'bg-black/70'}">
      {t.message}
    </div>
  {/each}
</div>

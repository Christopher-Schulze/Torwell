<script lang="ts">
  import { torStore } from '$lib/stores/torStore';
</script>

{#if $torStore.securityWarning}
  <div class="fixed top-4 right-4 bg-yellow-400 text-black text-xs font-bold px-2 py-1 rounded-full z-50 pointer-events-none" role="alert" aria-label="Security warning active">
    !
  </div>
{/if}

<script lang="ts">
  import NetworkMonitor from '$lib/components/NetworkMonitor.svelte';
  import { goto } from '$app/navigation';
  function back() {
    goto('/');
  }
</script>

<div class="p-6 max-w-6xl mx-auto">
  <button class="text-blue-400 underline" on:click={back}>Back</button>
  <NetworkMonitor class="mt-4" />
</div>

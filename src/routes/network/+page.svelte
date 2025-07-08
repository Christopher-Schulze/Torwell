<script lang="ts">
  import NetworkMonitor from '$lib/components/NetworkMonitor.svelte';
  export const ssr = false;
</script>

<div class="p-6 max-w-6xl mx-auto">
  <a href={'/'} class="text-blue-400 underline">Back</a>
  <NetworkMonitor className="mt-4" />
</div>

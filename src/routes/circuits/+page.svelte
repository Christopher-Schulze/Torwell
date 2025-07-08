<script lang="ts">
  import CircuitManager from '$lib/components/CircuitManager.svelte';
  export const ssr = false;
</script>

<div class="p-6 max-w-6xl mx-auto">
  <a href={'/'} class="text-blue-400 underline">Back</a>
  <CircuitManager className="mt-4" />
</div>

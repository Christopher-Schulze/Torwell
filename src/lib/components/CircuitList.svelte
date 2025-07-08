<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '$lib/api';
  export let show = false;
  let circuits: number[] = [];

  async function refresh() {
    if (!show) return;
    try {
      circuits = (await invoke('list_circuits')) as number[];
    } catch (e) {
      console.error('list_circuits failed', e);
      circuits = [];
    }
  }

  async function close(id: number) {
    try {
      await invoke('close_circuit', { id });
      await refresh();
    } catch (e) {
      console.error('close_circuit failed', e);
    }
  }

  onMount(refresh);
  $: if (show) refresh();
</script>

{#if show}
<div class="glass-md rounded-xl p-4 mt-4" aria-label="Circuits">
  <h3 class="text-base font-medium text-white mb-2">Circuits</h3>
  <ul class="space-y-1">
    {#each circuits as id}
      <li class="flex items-center justify-between text-xs text-white bg-black/50 rounded px-2 py-1">
        <span>#{id}</span>
        <button class="text-red-200 hover:text-red-400" on:click={() => close(id)}>Close</button>
      </li>
    {/each}
    {#if circuits.length === 0}
      <li class="text-gray-300">No circuits</li>
    {/if}
  </ul>
</div>
{/if}

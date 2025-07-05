<script lang="ts">
  import { invoke } from "$lib/api";
  let host = "";
  let dns: string[] = [];
  let route: string[] = [];
  let loading = false;

  async function lookup() {
    if (!host) return;
    loading = true;
    try {
      dns = (await invoke("dns_lookup", { host })) as string[];
    } catch (e) {
      dns = ["error"];
    } finally {
      loading = false;
    }
  }

  async function trace() {
    if (!host) return;
    loading = true;
    try {
      route = (await invoke("traceroute_host", { host, maxHops: 8 })) as string[];
    } catch (e) {
      route = ["error"];
    } finally {
      loading = false;
    }
  }
</script>

<div class="glass-md rounded-xl p-4 flex flex-col gap-2" aria-label="Network tools">
  <div>
    <label class="text-sm text-white">Host</label>
    <input class="ml-2 p-1 rounded text-black" bind:value={host} />
  </div>
  <div class="flex gap-2">
    <button class="glass px-2 py-1 rounded" on:click|preventDefault={lookup} disabled={loading}>DNS Lookup</button>
    <button class="glass px-2 py-1 rounded" on:click|preventDefault={trace} disabled={loading}>Traceroute</button>
  </div>
  {#if dns.length}
    <div class="text-xs text-white break-all">DNS: {dns.join(", ")}</div>
  {/if}
  {#if route.length}
    <div class="text-xs text-white break-all">Route: {route.join(" -> ")}</div>
  {/if}
</div>

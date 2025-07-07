<script lang="ts">
  import { invoke } from "$lib/api";
  import { addToast, addErrorToast } from "$lib/stores/toastStore";
  let host = "";
  let dns: string[] = [];
  let route: string[] = [];
  let countries: string[] = [];
  let loading = false;

  async function lookupCountry(ip: string): Promise<string> {
    try {
      const res = await fetch(`https://ipapi.co/${ip}/country/`);
      if (res.ok) {
        return (await res.text()).trim();
      }
    } catch (_) {
      // ignore
    }
    return "??";
  }

  function copyDns() {
    navigator.clipboard.writeText(dns.join('\n'));
    addToast('DNS results copied');
  }

  function copyRoute() {
    const text = route.map((ip, i) => `${i + 1}. ${ip}`).join('\n');
    navigator.clipboard.writeText(text);
    addToast('Traceroute copied');
  }

  async function lookup() {
    if (!host) return;
    loading = true;
    try {
      dns = (await invoke("dns_lookup", { host })) as string[];
    } catch (e: any) {
      dns = [];
      addErrorToast('dns', e?.message ?? String(e));
    } finally {
      loading = false;
    }
  }

  async function trace() {
    if (!host) return;
    loading = true;
    try {
      route = (await invoke("traceroute_host", { host, maxHops: 8 })) as string[];
      countries = await Promise.all(route.map((ip) => lookupCountry(ip)));
    } catch (e: any) {
      route = [];
      countries = [];
      addErrorToast('traceroute', e?.message ?? String(e));
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
    <div class="flex items-center justify-between">
      <h3 class="text-sm text-white">DNS Results</h3>
      <button class="glass px-1 rounded text-xs" on:click={copyDns}>Copy</button>
    </div>
    <table class="text-xs text-white w-full" aria-label="DNS results">
      <thead>
        <tr><th class="text-left">IP Address</th></tr>
      </thead>
      <tbody>
        {#each dns as ip}
          <tr class="odd:bg-black/30"><td class="px-1 py-0.5">{ip}</td></tr>
        {/each}
      </tbody>
    </table>
  {/if}

  {#if route.length}
    <div class="flex items-center justify-between mt-2">
      <h3 class="text-sm text-white">Traceroute</h3>
      <button class="glass px-1 rounded text-xs" on:click={copyRoute}>Copy</button>
    </div>
    <table class="text-xs text-white w-full" aria-label="Traceroute results">
      <thead>
        <tr><th class="text-left">Hop</th><th class="text-left">IP Address</th></tr>
      </thead>
      <tbody>
        {#each route as ip, i}
          <tr class="odd:bg-black/30">
            <td class="px-1 py-0.5">{i + 1}</td>
            <td class="px-1 py-0.5">{ip}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    {#if countries.length}
      <div class="flex flex-wrap gap-1 mt-1" aria-label="Traceroute countries">
        {#each countries as cc}
          <span class="text-xs bg-black/40 text-white px-1 rounded">{cc}</span>
        {/each}
      </div>
    {/if}
  {/if}
</div>

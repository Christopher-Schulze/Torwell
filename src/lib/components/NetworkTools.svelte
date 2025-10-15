<script lang="ts">
  import { invoke } from "$lib/api";
  import { torStore } from "$lib/stores/torStore";
  import { addToast, addErrorToast } from "$lib/stores/toastStore";
  import { getCountryFlag, getCountryLabel } from "$lib/utils/countries";

  type ToolAction = "dns" | "trace" | null;

  const quickTargets = [
    { label: "Tor Check", value: "check.torproject.org" },
    { label: "Tor Project", value: "torproject.org" },
    { label: "Cloudflare DNS", value: "1.1.1.1" },
    { label: "Google DNS", value: "8.8.8.8" },
  ];

  const parseError = (error: unknown) =>
    error instanceof Error ? error.message : String(error ?? "Unknown error");

  let host = "";
  let dns: string[] = [];
  let route: string[] = [];
  let countries: Array<string | null> = [];
  let action: ToolAction = null;
  let dnsError: string | null = null;
  let traceError: string | null = null;

  const trimmedHost = () => host.trim();

  async function lookupCountryLocal(ip: string): Promise<string | null> {
    try {
      const res = (await invoke("lookup_country", { ip })) as string;
      const code = res.trim().toUpperCase();
      return /^[A-Z]{2}$/.test(code) ? code : null;
    } catch (error) {
      console.warn("Country lookup failed", error);
      return null;
    }
  }

  function resetErrors() {
    dnsError = null;
    traceError = null;
  }

  function selectQuickTarget(value: string) {
    host = value;
    resetErrors();
  }

  function ensureReady(actionName: string): string | null {
    const trimmed = trimmedHost();
    if ($torStore.status !== "CONNECTED") {
      addErrorToast(
        "connection",
        `${actionName} requires an active Tor connection.`,
      );
      return null;
    }
    if (!trimmed) {
      addErrorToast("input", "Please enter a host or IP address first.");
      return null;
    }
    return trimmed;
  }

  async function copyToClipboard(text: string, label: string) {
    if (!text) return;
    try {
      if (typeof navigator === "undefined" || !navigator.clipboard) {
        throw new Error("Clipboard API is not available in this environment.");
      }
      await navigator.clipboard.writeText(text);
      addToast(`${label} copied to clipboard`);
    } catch (error) {
      addErrorToast("clipboard", parseError(error));
    }
  }

  async function runDnsLookup() {
    const target = ensureReady("DNS lookup");
    if (!target || action) return;
    action = "dns";
    dnsError = null;
    try {
      const results = (await invoke("dns_lookup", { host: target })) as string[];
      dns = Array.isArray(results)
        ? results.filter((value) => typeof value === "string" && value.trim().length > 0)
        : [];
      if (dns.length === 0) {
        addToast("No DNS records were returned for this host.");
      } else {
        addToast(`Resolved ${dns.length} ${dns.length === 1 ? "record" : "records"}.`);
      }
    } catch (error) {
      dns = [];
      const message = parseError(error);
      dnsError = message;
      addErrorToast("dns", message);
    } finally {
      action = null;
    }
  }

  async function runTraceroute() {
    const target = ensureReady("Traceroute");
    if (!target || action) return;
    action = "trace";
    traceError = null;
    try {
      route = (await invoke("traceroute_host", { host: target, maxHops: 8 })) as string[];
      if (route.length === 0) {
        addToast("Traceroute did not return any hops.");
        countries = [];
        return;
      }
      countries = await Promise.all(route.map((ip) => lookupCountryLocal(ip)));
      addToast(`Traceroute returned ${route.length} hops.`);
    } catch (error) {
      route = [];
      countries = [];
      const message = parseError(error);
      traceError = message;
      addErrorToast("traceroute", message);
    } finally {
      action = null;
    }
  }

  function clearResults() {
    dns = [];
    route = [];
    countries = [];
    dnsError = null;
    traceError = null;
  }

  function copyDns() {
    copyToClipboard(dns.join("\n"), "DNS results");
  }

  function copyRoute() {
    const text = route.map((ip, index) => `${index + 1}. ${ip}`).join("\n");
    copyToClipboard(text, "Traceroute");
  }

  $: isConnected = $torStore.status === "CONNECTED";
  $: isPending = action !== null;
  $: hasResults = dns.length > 0 || route.length > 0;
  $: statusLabel = (() => {
    switch ($torStore.status) {
      case "CONNECTED":
        return "Connected";
      case "CONNECTING":
      case "RETRYING":
        return "Connecting…";
      case "DISCONNECTING":
        return "Disconnecting…";
      default:
        return "Disconnected";
    }
  })();
  $: badgeState = (() => {
    if (isPending) return "pending";
    if ($torStore.status === "CONNECTED") return "connected";
    if ($torStore.status === "DISCONNECTED" || $torStore.status === "ERROR") return "error";
    return "idle";
  })();
  $: routeRows = route.map((ip, index) => {
    const code = countries[index] ?? null;
    return {
      hop: index + 1,
      ip,
      code,
      flag: getCountryFlag(code),
      label: code ? getCountryLabel(code) : "Unknown",
    };
  });
</script>

<div class="glass-md rounded-2xl p-6 space-y-6" role="region" aria-label="Network tools">
  <header class="flex flex-wrap items-center justify-between gap-4">
    <div class="space-y-1">
      <p class="tw-section-title">Network Diagnostics</p>
      <h2 class="text-xl font-semibold tracking-tight text-slate-50">Resolve and trace through your Tor connection</h2>
      <p class="text-sm text-slate-300/80 max-w-2xl">
        Use these tools to confirm DNS resolution and hop-level routing while connected to the Tor network. Each request
        runs through the current Tor circuit so you can validate exit behaviour and latency.
      </p>
    </div>
    <span class="tw-badge" data-state={badgeState} aria-live="polite">{statusLabel}</span>
  </header>

  {#if !isConnected}
    <p class="tw-inline-alert" role="status">
      Connect to Tor to enable the diagnostic tools.
    </p>
  {/if}

  <form class="space-y-4" on:submit|preventDefault={runDnsLookup}>
    <div class="space-y-2">
      <label class="tw-section-title" for="host-input">Target host or address</label>
      <input
        id="host-input"
        class="tw-input"
        type="text"
        placeholder="e.g. check.torproject.org"
        bind:value={host}
        autocomplete="off"
        spellcheck={false}
        aria-required="true"
      />
    </div>

    <div class="flex flex-wrap gap-2" aria-label="Quick targets">
      {#each quickTargets as target}
        <button
          type="button"
          class="tw-chip"
          on:click={() => selectQuickTarget(target.value)}
          disabled={isPending}
        >
          {target.label}
        </button>
      {/each}
    </div>

    <div class="flex flex-col gap-3 sm:flex-row sm:items-center">
      <div class="flex flex-1 flex-wrap gap-2">
        <button
          type="submit"
          class="tw-button tw-button--accent flex-1 sm:flex-none"
          aria-label="Run DNS lookup"
          disabled={!isConnected || isPending || !trimmedHost()}
          data-state={action === "dns" ? "pending" : isConnected && trimmedHost() ? "active" : "idle"}
        >
          {#if action === "dns"}
            <span class="tw-spinner" aria-hidden="true"></span>
            Resolving…
          {:else}
            DNS Lookup
          {/if}
        </button>

        <button
          type="button"
          class="tw-button tw-button--neutral flex-1 sm:flex-none"
          on:click={runTraceroute}
          aria-label="Run traceroute"
          disabled={!isConnected || isPending || !trimmedHost()}
          data-state={action === "trace" ? "pending" : isConnected && trimmedHost() ? "active" : "idle"}
        >
          {#if action === "trace"}
            <span class="tw-spinner" aria-hidden="true"></span>
            Tracing…
          {:else}
            Traceroute
          {/if}
        </button>
      </div>

      <button
        type="button"
        class="tw-button tw-button--neutral"
        on:click={clearResults}
        aria-label="Clear results"
        disabled={!hasResults || isPending}
      >
        Clear
      </button>
    </div>
  </form>

  <div class="grid gap-4 lg:grid-cols-2">
    <section class="tw-results-card" aria-live="polite">
      <header class="tw-results-header">
        <h3>DNS Results</h3>
        <button
          type="button"
          class="tw-chip tw-chip--action"
          on:click={copyDns}
          disabled={dns.length === 0 || isPending}
        >
          Copy
        </button>
      </header>
      {#if dnsError}
        <p class="tw-inline-alert tw-inline-alert--error" role="alert">{dnsError}</p>
      {:else if action === "dns"}
        <p class="tw-subtle-text">Resolving records…</p>
      {:else if dns.length === 0}
        <p class="tw-empty-state">Run a lookup to populate DNS records.</p>
      {:else}
        <table class="tw-data-table" aria-label="DNS results">
          <thead>
            <tr>
              <th scope="col">#</th>
              <th scope="col">Address</th>
            </tr>
          </thead>
          <tbody>
            {#each dns as ip, index}
              <tr>
                <td data-title="Record">{index + 1}</td>
                <td data-title="Address">{ip}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="tw-results-card" aria-live="polite">
      <header class="tw-results-header">
        <h3>Traceroute</h3>
        <button
          type="button"
          class="tw-chip tw-chip--action"
          on:click={copyRoute}
          disabled={route.length === 0 || isPending}
        >
          Copy
        </button>
      </header>
      {#if traceError}
        <p class="tw-inline-alert tw-inline-alert--error" role="alert">{traceError}</p>
      {:else if action === "trace"}
        <p class="tw-subtle-text">Running traceroute…</p>
      {:else if route.length === 0}
        <p class="tw-empty-state">Traceroute results will appear here.</p>
      {:else}
        <table class="tw-data-table" aria-label="Traceroute results">
          <thead>
            <tr>
              <th scope="col">Hop</th>
              <th scope="col">IP</th>
              <th scope="col">Country</th>
            </tr>
          </thead>
          <tbody>
            {#each routeRows as row}
              <tr>
                <td data-title="Hop">{row.hop}</td>
                <td data-title="IP">{row.ip}</td>
                <td data-title="Country">
                  <span class="inline-flex items-center gap-2">
                    <span aria-hidden="true">{row.flag}</span>
                    <span>{row.label}</span>
                    {#if row.code}
                      <span class="tw-chip tw-chip--code">{row.code}</span>
                    {/if}
                  </span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>
  </div>
</div>

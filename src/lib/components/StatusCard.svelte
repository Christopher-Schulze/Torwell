<script lang="ts">
  import { Activity, Cpu, Gauge, Zap, Shield } from "lucide-svelte";
  import { invoke } from "$lib/api";
  import MetricsChart from "./MetricsChart.svelte";
  import CircuitList from "./CircuitList.svelte";
  import GlassCard from "./GlassCard.svelte";
  import type { CircuitPolicyReport, StatusSummary } from "$lib/types";
  import type { TorStatus } from "$lib/stores/torStore";
  import { reducedMotion } from "$lib/utils/motion";

  export let status;
  export let totalTrafficMB = 0;
  export let pingMs: number | undefined = undefined;
  export let summary: StatusSummary | null = null;
  export let policyReport: CircuitPolicyReport | null = null;

  import { torStore } from "$lib/stores/torStore";
  import { uiStore } from "$lib/stores/uiStore";
  import {
    ensureUniqueRoute,
    getCountryFlag,
    getCountryLabel,
    isFastCountry,
    normaliseCountryCode,
  } from "$lib/utils/countries";
  let memoryMB: number;
  let circuitCount: number;
  let metrics = [];
  let pingHistory: number[] = [];
  const width = 120;
  const height = 40;
  const pingHistoryLimit = 30;

  function buildPath(data: number[]): string {
    if (data.length === 0) return "";
    const maxVal = Math.max(...data, 1);
    const step = width / Math.max(data.length - 1, 1);
    let d = `M0 ${height}`;
    data.forEach((pt, idx) => {
      const x = idx * step;
      const y = height - (pt / maxVal) * height;
      d += ` L${x} ${y}`;
    });
    d += ` L${width} ${height} Z`;
    return d;
  }

  $: pingPath = buildPath(pingHistory);
  $: memoryMB = $torStore.memoryUsageMB;
  $: circuitCount = $torStore.circuitCount;
  $: metrics = $torStore.metrics;
  $: proxyActive = $torStore.systemProxyEnabled && status === "CONNECTED";

  const statusStyles: Record<
    TorStatus,
    { label: string; caption: string; indicator: string; gradient: string; aura: string }
  > = {
    DISCONNECTED: {
      label: "Disconnected",
      caption: "Bereit für eine neue Sitzung",
      indicator: "bg-rose-400",
      gradient: "from-slate-900/70 via-slate-800/40 to-slate-900/60",
      aura: "bg-gradient-to-tr from-slate-600/20 via-transparent to-slate-900/60",
    },
    CONNECTING: {
      label: "Connecting",
      caption: "Tor bootstrap läuft",
      indicator: "bg-amber-300",
      gradient: "from-amber-500/25 via-purple-500/10 to-slate-900/60",
      aura: "bg-gradient-to-tr from-amber-400/25 via-transparent to-purple-500/20",
    },
    RETRYING: {
      label: "Retrying",
      caption: "Erneuter Verbindungsversuch",
      indicator: "bg-amber-300",
      gradient: "from-amber-500/25 via-purple-500/10 to-slate-900/60",
      aura: "bg-gradient-to-tr from-amber-500/20 via-transparent to-rose-500/20",
    },
    CONNECTED: {
      label: "Connected",
      caption: "Tor-Tunnel aktiv",
      indicator: "bg-emerald-400",
      gradient: "from-emerald-500/25 via-sky-500/10 to-slate-900/60",
      aura: "bg-gradient-to-tr from-emerald-400/25 via-transparent to-indigo-500/20",
    },
    DISCONNECTING: {
      label: "Disconnecting",
      caption: "Sitzung wird getrennt",
      indicator: "bg-blue-300",
      gradient: "from-blue-500/25 via-slate-800/40 to-slate-900/60",
      aura: "bg-gradient-to-tr from-blue-400/25 via-transparent to-slate-700/20",
    },
    ERROR: {
      label: "Error",
      caption: "Fehlerdetails prüfen",
      indicator: "bg-rose-400",
      gradient: "from-rose-500/30 via-amber-400/10 to-slate-900/60",
      aura: "bg-gradient-to-tr from-rose-500/25 via-transparent to-amber-400/20",
    },
  };

  $: statusInfo = statusStyles[status as TorStatus] ?? statusStyles.DISCONNECTED;

  let isPinging = false;

  const ROUTE_ROLES = ["Entry Node", "Middle Node", "Exit Node"] as const;

  $: configuredRoute = [
    $uiStore.settings.entryCountry,
    $uiStore.settings.middleCountry,
    $uiStore.settings.exitCountry,
  ];

  $: normalisedConfiguredRoute = configuredRoute.map((value) =>
    normaliseCountryCode(value),
  ) as Array<string | null>;

  $: requestedRoute = policyReport
    ? [
        normaliseCountryCode(policyReport.requested_entry),
        normaliseCountryCode(policyReport.requested_middle),
        normaliseCountryCode(policyReport.requested_exit),
      ]
    : normalisedConfiguredRoute;

  $: fallbackRoute = ensureUniqueRoute(requestedRoute);

  $: effectiveCandidate = policyReport
    ? [
        normaliseCountryCode(policyReport.effective_entry),
        normaliseCountryCode(policyReport.effective_middle),
        normaliseCountryCode(policyReport.effective_exit),
      ]
    : [null, null, null];

  $: hasEffectiveCandidate = effectiveCandidate.some((code) => !!code);

  $: effectiveRoute = hasEffectiveCandidate
    ? effectiveCandidate.map((code, index) => code ?? fallbackRoute[index])
    : fallbackRoute;

  $: routeDisplay = effectiveRoute.map((code, index) => {
    const requested = requestedRoute[index];
    let statusLabel: "locked" | "fallback" | "auto";
    if (requested) {
      statusLabel = requested === code ? "locked" : "fallback";
    } else if (policyReport && hasEffectiveCandidate && !policyReport.matches_policy) {
      statusLabel = "fallback";
    } else {
      statusLabel = "auto";
    }
    return {
      role: ROUTE_ROLES[index],
      code,
      flag: getCountryFlag(code),
      label: getCountryLabel(code),
      status: statusLabel,
      requestedLabel: requested ? getCountryLabel(requested) : null,
      isFast: isFastCountry(code),
    };
  });

  $: routePolicyState = policyReport
    ? hasEffectiveCandidate
      ? policyReport.matches_policy
        ? "All pinned countries satisfied."
        : "Pinned route not fully available – using fallback countries."
      : "Awaiting live circuit details…"
    : null;

  $: routePolicyTone = policyReport
    ? hasEffectiveCandidate
      ? policyReport.matches_policy
        ? "text-emerald-300"
        : "text-amber-300"
      : "text-slate-300"
    : "text-slate-400";

  // Format traffic display with automatic MB/GB conversion
  function formatTraffic(mb: number): string {
    if (!Number.isFinite(mb) || mb <= 0) {
      return "0 MB";
    }
    if (mb >= 1000) {
      return `${(mb / 1000).toFixed(1)} GB`;
    }
    return mb >= 10 ? `${Math.round(mb)} MB` : `${mb.toFixed(1)} MB`;
  }

  function formatThroughput(bytes: number | null | undefined): string {
    if (bytes == null) return "-";
    if (bytes >= 1_000_000) {
      return `${(bytes / 1_000_000).toFixed(1)} MB/s`;
    }
    if (bytes >= 1_000) {
      return `${(bytes / 1_000).toFixed(1)} KB/s`;
    }
    return `${Math.round(bytes)} B/s`;
  }

  function formatDuration(seconds: number | null | undefined): string {
    if (seconds == null) return "-";
    const total = Math.max(0, Math.floor(seconds));
    const days = Math.floor(total / 86400);
    const hours = Math.floor((total % 86400) / 3600);
    const minutes = Math.floor((total % 3600) / 60);
    const secs = total % 60;
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    if (minutes > 0) return `${minutes}m ${secs}s`;
    return `${secs}s`;
  }

  function formatTimestamp(value: string | null | undefined): string {
    if (!value) return "-";
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return "-";
    return date.toLocaleString();
  }

  $: uptimeLabel = formatDuration(summary?.uptime_seconds ?? null);
  $: throughputLabel = formatThroughput(summary?.network_bytes_per_sec ?? null);
  $: connectedSinceLabel = formatTimestamp(summary?.connected_since ?? null);
  $: oldestCircuitLabel = formatDuration(summary?.oldest_circuit_age ?? null);

  // Ping function - execute backend ping command
  async function performPing() {
    if (isPinging) return;
    isPinging = true;
    try {
      const result = (await invoke("ping_host_series", {
        host: "google.com",
        count: 5,
      })) as number[];
      if (result.length) {
        pingMs = Math.round(result.reduce((a, b) => a + b, 0) / result.length);
        pingHistory = [...pingHistory, ...result].slice(-pingHistoryLimit);
      }
    } catch (error) {
      console.error("Ping failed:", error);
      pingMs = -1;
    } finally {
      isPinging = false;
    }
  }

  $: highlightMetrics = [
    {
      label: "Traffic",
      value: formatTraffic(totalTrafficMB),
      icon: Activity,
      accent: "from-sky-300/80 via-indigo-400/80 to-purple-400/70",
      detail: "Total",
    },
    {
      label: "Memory",
      value: `${memoryMB} MB`,
      icon: Cpu,
      accent: "from-emerald-300/70 via-teal-400/60 to-sky-400/60",
      detail: "Arti Resident",
    },
    {
      label: "Circuits",
      value: `${circuitCount}`,
      icon: Gauge,
      accent: "from-violet-400/70 via-purple-400/60 to-fuchsia-400/50",
      detail: "Aktiv",
    },
    {
      label: "Latency",
      value: pingMs !== undefined && pingMs >= 0 ? `${pingMs} ms` : "–",
      icon: Zap,
      accent: "from-amber-400/70 via-orange-400/60 to-rose-400/50",
      detail: "Ping",
    },
  ];
</script>

<!-- Status Card -->
<GlassCard className="rounded-[32px] p-6 shadow-[0_45px_120px_rgba(10,15,45,0.6)] border-white/5 bg-void/80" role="region" aria-label="Status information">
  <div
    class={`pointer-events-none absolute -inset-24 bg-gradient-to-br ${statusInfo.gradient} opacity-60 blur-3xl`}
    class:opacity-30={$reducedMotion}
  ></div>
  <div
    class={`pointer-events-none absolute inset-0 ${statusInfo.aura} opacity-40`}
    class:opacity-20={$reducedMotion}
  ></div>

  <div class="relative z-10 space-y-8">
    <div class="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
      <div class="flex-1 space-y-4">
        <div class="flex items-center gap-3">
          <span class={`h-3 w-3 rounded-full shadow-[0_0_12px_rgba(255,255,255,0.35)] ${statusInfo.indicator}`}></span>
          <div>
            <h3 class="text-lg font-semibold uppercase tracking-[0.22em] text-white">
              {statusInfo.label}
            </h3>
            <p class="text-sm text-slate-200/80">{statusInfo.caption}</p>
          </div>
        </div>

        {#if $torStore.securityWarning}
          <p
            class="rounded-2xl border border-amber-300/40 bg-amber-500/20 px-4 py-3 text-xs text-amber-100 shadow-[0_20px_45px_rgba(161,98,7,0.35)]"
            role="alert"
          >
            {$torStore.securityWarning}
          </p>
        {/if}

        {#if proxyActive}
          <div class="inline-flex items-center gap-2 rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-xs font-medium text-emerald-300">
            <Shield class="h-3 w-3" />
            <span>System Routing (VPN Mode) Active</span>
          </div>
        {/if}

        <div class="flex flex-wrap gap-3">
          {#each highlightMetrics as metric (metric.label)}
            <GlassCard className="px-4 py-3 shadow-[0_25px_55px_rgba(12,18,46,0.55)] bg-white/[0.03] border-white/5">
              <div class={`absolute inset-0 bg-gradient-to-r ${metric.accent} opacity-70 blur-xl`}></div>
              <div class="relative z-10 flex items-center gap-3">
                <svelte:component this={metric.icon} class="h-4 w-4 text-white/90" aria-hidden="true" />
                <div class="flex flex-col leading-tight">
                  <span class="text-[11px] uppercase tracking-[0.28em] text-white/70">
                    {metric.label}
                  </span>
                  <span class="text-sm font-semibold text-white">{metric.value}</span>
                  <span class="text-[11px] text-white/60">{metric.detail}</span>
                </div>
              </div>
            </GlassCard>
          {/each}
        </div>
      </div>

      <GlassCard className="w-full max-w-sm flex-col gap-3 p-4 shadow-[0_35px_65px_rgba(8,12,32,0.6)] bg-slate-900/60">
        <div class="flex items-center justify-between">
          <span class="text-[11px] uppercase tracking-[0.32em] text-white/60">Ping Monitor</span>
          <button
            class="group relative flex h-9 w-9 items-center justify-center overflow-hidden rounded-2xl border border-white/10 bg-white/10 text-white shadow-[0_18px_40px_rgba(37,99,235,0.45)] transition-transform duration-200 ease-out disabled:cursor-not-allowed disabled:opacity-60"
            class:scale-95={isPinging}
            on:click={performPing}
            disabled={isPinging}
            title="Start Ping Test"
            aria-label="Run ping test"
          >
            {#if isPinging}
              <div class="absolute inset-0 bg-gradient-to-br from-sky-400/60 via-indigo-400/50 to-purple-500/50"></div>
              <div class="relative flex items-center justify-center">
                <div
                  class="h-2.5 w-2.5 rounded-full bg-white/90 shadow-[0_0_12px_rgba(255,255,255,0.6)]"
                ></div>
              </div>
            {:else}
              <div class="absolute inset-0 bg-gradient-to-br from-sky-400/40 via-indigo-400/30 to-purple-500/30"></div>
              <Zap class="relative h-4 w-4" aria-hidden="true" />
            {/if}
          </button>
        </div>
        <div class="relative h-20 overflow-hidden rounded-xl border border-white/5 bg-black/20">
          <svg {width} {height} class="absolute inset-0 h-full w-full text-sky-300" aria-label="Ping history chart" role="img">
            {#if pingPath}
              <path d={pingPath} fill="currentColor" fill-opacity="0.25" stroke="currentColor" stroke-width="1.5" />
            {:else}
              <text x="10" y="24" class="fill-white/40 text-[10px]">No data</text>
            {/if}
          </svg>
        </div>
        <p class="text-xs text-white/70">
          Zuletzt gemessen: {pingMs !== undefined && pingMs >= 0 ? `${pingMs} ms` : "keine Daten"}
        </p>
      </GlassCard>
    </div>

    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
      <GlassCard className="px-4 py-3 shadow-[0_22px_55px_rgba(12,18,46,0.45)] bg-white/5 border-white/10">
        <span class="text-[11px] uppercase tracking-[0.28em] text-white/70">Uptime</span>
        <p class="mt-1 text-sm font-semibold text-white">{uptimeLabel}</p>
      </GlassCard>
      <GlassCard className="px-4 py-3 shadow-[0_22px_55px_rgba(12,18,46,0.45)] bg-white/5 border-white/10">
        <span class="text-[11px] uppercase tracking-[0.28em] text-white/70">Throughput</span>
        <p class="mt-1 text-sm font-semibold text-white">{throughputLabel}</p>
      </GlassCard>
      <GlassCard className="px-4 py-3 shadow-[0_22px_55px_rgba(12,18,46,0.45)] bg-white/5 border-white/10">
        <span class="text-[11px] uppercase tracking-[0.28em] text-white/70">Connected Since</span>
        <p class="mt-1 text-sm font-semibold text-white/90 break-words">{connectedSinceLabel}</p>
      </GlassCard>
      <GlassCard className="px-4 py-3 shadow-[0_22px_55px_rgba(12,18,46,0.45)] bg-white/5 border-white/10">
        <span class="text-[11px] uppercase tracking-[0.28em] text-white/70">Oldest Circuit</span>
        <p class="mt-1 text-sm font-semibold text-white">{oldestCircuitLabel}</p>
      </GlassCard>
    </div>

    <GlassCard className="p-4 shadow-[0_30px_65px_rgba(8,12,32,0.55)] bg-slate-900/40">
      <div class="flex items-center justify-between">
        <h4 class="text-[11px] uppercase tracking-[0.28em] text-white/70">Circuit Route</h4>
        {#if routePolicyState}
          <span class={`text-[11px] font-medium ${routePolicyTone}`}>{routePolicyState}</span>
        {/if}
      </div>
      <div class="mt-3 grid gap-3 sm:grid-cols-3">
        {#each routeDisplay as detail (detail.role)}
          <GlassCard
            className="px-4 py-3 shadow-[0_22px_55px_rgba(12,18,46,0.45)] bg-white/5 border-white/10"
            title={`Effective ${detail.role.toLowerCase()}: ${detail.label}`}
          >
            <div class="flex items-center justify-between text-[11px] uppercase tracking-[0.24em] text-white/70">
              <span>{detail.role}</span>
              <span aria-hidden="true">{detail.flag}</span>
            </div>
            <p class="mt-2 text-sm font-semibold text-white">{detail.label}</p>
            {#if detail.status === 'locked'}
              <p class="text-[11px] text-emerald-200/90">Pinned</p>
            {:else if detail.status === 'fallback'}
              <p class="text-[11px] text-amber-200/90">
                Fallback{#if detail.requestedLabel} · {detail.requestedLabel}{/if}
              </p>
            {:else}
              <p class="text-[11px] text-slate-300/80">Automatic</p>
            {/if}
            {#if detail.isFast}
              <p class="text-[10px] text-sky-200/90">Fast-tier relay</p>
            {/if}
          </GlassCard>
        {/each}
      </div>
    </GlassCard>

    <div class="space-y-4">
      <MetricsChart {metrics} />
      <CircuitList show={status === 'CONNECTED'} />
    </div>
  </div>
</GlassCard>

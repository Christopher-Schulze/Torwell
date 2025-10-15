<script lang="ts">
  import { browser } from '$app/environment';
  import { onDestroy, onMount } from 'svelte';
  import {
    ActivitySquare,
    AlertTriangle,
    CheckCircle2,
    Clock3,
    History,
    Power,
    RefreshCcw,
    TimerReset,
  } from 'lucide-svelte';
  import { connectionDiagnosticsStore } from '$lib/stores/connectionDiagnosticsStore';
  import { torStore } from '$lib/stores/torStore';
  import { formatRelativeTime } from '$lib/utils/metrics';
  import type {
    ConnectionDiagnosticsState,
  } from '$lib/stores/connectionDiagnosticsStore';
  import type { ConnectionEvent, ConnectionHealthSummary } from '$lib/types';

  let timeline: ConnectionEvent[] = [];
  let summary: ConnectionHealthSummary | null = null;
  let loading = false;
  let error: string | null = null;
  let lastUpdated: number | null = null;

  const STATUS_CONFIG = {
    CONNECTED: { label: 'Verbunden', tone: 'text-emerald-300', icon: CheckCircle2 },
    CONNECTING: {
      label: 'Verbindung wird aufgebaut',
      tone: 'text-sky-300',
      icon: RefreshCcw,
    },
    RETRYING: { label: 'Neuer Versuch', tone: 'text-amber-300', icon: RefreshCcw },
    ERROR: { label: 'Fehler', tone: 'text-rose-300', icon: AlertTriangle },
    DISCONNECTED: { label: 'Getrennt', tone: 'text-slate-300', icon: Power },
    DISCONNECTING: { label: 'Trenne Verbindung', tone: 'text-slate-300', icon: Power },
    NEW_IDENTITY: { label: 'Neue Identität', tone: 'text-indigo-300', icon: ActivitySquare },
    NEW_CIRCUIT: { label: 'Neue Schaltung', tone: 'text-violet-300', icon: ActivitySquare },
  } as const;

  function resolveStatus(event: ConnectionEvent) {
    return STATUS_CONFIG[event.status as keyof typeof STATUS_CONFIG] ?? {
      label: event.status,
      tone: 'text-slate-300',
      icon: History,
    };
  }

  function formatDuration(seconds: number | null | undefined): string {
    if (!seconds || seconds < 0) return '–';
    const total = Math.floor(seconds);
    const days = Math.floor(total / 86_400);
    const hours = Math.floor((total % 86_400) / 3_600);
    const minutes = Math.floor((total % 3_600) / 60);
    const secs = total % 60;
    if (days) return `${days}d ${hours}h`;
    if (hours) return `${hours}h ${minutes}m`;
    if (minutes) return `${minutes}m ${secs}s`;
    return `${secs}s`;
  }

  function formatAvailability(value: number | null | undefined): string {
    if (value == null || Number.isNaN(value)) return '–';
    return `${value.toFixed(1)} %`;
  }

  function formatUpdated(timestamp: number | null | undefined): string {
    if (!timestamp) return 'Noch nicht aktualisiert';
    const date = new Date(timestamp);
    return `Aktualisiert ${formatRelativeTime(date.getTime())}`;
  }

  function formatDetail(value: string | null | undefined): string | null {
    if (!value) return null;
    return value;
  }

  function formatBytes(bytes: number | null | undefined): string | null {
    if (bytes == null) return null;
    if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(1)} MB`;
    if (bytes >= 1_000) return `${(bytes / 1_000).toFixed(1)} KB`;
    if (bytes > 0) return `${bytes} B`;
    return null;
  }

  function formatCircuits(count: number | null | undefined): string | null {
    if (count == null) return null;
    return `${count} Kreise`;
  }

  function formatLatency(latency: number | null | undefined): string | null {
    if (latency == null) return null;
    if (latency <= 0) return null;
    return `${latency} ms`; 
  }

  function refresh() {
    connectionDiagnosticsStore.refresh();
  }

  onMount(() => {
    if (!browser) return;
    const unsubscribeStore = connectionDiagnosticsStore.subscribe(
      (state: ConnectionDiagnosticsState) => {
        timeline = state.timeline ?? [];
        summary = state.summary;
        loading = state.loading;
        error = state.error;
        lastUpdated = state.lastUpdated;
      }
    );

    const unsubscribeTor = torStore.subscribe(($tor) => {
      if ($tor.status === 'CONNECTED') {
        connectionDiagnosticsStore.start();
      } else {
        connectionDiagnosticsStore.reset();
      }
    });

    return () => {
      unsubscribeStore();
      unsubscribeTor();
      connectionDiagnosticsStore.stop();
    };
  });

  onDestroy(() => {
    if (browser) {
      connectionDiagnosticsStore.stop();
    }
  });

  $: availabilityLabel = formatAvailability(summary?.availabilityPercent ?? null);
  $: uptimeLabel = formatDuration(summary?.currentUptimeSeconds ?? null);
  $: longestUptimeLabel = formatDuration(summary?.longestUptimeSeconds ?? null);
  $: retryLabel = summary ? summary.retryAttemptsLastHour : 0;
</script>

<div class="glass-md rounded-xl p-6 flex flex-col gap-6" role="region" aria-label="Verbindungsdiagnose">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <div>
      <h2 class="text-lg font-semibold text-slate-100">Verbindungsdiagnose</h2>
      <p class="text-sm text-slate-400">Langzeitübersicht über Stabilität und Ereignisse der Tor-Verbindung.</p>
    </div>
    <div class="flex items-center gap-3 text-sm text-slate-400">
      <span>{formatUpdated(lastUpdated)}</span>
      <button
        class="inline-flex items-center gap-2 rounded-lg border border-slate-600/50 px-3 py-1.5 text-slate-200 transition hover:border-slate-400 hover:text-white"
        type="button"
        on:click={refresh}
        aria-label="Diagnosedaten aktualisieren"
      >
        <RefreshCcw size={16} />
        Aktualisieren
      </button>
    </div>
  </div>

  {#if error}
    <div class="rounded-lg border border-rose-500/40 bg-rose-500/10 px-4 py-3 text-sm text-rose-200">
      <div class="font-medium">Diagnosedaten konnten nicht geladen werden.</div>
      <div>{error}</div>
    </div>
  {/if}

  <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
    <div class="rounded-lg border border-slate-700/60 bg-slate-900/40 p-4">
      <div class="flex items-center justify-between text-sm text-slate-400">
        <span>Verfügbarkeit</span>
        <Clock3 size={18} class="text-slate-500" />
      </div>
      <div class="mt-2 text-2xl font-semibold text-slate-100">{availabilityLabel}</div>
      <p class="mt-1 text-xs text-slate-400">
        Basierend auf den letzten {summary?.totalEvents ?? 0} Statuswechseln.
      </p>
    </div>

    <div class="rounded-lg border border-slate-700/60 bg-slate-900/40 p-4">
      <div class="flex items-center justify-between text-sm text-slate-400">
        <span>Aktuelle Laufzeit</span>
        <TimerReset size={18} class="text-slate-500" />
      </div>
      <div class="mt-2 text-2xl font-semibold text-slate-100">{uptimeLabel}</div>
      <p class="mt-1 text-xs text-slate-400">
        Längste Session: {longestUptimeLabel}
      </p>
    </div>

    <div class="rounded-lg border border-slate-700/60 bg-slate-900/40 p-4">
      <div class="flex items-center justify-between text-sm text-slate-400">
        <span>Retries (1h)</span>
        <RefreshCcw size={18} class="text-slate-500" />
      </div>
      <div class="mt-2 text-2xl font-semibold text-slate-100">{retryLabel}</div>
      <p class="mt-1 text-xs text-slate-400">
        Letzter Fehler: {summary?.lastErrorAt ? new Date(summary.lastErrorAt).toLocaleString() : '–'}
      </p>
    </div>

    <div class="rounded-lg border border-slate-700/60 bg-slate-900/40 p-4">
      <div class="flex items-center justify-between text-sm text-slate-400">
        <span>Letztes Ereignis</span>
        <History size={18} class="text-slate-500" />
      </div>
      <div class="mt-2 text-2xl font-semibold text-slate-100">
        {#if summary?.lastEvent}
          {resolveStatus(summary.lastEvent).label}
        {:else}
          –
        {/if}
      </div>
      <p class="mt-1 text-xs text-slate-400">
        {summary?.lastEvent ? new Date(summary.lastEvent.timestamp).toLocaleString() : 'Noch keine Daten'}
      </p>
    </div>
  </div>

  <div>
    <h3 class="text-sm font-semibold uppercase tracking-wide text-slate-300">Ereignisprotokoll</h3>
    {#if loading}
      <div class="mt-4 text-sm text-slate-400">Diagnosedaten werden geladen…</div>
    {:else if !timeline.length}
      <div class="mt-4 text-sm text-slate-400">Noch keine Verbindungsereignisse aufgezeichnet.</div>
    {:else}
      <ul class="mt-4 space-y-4">
        {#each [...timeline].reverse() as event (event.timestamp)}
          {@const config = resolveStatus(event)}
          <li class="flex gap-3 rounded-lg border border-slate-800/70 bg-slate-950/40 p-4">
            <div class={`flex h-10 w-10 items-center justify-center rounded-full border border-slate-700/50 ${config.tone}`}>
              <svelte:component this={config.icon} size={18} />
            </div>
            <div class="flex-1">
              <div class="flex flex-wrap items-center justify-between gap-2">
                <div class="text-sm font-medium text-slate-100">{config.label}</div>
                <div class="text-xs text-slate-400">
                  {#if browser}
                    {#if event.timestamp}
                      {formatRelativeTime(new Date(event.timestamp).getTime())}
                    {:else}
                      –
                    {/if}
                  {:else}
                    –
                  {/if}
                </div>
              </div>
              {#if event.message}
                <div class="mt-1 text-sm text-slate-200">{event.message}</div>
              {/if}
              <div class="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-slate-400">
                {#if formatDetail(event.detail)}
                  <span>{formatDetail(event.detail)}</span>
                {/if}
                {#if typeof event.retryCount === 'number'}
                  <span>Versuch #{event.retryCount}</span>
                {/if}
                {#if formatLatency(event.latencyMs ?? null)}
                  <span>Latenz: {formatLatency(event.latencyMs ?? null)}</span>
                {/if}
                {#if formatBytes(event.memoryBytes ?? null)}
                  <span>Speicher: {formatBytes(event.memoryBytes ?? null)}</span>
                {/if}
                {#if formatCircuits(event.circuitCount ?? null)}
                  <span>{formatCircuits(event.circuitCount ?? null)}</span>
                {/if}
                <span class="text-slate-500">
                  {new Date(event.timestamp).toLocaleString()}
                </span>
              </div>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

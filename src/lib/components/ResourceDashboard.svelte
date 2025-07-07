<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/tauri';
  import MetricsChart from './MetricsChart.svelte';
  import type { MetricPoint } from '$lib/stores/torStore';

  let metrics: MetricPoint[] = [];
  const MAX_POINTS = 30;

  const width = 120;
  const height = 40;

  function buildPath(data: MetricPoint[], field: keyof MetricPoint): string {
    if (data.length === 0) return "";
    const maxVal = Math.max(...data.map((d) => d[field] as number), 1);
    const step = width / Math.max(data.length - 1, 1);
    let d = `M0 ${height}`;
    data.forEach((pt, idx) => {
      const x = idx * step;
      const y = height - ((pt[field] as number) / maxVal) * height;
      d += ` L${x} ${y}`;
    });
    d += ` L${width} ${height} Z`;
    return d;
  }

  const MAX_MEMORY_MB = 1024;
  const MAX_CIRCUITS = 20;

  $: cpuPath = buildPath(metrics, 'cpuPercent');
  $: networkPath = buildPath(metrics, 'networkBytes');
  $: avgPath = buildPath(metrics, 'avgCreateMs');
  $: failPath = buildPath(metrics, 'failedAttempts');
  $: pingPath = buildPath(metrics, 'latencyMs');

  $: latest =
    metrics[metrics.length - 1] ?? {
      memoryMB: 0,
      circuitCount: 0,
      latencyMs: 0,
      oldestAge: 0,
      avgCreateMs: 0,
      failedAttempts: 0,
      cpuPercent: 0,
      networkBytes: 0,
      networkTotal: 0,
      time: 0,
    };

  onMount(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const data = await invoke<MetricPoint[]>('load_metrics');
        metrics = data.slice(-MAX_POINTS);
      } catch (e) {
        console.error('Failed to load metrics', e);
      }

      unlisten = await listen<any>('metrics-update', (event) => {
        const point: MetricPoint = {
          time: Date.now(),
          memoryMB: Math.round(event.payload.memory_bytes / 1_000_000),
          circuitCount: event.payload.circuit_count,
          latencyMs: event.payload.latency_ms,
          oldestAge: event.payload.oldest_age ?? 0,
          avgCreateMs: event.payload.avg_create_ms ?? 0,
          failedAttempts: event.payload.failed_attempts ?? 0,
          cpuPercent: event.payload.cpu_percent ?? 0,
          networkBytes: event.payload.network_bytes ?? 0,
          networkTotal: event.payload.total_network_bytes ?? 0,
        };
        metrics = [...metrics, point].slice(-MAX_POINTS);
      });
    })();

    return () => {
      if (unlisten) unlisten();
    };
  });
</script>

<div class="glass-md rounded-xl p-4 space-y-4" role="region" aria-label="Resource dashboard">
  <div class="flex gap-4">
    <div class="flex-1">
      <p class="text-sm text-white">Memory: {latest.memoryMB} MB</p>
      {#if latest.memoryMB > MAX_MEMORY_MB}
        <p class="text-sm text-red-400" role="alert">Memory usage high</p>
      {/if}
    </div>
    <div class="flex-1">
      <p class="text-sm text-white">Circuits: {latest.circuitCount}</p>
      {#if latest.circuitCount > MAX_CIRCUITS}
        <p class="text-sm text-red-400" role="alert">Circuit count high</p>
      {/if}
    </div>
    <div class="flex-1">
      <p class="text-sm text-white">Avg build: {latest.avgCreateMs} ms</p>
    </div>
    <div class="flex-1">
      <p class="text-sm text-white">Failures: {latest.failedAttempts}</p>
    </div>
  </div>
  <div class="flex gap-4">
    <div class="flex-1">
      <p class="text-sm text-white">CPU: {latest.cpuPercent.toFixed(1)} %</p>
      <svg {width} {height} class="text-yellow-400" aria-label="CPU usage chart" role="img">
        {#if cpuPath}
          <path d={cpuPath} fill="currentColor" fill-opacity="0.3" stroke="currentColor" stroke-width="1" />
        {/if}
      </svg>
    </div>
    <div class="flex-1">
      <p class="text-sm text-white">Network: {latest.networkBytes} B/s</p>
      <svg {width} {height} class="text-purple-400" aria-label="Network usage chart" role="img">
        {#if networkPath}
          <path d={networkPath} fill="currentColor" fill-opacity="0.3" stroke="currentColor" stroke-width="1" />
        {/if}
      </svg>
    </div>
    <div class="flex-1">
      <p class="text-sm text-white">Ping: {latest.latencyMs} ms</p>
      <svg {width} {height} class="text-blue-400" aria-label="Ping chart" role="img">
        {#if pingPath}
          <path d={pingPath} fill="currentColor" fill-opacity="0.3" stroke="currentColor" stroke-width="1" />
        {/if}
      </svg>
    </div>
    <div class="flex-1">
      <p class="text-sm text-white">Avg build: {latest.avgCreateMs} ms</p>
      <svg {width} {height} class="text-purple-300" aria-label="Average build time chart" role="img">
        {#if avgPath}
          <path d={avgPath} fill="currentColor" fill-opacity="0.3" stroke="currentColor" stroke-width="1" />
        {/if}
      </svg>
    </div>
    <div class="flex-1">
      <p class="text-sm text-white">Failures: {latest.failedAttempts}</p>
      <svg {width} {height} class="text-red-400" aria-label="Failed attempts chart" role="img">
        {#if failPath}
          <path d={failPath} fill="currentColor" fill-opacity="0.3" stroke="currentColor" stroke-width="1" />
        {/if}
      </svg>
    </div>
  </div>
  <MetricsChart {metrics} />
</div>

<style>
  .glass-md {
    @apply bg-white/20 backdrop-blur-xl;
  }
</style>

<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import MetricsChart from './MetricsChart.svelte';
  import type { MetricPoint } from '$lib/stores/torStore';

  const CHART_WIDTH = 120;
  const CHART_HEIGHT = 40;

  function buildPath(data: MetricPoint[], field: keyof MetricPoint): string {
    if (data.length === 0) return '';
    const maxVal = Math.max(...data.map((d) => d[field] as number), 1);
    const step = CHART_WIDTH / Math.max(data.length - 1, 1);
    let d = `M0 ${CHART_HEIGHT}`;
    data.forEach((pt, idx) => {
      const x = idx * step;
      const y = CHART_HEIGHT - ((pt[field] as number) / maxVal) * CHART_HEIGHT;
      d += ` L${x} ${y}`;
    });
    d += ` L${CHART_WIDTH} ${CHART_HEIGHT} Z`;
    return d;
  }

  $: cpuPath = buildPath(metrics, 'cpuPercent');
  $: networkPath = buildPath(metrics, 'networkBytes');

  let metrics: MetricPoint[] = [];
  const MAX_POINTS = 30;

  const MAX_MEMORY_MB = 1024;
  const MAX_CIRCUITS = 20;

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
      time: 0,
    };

  onMount(() => {
    const unlisten = listen<any>('metrics-update', (event) => {
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
      };
      metrics = [...metrics, point].slice(-MAX_POINTS);
    });

    return () => {
      unlisten.then((fn) => fn());
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
    </div>
    <div class="flex-1">
      <p class="text-sm text-white">Network: {latest.networkBytes} B/s</p>
    </div>
  </div>
  <div class="flex gap-2 items-end">
    <MetricsChart {metrics} />
    <svg
      width={CHART_WIDTH}
      height={CHART_HEIGHT}
      class="text-purple-400"
      role="img"
      aria-label="CPU usage chart"
    >
      {#if cpuPath}
        <path
          d={cpuPath}
          fill="currentColor"
          fill-opacity="0.3"
          stroke="currentColor"
          stroke-width="1"
        />
      {/if}
    </svg>
    <svg
      width={CHART_WIDTH}
      height={CHART_HEIGHT}
      class="text-cyan-400"
      role="img"
      aria-label="Network usage chart"
    >
      {#if networkPath}
        <path
          d={networkPath}
          fill="currentColor"
          fill-opacity="0.3"
          stroke="currentColor"
          stroke-width="1"
        />
      {/if}
    </svg>
  </div>
</div>

<style>
  .glass-md {
    @apply bg-white/20 backdrop-blur-xl;
  }
</style>

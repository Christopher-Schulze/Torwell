<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import MetricsChart from './MetricsChart.svelte';
  import type { MetricPoint } from '$lib/stores/torStore';

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
  <MetricsChart {metrics} />
</div>

<style>
  .glass-md {
    @apply bg-white/20 backdrop-blur-xl;
  }
</style>

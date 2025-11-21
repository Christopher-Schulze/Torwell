<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '$lib/api';
  import type { MetricPoint } from '$lib/stores/torStore';
  import GlassCard from './GlassCard.svelte';

  export let className = '';

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

  $: cpuPath = buildPath(metrics, 'cpuPercent');
  $: totalPath = buildPath(metrics, 'networkTotal');

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
      complete: true,
    };

  onMount(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const data = await invoke<MetricPoint[]>('load_metrics', { limit: MAX_POINTS });
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
          complete: event.payload.complete ?? true,
        };
        metrics = [...metrics, point].slice(-MAX_POINTS);
      });
    })();

    return () => {
      if (unlisten) unlisten();
    };
  });
</script>

<GlassCard className={"p-4 space-y-4 " + className}>
  <div class="flex flex-col gap-4 md:flex-row">
    <div class="flex-1">
      <div class="flex items-center justify-between mb-1">
        <p class="text-xs uppercase tracking-widest text-slate-400">CPU Load</p>
        <p class="font-mono text-neon-green text-sm">{latest.cpuPercent.toFixed(1)}%</p>
      </div>
      <svg {width} {height} class="text-neon-green w-full h-12" aria-label="CPU usage chart" role="img" preserveAspectRatio="none">
        {#if cpuPath}
          <path d={cpuPath} fill="currentColor" fill-opacity="0.2" stroke="currentColor" stroke-width="2" vector-effect="non-scaling-stroke"/>
        {/if}
      </svg>
    </div>
    <div class="flex-1">
      <div class="flex items-center justify-between mb-1">
        <p class="text-xs uppercase tracking-widest text-slate-400">Total Traffic</p>
        <p class="font-mono text-neon-purple text-sm">{(latest.networkTotal / 1_000_000).toFixed(2)} MB</p>
      </div>
      <svg {width} {height} class="text-neon-purple w-full h-12" aria-label="Total traffic chart" role="img" preserveAspectRatio="none">
        {#if totalPath}
          <path d={totalPath} fill="currentColor" fill-opacity="0.2" stroke="currentColor" stroke-width="2" vector-effect="non-scaling-stroke"/>
        {/if}
      </svg>
    </div>
  </div>
</GlassCard>

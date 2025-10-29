<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '$lib/api';
  import MetricsChart from './MetricsChart.svelte';
  import MetricTrendIndicator from './MetricTrendIndicator.svelte';
  import MetricHistoryTable from './MetricHistoryTable.svelte';
  import HealthChecklist from './HealthChecklist.svelte';
  import type { MetricPoint } from '$lib/stores/torStore';
  import {
    summarizeMetric,
    evaluateTorHealth,
    getRecentWindow,
    formatRelativeTime,
    resolveSeverity,
    humanizeBytes,
    toRounded,
    computeRollingAverage,
    type MetricSummary,
    type HealthAssessment,
  } from '$lib/utils/metrics';

  const MAX_POINTS = 240;
  const INITIAL_WINDOW = 30;

  type WindowOption = { label: string; value: number };

  let metrics: MetricPoint[] = [];
  let selectedWindow: number = INITIAL_WINDOW;
  let assessments: HealthAssessment[] = [];

  const windowOptions: WindowOption[] = [
    { label: 'Letzte 10 Samples', value: 10 },
    { label: 'Letzte 20 Samples', value: 20 },
    { label: 'Letzte 30 Samples', value: 30 },
    { label: 'Letzte 60 Samples', value: 60 },
  ];

  let unlisten: (() => void) | undefined;

  function handleWindowChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const parsed = Number.parseInt(target.value, 10);
    if (!Number.isNaN(parsed)) {
      selectedWindow = parsed;
    }
  }

  function last<T>(items: T[]): T | undefined {
    if (items.length === 0) return undefined;
    return items[items.length - 1];
  }

  onMount(() => {
    (async () => {
      try {
        const data = await invoke<MetricPoint[]>('load_metrics', { limit: MAX_POINTS });
        metrics = data.map((point) => ({ ...point, complete: true })).slice(-MAX_POINTS);
      } catch (error) {
        console.error('Failed to load metrics', error);
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

  $: recentMetrics = getRecentWindow(metrics, selectedWindow);
  $: lastUpdated = last(recentMetrics)?.time ?? null;
  $: assessments = evaluateTorHealth(recentMetrics);
  $: assessmentLookup = assessments.reduce<Record<string, HealthAssessment>>((map, item) => {
    map[item.title] = item;
    return map;
  }, {});

  function toSummary(field: Parameters<typeof summarizeMetric>[1]): MetricSummary | null {
    return summarizeMetric(recentMetrics, field, { window: selectedWindow });
  }

  $: memorySummary = toSummary('memoryMB');
  $: circuitSummary = toSummary('circuitCount');
  $: latencySummary = toSummary('latencyMs');
  $: throughputSummary = toSummary('networkBytes');
  $: failureSummary = toSummary('failedAttempts');
  $: cpuSummary = toSummary('cpuPercent');
  $: buildSummary = toSummary('avgCreateMs');

  $: latencySeries = recentMetrics
    .map((point) => point.latencyMs)
    .filter((value): value is number => typeof value === "number");
  $: rollingLatency = computeRollingAverage(
    latencySeries,
    Math.min(5, latencySeries.length || 1)
  );
  $: rollingThroughput = computeRollingAverage(
    recentMetrics.map((point) => point.networkBytes),
    Math.min(5, recentMetrics.length || 1)
  );
  $: rollingFailures = computeRollingAverage(
    recentMetrics.map((point) => point.failedAttempts),
    Math.min(5, recentMetrics.length || 1)
  );

  $: latestPoint = last(recentMetrics);
  $: totalTransferred = latestPoint ? humanizeBytes(latestPoint.networkTotal) : '0 B';
  $: averageThroughput = throughputSummary
    ? humanizeBytes(Math.max(throughputSummary.average, 0))
    : '0 B';
  $: uptimeMinutes = metrics.length
    ? Math.max(
        0,
        Math.round((metrics[metrics.length - 1].time - metrics[0].time) / 60_000)
      )
    : 0;
</script>

<div class="glass-md rounded-xl p-6 space-y-6" role="region" aria-label="Resource dashboard">
  <header class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
    <div>
      <h2 class="text-lg font-semibold text-white">Ressourcenübersicht</h2>
      <p class="text-sm text-slate-300">
        Letzte Aktualisierung: <span class="font-medium text-slate-100">{formatRelativeTime(lastUpdated)}</span>
      </p>
      <p class="text-xs text-slate-400">Gesamte Aufzeichnung: {uptimeMinutes} Minuten</p>
    </div>
    <label class="flex items-center gap-3 text-sm text-slate-200">
      <span>Zeitraum</span>
      <select
        class="rounded-lg border border-slate-700 bg-slate-900/80 px-3 py-2 text-sm text-white"
        on:change={handleWindowChange}
      >
        {#each windowOptions as option}
          <option value={option.value} selected={option.value === selectedWindow}>{option.label}</option>
        {/each}
      </select>
    </label>
  </header>

  <section class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
    {#if memorySummary}
      <MetricTrendIndicator
        label="Speicher"
        value={memorySummary.current}
        average={toRounded(memorySummary.average, 1)}
        min={memorySummary.min}
        max={memorySummary.max}
        unit=" MB"
        decimals={1}
        trend={memorySummary.trend}
        severity={resolveSeverity(memorySummary.current, {
          warning: 900,
          critical: 1100,
          direction: 'higher-is-worse',
        })}
        description={`Fenster Ø: ${toRounded(memorySummary.average, 1)} MB`}
        hint={assessmentLookup['Speicherauslastung']?.hint}
        trendPoints={memorySummary.values}
      />
    {/if}

    {#if cpuSummary}
      <MetricTrendIndicator
        label="CPU-Auslastung"
        value={toRounded(cpuSummary.current, 1)}
        average={toRounded(cpuSummary.average, 1)}
        min={toRounded(cpuSummary.min, 1)}
        max={toRounded(cpuSummary.max, 1)}
        unit=" %"
        decimals={1}
        trend={cpuSummary.trend}
        severity={resolveSeverity(cpuSummary.current, {
          warning: 70,
          critical: 90,
          direction: 'higher-is-worse',
        })}
        description={`Durchschnittliche CPU ${toRounded(cpuSummary.average, 1)} %`}
        trendPoints={cpuSummary.values}
      />
    {/if}

    {#if circuitSummary}
      <MetricTrendIndicator
        label="Circuit-Kapazität"
        value={circuitSummary.current}
        average={toRounded(circuitSummary.average, 1)}
        min={circuitSummary.min}
        max={circuitSummary.max}
        unit=""
        decimals={1}
        trend={circuitSummary.trend}
        severity={resolveSeverity(circuitSummary.current, {
          warning: 12,
          critical: 16,
          direction: 'higher-is-worse',
        })}
        description={`Durchschnittlich ${toRounded(circuitSummary.average, 1)} parallele Circuits`}
        hint={assessmentLookup['Circuit-Pool']?.hint}
        trendPoints={circuitSummary.values}
      />
    {/if}

    {#if latencySummary}
      <MetricTrendIndicator
        label="Latenz"
        value={latencySummary.current}
        average={toRounded(latencySummary.average, 1)}
        min={latencySummary.min}
        max={latencySummary.max}
        unit=" ms"
        decimals={1}
        trend={latencySummary.trend}
        severity={resolveSeverity(latencySummary.current, {
          warning: 300,
          critical: 600,
          direction: 'higher-is-worse',
        })}
        description={`Rollender Ø: ${toRounded(last(rollingLatency) ?? latencySummary.current, 1)} ms`}
        hint={assessmentLookup['Latenz']?.hint}
        trendPoints={latencySummary.values}
      />
    {/if}

    {#if throughputSummary}
      <MetricTrendIndicator
        label="Durchsatz"
        value={throughputSummary.current}
        average={throughputSummary.average}
        min={throughputSummary.min}
        max={throughputSummary.max}
        unit=""
        decimals={1}
        trend={throughputSummary.trend}
        severity={resolveSeverity(throughputSummary.current, {
          warning: 20_000,
          critical: 5_000,
          direction: 'lower-is-worse',
        })}
        description={`Durchschnitt: ${averageThroughput}`}
        hint={assessmentLookup['Durchsatz']?.hint}
        trendPoints={throughputSummary.values}
        formatter={(value) => humanizeBytes(Math.max(value, 0))}
      />
    {/if}

    {#if failureSummary}
      <MetricTrendIndicator
        label="Fehlversuche"
        value={failureSummary.current}
        average={toRounded(failureSummary.average, 1)}
        min={failureSummary.min}
        max={failureSummary.max}
        unit=""
        decimals={1}
        trend={failureSummary.trend}
        severity={resolveSeverity(failureSummary.current, {
          warning: 2,
          critical: 5,
          direction: 'higher-is-worse',
        })}
        description={`Rollender Ø: ${toRounded(last(rollingFailures) ?? 0, 1)} Fehler`}
        hint={assessmentLookup['Fehlgeschlagene Versuche']?.hint}
        trendPoints={failureSummary.values}
      />
    {/if}

    {#if buildSummary}
      <MetricTrendIndicator
        label="Aufbauzeit"
        value={buildSummary.current}
        average={toRounded(buildSummary.average, 1)}
        min={buildSummary.min}
        max={buildSummary.max}
        unit=" ms"
        decimals={1}
        trend={buildSummary.trend}
        severity={resolveSeverity(buildSummary.current, {
          warning: 1_500,
          critical: 3_000,
          direction: 'higher-is-worse',
        })}
        description={`Durchschnittliche Aufbauzeit ${toRounded(buildSummary.average, 1)} ms`}
        trendPoints={buildSummary.values}
      />
    {/if}
  </section>

  <section class="grid gap-4 lg:grid-cols-3">
    <div class="rounded-xl border border-slate-700/60 bg-slate-900/40 p-4 lg:col-span-2">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-semibold text-white">Sparklines</h3>
        <span class="text-xs text-slate-300">Gesamtdaten: {totalTransferred}</span>
      </div>
      <p class="mt-1 text-xs text-slate-400">Aggregierte Visualisierung der Rohdaten.</p>
      <div class="mt-4 grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        <div>
          <h4 class="text-xs uppercase text-slate-300">Speicher & Circuits</h4>
          <MetricsChart metrics={recentMetrics} />
        </div>
        <div>
          <h4 class="text-xs uppercase text-slate-300">Latenz (rollender Ø)</h4>
          <svg width="120" height="40" class="text-blue-400" role="img" aria-label="Rolling latency">
            {#if rollingLatency.length > 1}
              {#key rollingLatency.length}
                <path
                  d={`M0 40 ${rollingLatency
                    .map((value, index) => {
                      const maxValue = Math.max(...rollingLatency);
                      const minValue = Math.min(...rollingLatency);
                      const span = maxValue - minValue || 1;
                      const step = 120 / Math.max(rollingLatency.length - 1, 1);
                      const x = index * step;
                      const y = 40 - ((value - minValue) / span) * 40;
                      return `L${x} ${y}`;
                    })
                    .join(' ')}`}
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.5"
                />
              {/key}
            {/if}
          </svg>
        </div>
        <div>
          <h4 class="text-xs uppercase text-slate-300">Durchsatz (rollender Ø)</h4>
          <svg width="120" height="40" class="text-purple-300" role="img" aria-label="Rolling throughput">
            {#if rollingThroughput.length > 1}
              {#key rollingThroughput.length}
                <path
                  d={`M0 40 ${rollingThroughput
                    .map((value, index) => {
                      const maxValue = Math.max(...rollingThroughput);
                      const minValue = Math.min(...rollingThroughput);
                      const span = maxValue - minValue || 1;
                      const step = 120 / Math.max(rollingThroughput.length - 1, 1);
                      const x = index * step;
                      const y = 40 - ((value - minValue) / span) * 40;
                      return `L${x} ${y}`;
                    })
                    .join(' ')}`}
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.5"
                />
              {/key}
            {/if}
          </svg>
        </div>
      </div>
    </div>
    <div class="rounded-xl border border-slate-700/60 bg-slate-900/40 p-4">
      <h3 class="text-sm font-semibold text-white">Gesundheits-Check</h3>
      <p class="text-xs text-slate-300">Automatisch erstellte Empfehlungen.</p>
      <div class="mt-3">
        <HealthChecklist {assessments} />
      </div>
    </div>
  </section>

  <MetricHistoryTable metrics={recentMetrics} maxRows={selectedWindow} />
</div>

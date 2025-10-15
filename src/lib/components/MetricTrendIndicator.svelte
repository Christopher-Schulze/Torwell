<script lang="ts">
  import type { Severity, TrendDetails } from '$lib/utils/metrics';

  export let label: string;
  export let value: number = 0;
  export let unit: string = '';
  export let trend: TrendDetails = {
    direction: 'flat',
    change: 0,
    percent: 0,
    previous: 0,
    current: 0,
  };
  export let average: number | null = null;
  export let min: number | null = null;
  export let max: number | null = null;
  export let decimals = 1;
  export let severity: Severity = 'good';
  export let hint: string | null = null;
  export let description: string | null = null;
  export let trendPoints: number[] = [];
  export let formatter: ((value: number) => string) | null = null;

  const width = 120;
  const height = 36;

  const severityClasses: Record<Severity, string> = {
    good: 'border-emerald-400/50 bg-emerald-500/5',
    warning: 'border-amber-400/50 bg-amber-500/10',
    critical: 'border-rose-400/70 bg-rose-500/10',
  };

  const trendClasses: Record<TrendDetails['direction'], string> = {
    up: 'text-emerald-300',
    down: 'text-rose-300',
    flat: 'text-slate-300',
  };

  const trendGlyph: Record<TrendDetails['direction'], string> = {
    up: '▲',
    down: '▼',
    flat: '◆',
  };

  function format(valueToFormat: number): string {
    if (formatter) {
      return formatter(valueToFormat);
    }
    return valueToFormat.toLocaleString(undefined, {
      minimumFractionDigits: valueToFormat % 1 === 0 ? 0 : decimals,
      maximumFractionDigits: decimals,
    });
  }

  function buildPath(values: number[]): string {
    if (!values.length) return '';
    if (values.every((point) => point === values[0])) {
      return `M0 ${height / 2} L${width} ${height / 2}`;
    }
    const maxValue = Math.max(...values);
    const minValue = Math.min(...values);
    const span = maxValue - minValue || 1;
    const step = width / Math.max(values.length - 1, 1);
    const points = values.map((val, index) => {
      const normalised = (val - minValue) / span;
      const x = index * step;
      const y = height - normalised * height;
      return `${index === 0 ? 'M' : 'L'}${x} ${y}`;
    });
    return `${points.join(' ')}`;
  }

  $: sparklinePath = buildPath(trendPoints);
</script>

<div
  class={`rounded-xl border px-4 py-3 transition-colors duration-300 ${severityClasses[severity]} tw-surface`}
  role="group"
  aria-label={`${label} Kennzahl`}
>
  <div class="flex items-start justify-between gap-3">
    <div class="flex-1 space-y-1">
      <p class="text-xs uppercase tracking-wide text-slate-300">{label}</p>
      <p class="text-2xl font-semibold text-white">
        {format(value)}{unit}
      </p>
      {#if average !== null}
        <p class="text-xs text-slate-300">
          Mittelwert: <span class="text-slate-100">{format(average)}{unit}</span>
          {#if min !== null && max !== null}
            · min {format(min)}{unit} · max {format(max)}{unit}
          {/if}
        </p>
      {/if}
    </div>
    <div class="flex flex-col items-end justify-between">
      <span class={`text-sm font-medium ${trendClasses[trend.direction]}`}>
        {trendGlyph[trend.direction]} {trend.change > 0 ? '+' : ''}{trend.change.toFixed(decimals)}{unit}
      </span>
      {#if sparklinePath}
        <svg {width} {height} class="mt-2 text-slate-400" role="presentation">
          <path d={sparklinePath} fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      {/if}
    </div>
  </div>
  {#if description}
    <p class="mt-2 text-sm text-slate-200">{description}</p>
  {/if}
  {#if hint}
    <p class="mt-1 text-xs text-slate-300">{hint}</p>
  {/if}
</div>

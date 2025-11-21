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

  // Cyber-Stealth Palette Mapping
  const severityClasses: Record<Severity, string> = {
    good: 'border-neon-green/30 bg-neon-green/5 shadow-[0_0_10px_-5px_rgba(0,255,65,0.2)]',
    warning: 'border-neon-purple/30 bg-neon-purple/5 shadow-[0_0_10px_-5px_rgba(188,19,254,0.2)]',
    critical: 'border-red-500/30 bg-red-500/5 shadow-[0_0_10px_-5px_rgba(239,68,68,0.2)]',
  };

  const trendClasses: Record<TrendDetails['direction'], string> = {
    up: 'text-neon-green',
    down: 'text-red-400', // Down is usually bad, or good depending on metric, but let's stick to standard semantic for now. If direction is bad, it should be red.
    flat: 'text-slate-400',
  };

  // Override trend colors based on context if needed, but for now rely on severity passed in or simple logic
  // Actually, for 'good' trend, let's use neon-green, for bad use red/purple.
  // The component receives 'severity' which is the main indicator.

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
  class={`rounded-xl border px-4 py-3 transition-colors duration-300 ${severityClasses[severity]} backdrop-blur-sm`}
  role="group"
  aria-label={`${label} Kennzahl`}
>
  <div class="flex items-start justify-between gap-3">
    <div class="flex-1 space-y-1">
      <p class="text-[10px] uppercase tracking-widest text-slate-400 font-mono">{label}</p>
      <p class="text-2xl font-bold text-white font-mono tracking-tight shadow-black drop-shadow-md">
        {format(value)}<span class="text-sm text-slate-400 ml-1">{unit}</span>
      </p>
      {#if average !== null}
        <p class="text-xs text-slate-400 font-mono">
          AVG: <span class="text-slate-200">{format(average)}</span>
          {#if min !== null && max !== null}
            <span class="opacity-50 mx-1">|</span> {format(min)} - {format(max)}
          {/if}
        </p>
      {/if}
    </div>
    <div class="flex flex-col items-end justify-between">
      <span class={`text-sm font-bold font-mono ${trendClasses[trend.direction]}`}>
        {trendGlyph[trend.direction]} {trend.change > 0 ? '+' : ''}{trend.change.toFixed(decimals)}
      </span>
      {#if sparklinePath}
        <svg {width} {height} class={`mt-2 ${severity === 'good' ? 'text-neon-green' : severity === 'warning' ? 'text-neon-purple' : 'text-red-500'}`} role="presentation">
          <path d={sparklinePath} fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" vector-effect="non-scaling-stroke" />
        </svg>
      {/if}
    </div>
  </div>
  {#if description}
    <p class="mt-2 text-xs text-slate-300 border-t border-white/5 pt-2">{description}</p>
  {/if}
  {#if hint}
    <p class="mt-1 text-xs text-neon-green/80 flex items-center gap-1">
      <span class="w-1 h-1 rounded-full bg-neon-green inline-block"></span> {hint}
    </p>
  {/if}
</div>

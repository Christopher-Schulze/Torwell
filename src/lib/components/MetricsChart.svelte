<script lang="ts">
  import type { MetricPoint } from "$lib/stores/torStore";
  export let metrics: MetricPoint[] = [];
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

  $: memoryPath = buildPath(metrics, "memoryMB");
  $: circuitPath = buildPath(metrics, "circuitCount");
  $: failPath = buildPath(metrics, "failedAttempts");
</script>

<!-- Combined Chart -->
<svg {width} {height} class="w-full h-full overflow-visible" role="img" aria-label="Tor metrics chart" preserveAspectRatio="none">
  <!-- Memory (Background Area) -->
  {#if memoryPath}
    <path
      d={memoryPath}
      class="text-neon-green"
      fill="currentColor"
      fill-opacity="0.1"
      stroke="none"
    />
    <path
      d={memoryPath}
      class="text-neon-green"
      fill="none"
      stroke="currentColor"
      stroke-width="1"
      vector-effect="non-scaling-stroke"
    />
  {/if}

  <!-- Circuits (Line) -->
  {#if circuitPath}
    <path
      d={circuitPath}
      class="text-neon-purple"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      vector-effect="non-scaling-stroke"
    />
  {/if}

  <!-- Failures (Dotted Line) -->
  {#if failPath}
    <path
      d={failPath}
      class="text-red-500"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-dasharray="2,2"
      vector-effect="non-scaling-stroke"
    />
  {/if}
</svg>

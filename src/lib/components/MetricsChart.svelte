<script lang="ts">
  import type { MetricPoint } from "$lib/stores/torStore";
  export let metrics: MetricPoint[] = [];
  const width = 120;
  const height = 40;

  function buildPath(data: MetricPoint[]): string {
    if (data.length === 0) return "";
    const maxVal = Math.max(...data.map((d) => d.memoryMB), 1);
    const step = width / Math.max(data.length - 1, 1);
    let d = `M0 ${height}`;
    data.forEach((pt, idx) => {
      const x = idx * step;
      const y = height - (pt.memoryMB / maxVal) * height;
      d += ` L${x} ${y}`;
    });
    d += ` L${width} ${height} Z`;
    return d;
  }

  $: pathData = buildPath(metrics);
</script>

<svg {width} {height} class="text-green-400">
  {#if pathData}
    <path
      d={pathData}
      fill="currentColor"
      fill-opacity="0.3"
      stroke="currentColor"
      stroke-width="1"
    />
  {/if}
</svg>

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
  $: agePath = buildPath(metrics, "oldestAge");
  $: avgPath = buildPath(metrics, "avgCreateMs");
  $: failPath = buildPath(metrics, "failedAttempts");
  $: networkPath = buildPath(metrics, "networkBytes");
</script>

<svg {width} {height} class="text-green-400" role="img" aria-label="Tor metrics chart">
  {#if memoryPath}
    <path
      d={memoryPath}
      fill="currentColor"
      fill-opacity="0.3"
      stroke="currentColor"
      stroke-width="1"
    />
  {/if}
  {#if circuitPath}
    <path
      d={circuitPath}
      fill="none"
      stroke="blue"
      stroke-width="1"
    />
  {/if}
  {#if agePath}
    <path
      d={agePath}
      fill="none"
      stroke="orange"
      stroke-width="1"
      stroke-dasharray="2,2"
    />
  {/if}
  {#if avgPath}
    <path
      d={avgPath}
      fill="none"
      stroke="purple"
      stroke-width="1"
    />
  {/if}
  {#if failPath}
    <path
      d={failPath}
      fill="none"
      stroke="red"
      stroke-width="1"
      stroke-dasharray="4,2"
    />
  {/if}
  {#if networkPath}
    <path
      d={networkPath}
      fill="none"
      stroke="cyan"
      stroke-width="1"
    />
  {/if}
</svg>

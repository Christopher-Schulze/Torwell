<script lang="ts">
  import { Activity, Zap } from "lucide-svelte";
  import { invoke } from "$lib/api";
  import MetricsChart from "./MetricsChart.svelte";
  import CircuitList from "./CircuitList.svelte";

  export let status;
  export let totalTrafficMB = 0;
  export let pingMs: number | undefined = undefined;

  import { torStore } from "$lib/stores/torStore";
  let memoryMB: number;
  let circuitCount: number;
  let metrics = [];
  let pingHistory: number[] = [];
  const width = 120;
  const height = 40;

  function buildPath(data: number[]): string {
    if (data.length === 0) return "";
    const maxVal = Math.max(...data, 1);
    const step = width / Math.max(data.length - 1, 1);
    let d = `M0 ${height}`;
    data.forEach((pt, idx) => {
      const x = idx * step;
      const y = height - (pt / maxVal) * height;
      d += ` L${x} ${y}`;
    });
    d += ` L${width} ${height} Z`;
    return d;
  }

  $: pingPath = buildPath(pingHistory);
  $: memoryMB = $torStore.memoryUsageMB;
  $: circuitCount = $torStore.circuitCount;
  $: metrics = $torStore.metrics;

  let isPinging = false;

  // Format traffic display with automatic MB/GB conversion
  function formatTraffic(mb: number): string {
    if (mb >= 1000) {
      return `${(mb / 1000).toFixed(1)} GB`;
    }
    return `${mb} MB`;
  }

  // Ping function - execute backend ping command
  async function performPing() {
    if (isPinging) return;
    isPinging = true;
    try {
      const result = (await invoke("ping_host_series", {
        host: "google.com",
        count: 5,
      })) as number[];
      if (result.length) {
        pingMs = Math.round(result.reduce((a, b) => a + b, 0) / result.length);
        pingHistory = [...pingHistory, ...result].slice(-30);
      }
    } catch (error) {
      console.error("Ping failed:", error);
      pingMs = -1;
    } finally {
      isPinging = false;
    }
  }
</script>

<!-- Status Card -->
<div class="glass-md rounded-xl p-6" tabindex="0" role="region" aria-label="Status information">
  <div class="flex items-center justify-between gap-6">
    <!-- Status Section -->
    <div class="flex items-center gap-4">
      {#if status === "CONNECTED"}
        <div class="w-3 h-3 bg-green-500 rounded-full"></div>
        <div>
          <h3 class="text-base font-medium text-white">Connected</h3>
          <p class="text-xs text-gray-300">-</p>
        </div>
      {:else if status === "CONNECTING"}
        <div class="w-3 h-3 bg-yellow-500 rounded-full animate-pulse"></div>
        <div>
          <h3 class="text-base font-medium text-white">Connecting</h3>
          <p class="text-xs text-gray-300">-</p>
        </div>
      {:else}
        <div class="w-3 h-3 bg-red-500 rounded-full"></div>
        <div>
          <h3 class="text-base font-medium text-white">Disconnected</h3>
          <p class="text-xs text-gray-300 ml-2">-</p>
        </div>
      {/if}
    </div>

    <!-- Traffic and Metrics Section -->
    <div class="flex items-center gap-4">
      <!-- Traffic Display -->
      <div
        class="bg-black/50 rounded-lg px-2 py-1 h-8 flex items-center gap-2 min-w-[100px]"
      >
        <Activity class="w-4 h-4 text-green-400" />
        <span class="text-xs text-white font-medium"
          >{formatTraffic(totalTrafficMB)}</span
        >
      </div>

      <!-- Memory Display -->
      <div
        class="bg-black/50 rounded-lg px-2 py-1 h-8 flex items-center gap-2 min-w-[80px]"
      >
        <span class="text-xs text-white font-medium">{memoryMB} MB</span>
      </div>

      <!-- Circuits Display -->
      <div
        class="bg-black/50 rounded-lg px-2 py-1 h-8 flex items-center gap-2 min-w-[60px]"
      >
        <span class="text-xs text-white font-medium">{circuitCount}</span>
      </div>

      <!-- Ping Display -->
      <div
        class="bg-black/50 rounded-lg px-2 py-1 h-8 flex items-center gap-2 min-w-[80px]"
      >
        <Zap class="w-4 h-4 text-blue-400" />
        {#if pingMs !== undefined && pingMs >= 0}
          <span class="text-xs text-white font-medium">{pingMs} ms</span>
        {:else}
          <span class="text-xs text-gray-100">- ms</span>
        {/if}
      </div>
      <svg {width} {height} class="text-blue-400" aria-label="Ping history chart" role="img">
        {#if pingPath}
          <path d={pingPath} fill="currentColor" fill-opacity="0.3" stroke="currentColor" stroke-width="1" />
        {/if}
      </svg>

      <!-- Water Drop Ripple Button -->
      <button
        class="w-8 h-8 bg-black/50 rounded-lg hover:bg-black/60 transition-all flex items-center justify-center {isPinging
          ? 'opacity-50 cursor-not-allowed'
          : 'cursor-pointer'}"
        on:click={performPing}
        disabled={isPinging}
        title="Start Ping Test"
        aria-label="Run ping test"
      >
        {#if isPinging}
          <!-- Animated ripples during ping -->
          <div class="relative w-full h-full flex items-center justify-center">
            <div
              class="absolute w-2 h-2 bg-blue-400/60 rounded-full animate-ping"
            ></div>
            <div
              class="absolute w-3 h-3 bg-blue-400/40 rounded-full animate-ping"
              style="animation-delay: 0.2s;"
            ></div>
            <div
              class="absolute w-4 h-4 bg-blue-400/20 rounded-full animate-ping"
              style="animation-delay: 0.4s;"
            ></div>
            <div class="w-1.5 h-1.5 bg-blue-400 rounded-full"></div>
          </div>
        {:else}
          <!-- Static concentric circles -->
          <div class="relative w-full h-full flex items-center justify-center">
            <div
              class="absolute w-4 h-4 border border-white/20 rounded-full"
            ></div>
            <div
              class="absolute w-3 h-3 border border-white/30 rounded-full"
            ></div>
            <div
              class="absolute w-2 h-2 border border-white/40 rounded-full"
            ></div>
            <div class="w-1 h-1 bg-white rounded-full"></div>
          </div>
        {/if}
      </button>
    </div>
  </div>
  {#if $torStore.securityWarning}
    <p class="text-yellow-200 text-xs mt-2" role="alert">
      {$torStore.securityWarning}
    </p>
  {/if}
  <div class="mt-2">
    <MetricsChart {metrics} />
    <CircuitList show={status === 'CONNECTED'} />
  </div>
</div>

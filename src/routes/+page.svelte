<script lang="ts">
  import StatusCard from "$lib/components/StatusCard.svelte";
  import TorChain from "$lib/components/TorChain.svelte";
  import ActionCard from "$lib/components/ActionCard.svelte";
  import IdlePanel from "$lib/components/IdlePanel.svelte";
  import NetworkTools from "$lib/components/NetworkTools.svelte";
  import SecurityBanner from "$lib/components/SecurityBanner.svelte";
  import { browser } from "$app/environment";
  let LogsModalComponent: any = null;
  let SettingsModalComponent: any = null;
  import { uiStore } from "$lib/stores/uiStore";
  import { torStore } from "$lib/stores/torStore";
  import { invoke } from "$lib/api";

  import { onMount } from "svelte";

  let activeCircuit: any[] = [];
  let isolatedCircuits: { domain: string; nodes: any[] }[] = [];
  const isolatedDomain = "example.com";
  let circuitInterval: any = null;
  let trafficInterval: any = null;
  let totalTrafficMB = 0;

  async function fetchCircuit() {
    if ($torStore.status === "CONNECTED") {
      try {
        activeCircuit = (await invoke("get_active_circuit")) as any[];
      } catch (e) {
        console.error("Failed to get active circuit:", e);
        activeCircuit = [];
      }
    } else {
      activeCircuit = [];
    }
  }

  async function fetchIsolatedCircuit() {
    if ($torStore.status === "CONNECTED") {
      try {
        const nodes = (await invoke("get_isolated_circuit", { domain: isolatedDomain })) as any;
        isolatedCircuits = [{ domain: isolatedDomain, nodes }];
      } catch (e) {
        console.error("Failed to get isolated circuit:", e);
        isolatedCircuits = [];
      }
    } else {
      isolatedCircuits = [];
    }
  }

  async function fetchTraffic() {
    if ($torStore.status === "CONNECTED") {
      try {
        const stats = (await invoke("get_traffic_stats")) as any;
        const bytes = stats.bytes_sent + stats.bytes_received;
        totalTrafficMB = Math.round(bytes / 1_000_000);
      } catch (e) {
        console.error("Failed to get traffic stats:", e);
        totalTrafficMB = 0;
      }
    } else {
      totalTrafficMB = 0;
    }
  }

  // Fetch circuit info periodically when connected
  $: if ($torStore.status === "CONNECTED" && !circuitInterval) {
    fetchCircuit();
    fetchIsolatedCircuit();
    circuitInterval = setInterval(() => {
      fetchCircuit();
      fetchIsolatedCircuit();
    }, 5000);
  } else if ($torStore.status !== "CONNECTED" && circuitInterval) {
    clearInterval(circuitInterval);
    circuitInterval = null;
    activeCircuit = [];
    isolatedCircuits = [];
  }

  $: if ($torStore.status === "CONNECTED" && !trafficInterval) {
    fetchTraffic();
    trafficInterval = setInterval(fetchTraffic, 5000);
  } else if ($torStore.status !== "CONNECTED" && trafficInterval) {
    clearInterval(trafficInterval);
    trafficInterval = null;
    totalTrafficMB = 0;
  }

  onMount(() => {
    return () => {
      if (circuitInterval) {
        clearInterval(circuitInterval);
      }
      if (trafficInterval) {
        clearInterval(trafficInterval);
      }
      isolatedCircuits = [];
    };
  });

  $: if (browser && $uiStore.isLogsModalOpen && !LogsModalComponent) {
    import("$lib/components/LogsModal.svelte").then((m) => {
      LogsModalComponent = m.default;
    });
  }

  $: if (browser && $uiStore.isSettingsModalOpen && !SettingsModalComponent) {
    import("$lib/components/SettingsModal.svelte").then((m) => {
      SettingsModalComponent = m.default;
    });
  }
</script>

<div class="p-6 max-w-6xl mx-auto">
  <div
    class="bg-white/20 backdrop-blur-xl rounded-[32px] border border-white/20 p-6 flex flex-col gap-2"
  >
    <SecurityBanner />
    <StatusCard
      status={$torStore.status}
      {totalTrafficMB}
      pingMs={$torStore.pingMs}
    />

    <TorChain
      isConnected={$torStore.status === "CONNECTED"}
      isActive={$torStore.status === "CONNECTED"}
      nodeData={activeCircuit}
      isolatedCircuits={isolatedCircuits}
    />

    <ActionCard
      on:openLogs={() => uiStore.actions.openLogsModal()}
      on:openSettings={() => uiStore.actions.openSettingsModal()}
    />

    <NetworkTools />

    <IdlePanel
      connectionProgress={$torStore.bootstrapProgress}
      bootstrapMessage={$torStore.bootstrapMessage}
      currentStatus={$torStore.status}
      retryCount={$torStore.retryCount}
      retryDelay={$torStore.retryDelay}
    />
    <div class="text-right mt-2">
      <a href="/dashboard" class="text-sm text-blue-400 underline">Resource Dashboard</a>
      <span class="mx-2">|</span>
      <a href="/network" class="text-sm text-blue-400 underline">Network Monitor</a>
    </div>
  </div>
</div>

{#if LogsModalComponent}
  <svelte:component
    this={LogsModalComponent}
    bind:show={$uiStore.isLogsModalOpen}
    on:close={() => uiStore.actions.closeLogsModal()}
  />
{/if}

{#if SettingsModalComponent}
  <svelte:component
    this={SettingsModalComponent}
    bind:show={$uiStore.isSettingsModalOpen}
    on:close={() => uiStore.actions.closeSettingsModal()}
  />
{/if}

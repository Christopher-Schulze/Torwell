<script lang="ts">
  import StatusCard from "$lib/components/StatusCard.svelte";
  import TorChain from "$lib/components/TorChain.svelte";
  import ActionCard from "$lib/components/ActionCard.svelte";
  import IdlePanel from "$lib/components/IdlePanel.svelte";
  import SecurityBanner from "$lib/components/SecurityBanner.svelte";
  import { browser } from "$app/environment";
  let LogsModalComponent: any = null;
  let SettingsModalComponent: any = null;
  import { uiStore } from "$lib/stores/uiStore";
  import { torStore } from "$lib/stores/torStore";
  import { invoke } from "@tauri-apps/api/tauri";

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
        activeCircuit = await invoke("get_active_circuit");
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
        const nodes = await invoke<any>("get_isolated_circuit", {
          domain: isolatedDomain,
        });
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
        const stats = await invoke<any>("get_traffic_stats");
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

<div class="p-4 sm:p-6 max-w-6xl mx-auto">
  <div
    class="bg-white/10 sm:bg-white/20 backdrop-blur-md sm:backdrop-blur-xl rounded-[32px] border border-white/10 sm:border-white/20 p-4 sm:p-6 grid grid-cols-1 md:grid-cols-2 gap-4"
  >
    <div class="md:col-span-2">
      <SecurityBanner />
    </div>
    <div>
      <StatusCard
        status={$torStore.status}
        {totalTrafficMB}
        pingMs={$torStore.pingMs}
      />
    </div>
    <div class="md:col-span-2">
      <TorChain
        isConnected={$torStore.status === "CONNECTED"}
        isActive={$torStore.status === "CONNECTED"}
        nodeData={activeCircuit}
        {isolatedCircuits}
        cloudflareEnabled={false}
      />
    </div>
    <div class="md:col-span-2">
      <ActionCard
        on:openLogs={() => uiStore.actions.openLogsModal()}
        on:openSettings={() => uiStore.actions.openSettingsModal()}
      />
    </div>
    <div>
      <IdlePanel
        connectionProgress={$torStore.bootstrapProgress}
        bootstrapMessage={$torStore.bootstrapMessage}
        currentStatus={$torStore.status}
        retryCount={$torStore.retryCount}
        retryDelay={$torStore.retryDelay}
      />
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

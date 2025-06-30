<script lang="ts">
  import StatusCard from "$lib/components/StatusCard.svelte";
  import TorChain from "$lib/components/TorChain.svelte";
  import ActionCard from "$lib/components/ActionCard.svelte";
  import IdlePanel from "$lib/components/IdlePanel.svelte";
  import LogsModal from "$lib/components/LogsModal.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import { uiStore } from "$lib/stores/uiStore";
  import { torStore } from "$lib/stores/torStore";
  import { invoke } from "@tauri-apps/api/tauri";

  import { onMount } from "svelte";

  let activeCircuit: any[] = [];
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
    circuitInterval = setInterval(fetchCircuit, 5000);
  } else if ($torStore.status !== "CONNECTED" && circuitInterval) {
    clearInterval(circuitInterval);
    circuitInterval = null;
    activeCircuit = [];
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
    };
  });
</script>

<div class="p-6 max-w-6xl mx-auto">
  <div
    class="bg-white/20 backdrop-blur-xl rounded-[32px] border border-white/20 p-6 flex flex-col gap-2"
  >
    <StatusCard status={$torStore.status} {totalTrafficMB} pingMs={undefined} />

    <TorChain
      isConnected={$torStore.status === "CONNECTED"}
      isActive={$torStore.status === "CONNECTED"}
      nodeData={activeCircuit}
      cloudflareEnabled={false}
    />

    <ActionCard
      on:openLogs={() => uiStore.actions.openLogsModal()}
      on:openSettings={() => uiStore.actions.openSettingsModal()}
    />

    <IdlePanel
      connectionProgress={$torStore.bootstrapProgress}
      bootstrapMessage={$torStore.bootstrapMessage}
      currentStatus={$torStore.status}
      retryCount={$torStore.retryCount}
      retryDelay={$torStore.retryDelay}
    />
  </div>
</div>

<LogsModal
  bind:show={$uiStore.isLogsModalOpen}
  on:close={() => uiStore.actions.closeLogsModal()}
/>

<SettingsModal
  bind:show={$uiStore.isSettingsModalOpen}
  on:close={() => uiStore.actions.closeSettingsModal()}
/>

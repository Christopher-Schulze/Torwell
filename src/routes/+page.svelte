<script lang="ts">
  import StatusCard from "$lib/components/StatusCard.svelte";
  import TorChain from "$lib/components/TorChain.svelte";
  import ActionCard from "$lib/components/ActionCard.svelte";
  import IdlePanel from "$lib/components/IdlePanel.svelte";
  import NetworkTools from "$lib/components/NetworkTools.svelte";
  import ConnectionDiagnostics from "$lib/components/ConnectionDiagnostics.svelte";
  import SecurityBanner from "$lib/components/SecurityBanner.svelte";
  import { browser } from "$app/environment";
  let LogsModalComponent: any = null;
  let SettingsModalComponent: any = null;
  import { uiStore } from "$lib/stores/uiStore";
  import { torStore } from "$lib/stores/torStore";
  import { invoke } from "$lib/api";
  import type { CircuitPolicyReport, RelayInfo, StatusSummary } from "$lib/types";

  import { onMount } from "svelte";

  let activeCircuit: RelayInfo[] = [];
  let policyReport: CircuitPolicyReport | null = null;
  let isolatedCircuits: { domain: string; nodes: any[] }[] = [];
  const isolatedDomain = "example.com";
  let routeInterval: any = null;
  let summaryInterval: any = null;
  let totalTrafficMB = 0;
  let statusSummary: StatusSummary | null = null;

  async function fetchPolicyReport() {
    if ($torStore.status === "CONNECTED") {
      try {
        const report = await invoke<CircuitPolicyReport>("get_circuit_policy_report");
        policyReport = report;
        activeCircuit = report.relays ?? [];
      } catch (e) {
        console.error("Failed to load circuit policy report:", e);
        policyReport = null;
        activeCircuit = [];
      }
    } else {
      policyReport = null;
      activeCircuit = [];
    }
  }

  async function fetchIsolatedCircuit() {
    if ($torStore.status === "CONNECTED") {
      try {
        const nodes = await invoke<any>("get_isolated_circuit", { domain: isolatedDomain });
        isolatedCircuits = [{ domain: isolatedDomain, nodes }];
      } catch (e) {
        console.error("Failed to get isolated circuit:", e);
        isolatedCircuits = [];
      }
    } else {
      isolatedCircuits = [];
    }
  }

  async function fetchStatusSummary() {
    if ($torStore.status === "CONNECTED") {
      try {
        const summary = await invoke<StatusSummary>("get_status_summary");
        statusSummary = summary;
        totalTrafficMB = summary.total_traffic_bytes / 1_000_000;
      } catch (e) {
        console.error("Failed to load status summary:", e);
        statusSummary = null;
        totalTrafficMB = 0;
      }
    } else {
      statusSummary = null;
      totalTrafficMB = 0;
    }
  }

  // Fetch circuit info periodically when connected
  $: if ($torStore.status === "CONNECTED" && !routeInterval) {
    fetchPolicyReport();
    fetchIsolatedCircuit();
    routeInterval = setInterval(() => {
      fetchPolicyReport();
      fetchIsolatedCircuit();
    }, 5000);
  } else if ($torStore.status !== "CONNECTED" && routeInterval) {
    clearInterval(routeInterval);
    routeInterval = null;
    activeCircuit = [];
    policyReport = null;
    isolatedCircuits = [];
  }

  $: if ($torStore.status === "CONNECTED" && !summaryInterval) {
    fetchStatusSummary();
    summaryInterval = setInterval(fetchStatusSummary, 5000);
  } else if ($torStore.status !== "CONNECTED" && summaryInterval) {
    clearInterval(summaryInterval);
    summaryInterval = null;
    statusSummary = null;
    totalTrafficMB = 0;
  }

  onMount(() => {
    return () => {
      if (routeInterval) {
        clearInterval(routeInterval);
      }
      if (summaryInterval) {
        clearInterval(summaryInterval);
      }
      policyReport = null;
      isolatedCircuits = [];
    };
  });

  $: if (browser && $uiStore.isLogsModalOpen && !LogsModalComponent) {
    import("$lib/components/LogsModal.svelte").then(({ default: component }) => {
      LogsModalComponent = component;
    });
  }

  $: if (browser && $uiStore.isSettingsModalOpen && !SettingsModalComponent) {
    import("$lib/components/SettingsModal.svelte").then(({ default: component }) => {
      SettingsModalComponent = component;
    });
  }
</script>

<div class="p-6 max-w-6xl mx-auto">
  <div class="tw-surface flex flex-col gap-4">
    <SecurityBanner />
    <StatusCard
      status={$torStore.status}
      {totalTrafficMB}
      pingMs={$torStore.pingMs}
      summary={statusSummary}
      policyReport={policyReport}
    />

    <div class="grid gap-4 lg:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
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
    </div>

    <div class="grid gap-4 lg:grid-cols-2">
      <ConnectionDiagnostics />

      <IdlePanel
        connectionProgress={$torStore.bootstrapProgress}
        bootstrapMessage={$torStore.bootstrapMessage}
        currentStatus={$torStore.status}
        retryCount={$torStore.retryCount}
        retryDelay={$torStore.retryDelay}
      />
    </div>

    <NetworkTools />
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

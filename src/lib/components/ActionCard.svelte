<script lang="ts">
  import {
    Activity,
    Settings,
    Play,
    Square,
    RotateCcw,
    RefreshCw,
    AlertCircle,
  } from "lucide-svelte";
  import { createEventDispatcher } from "svelte";

  import { torStore } from "$lib/stores/torStore";
  import { invoke } from "$lib/api";
  import { addToast, addErrorToast } from "$lib/stores/toastStore";

  const dispatch = createEventDispatcher();

  let actionError: string | null = null;
  let isBuildingCircuit = false;
  let isCreatingIdentity = false;

  const parseError = (error: unknown) =>
    error instanceof Error ? error.message : String(error ?? "Unknown error");

  async function handleConnect() {
    actionError = null;
    try {
      await invoke("connect");
      addToast("Connecting to the Tor network…");
    } catch (error) {
      const message = parseError(error);
      actionError = message;
      addErrorToast("connection", message);
      console.error("Failed to connect:", error);
    }
  }

  async function handleDisconnect() {
    actionError = null;
    try {
      await invoke("disconnect");
      addToast("Disconnecting from Tor…");
    } catch (error) {
      const message = parseError(error);
      actionError = message;
      addErrorToast("connection", message);
      console.error("Failed to disconnect:", error);
    }
  }

  async function handleNewCircuit() {
    if (!isConnected || isBuildingCircuit) return;
    isBuildingCircuit = true;
    actionError = null;
    try {
      await invoke("build_circuit");
      addToast("Requested a fresh Tor circuit.");
    } catch (error) {
      const message = parseError(error);
      actionError = message;
      addErrorToast("connection", message);
      console.error("Failed to build circuit:", error);
    } finally {
      isBuildingCircuit = false;
    }
  }

  async function handleNewIdentity() {
    if (isCreatingIdentity) return;
    isCreatingIdentity = true;
    actionError = null;
    try {
      await invoke("new_identity");
      addToast("Requested a new Tor identity.");
    } catch (error) {
      const message = parseError(error);
      actionError = message;
      addErrorToast("connection", message);
      console.error("Failed to request new identity:", error);
    } finally {
      isCreatingIdentity = false;
    }
  }

  $: isConnected = $torStore.status === "CONNECTED";
  $: isStopped = $torStore.status === "DISCONNECTED";
  $: isConnecting =
    $torStore.status === "CONNECTING" || $torStore.status === "RETRYING";
  $: isRetrying = $torStore.status === "RETRYING";
  $: isDisconnecting = $torStore.status === "DISCONNECTING";
  $: hasError = $torStore.status === "ERROR";
</script>

<div class="glass-md rounded-xl p-6" role="region" aria-label="Tor controls">
  <!-- Error Message -->
  {#if $torStore.errorMessage}
    <div
      class="mb-4 p-3 bg-red-900/30 border border-red-700/50 text-red-300 rounded-lg flex items-center gap-2"
      role="alert"
      aria-live="assertive"
    >
      <AlertCircle size={16} />
      <span>
        {$torStore.errorMessage}
        {#if $torStore.errorStep}
          ({$torStore.errorStep}: {$torStore.errorSource})
        {/if}
        {#if isRetrying}
          (retry {$torStore.retryCount} in {$torStore.retryDelay}s)
        {/if}
      </span>
    </div>
  {/if}

  <!-- Four Buttons Layout -->
  <div class="grid grid-cols-4 gap-3">
    <!-- Connect/Disconnect Button -->
    {#if isStopped || hasError}
      <button
        class="tw-button tw-button--success"
        on:click={handleConnect}
        aria-label={hasError ? "Retry connection" : "Connect to Tor"}
      >
        <Play size={16} />
        {hasError ? "Retry" : "Connect"}
      </button>
    {:else if isConnecting}
      <button
        class="tw-button tw-button--pending"
        disabled={true}
        aria-live="polite"
      >
        <span class="tw-spinner" aria-hidden="true"></span>
        {#if isRetrying}
          Retrying in {$torStore.retryDelay}s (attempt {$torStore.retryCount})
        {:else}
          Connecting...
        {/if}
      </button>
    {:else if isConnected}
      <button
        class="tw-button tw-button--danger"
        on:click={handleDisconnect}
        aria-label="Disconnect from Tor"
      >
        <Square size={16} /> Disconnect
      </button>
    {:else if isDisconnecting}
      <button
        class="tw-button tw-button--pending"
        disabled={true}
      >
        <span class="tw-spinner" aria-hidden="true"></span>
        Disconnecting...
      </button>
    {/if}

    <!-- New Circuit Button -->
    <button
      class={`tw-button tw-button--accent ${
        isConnected && !isBuildingCircuit ? "" : "tw-button--disabled"
      }`}
      on:click={handleNewCircuit}
      disabled={!isConnected || isBuildingCircuit}
      aria-label="Request new circuit"
      aria-busy={isBuildingCircuit}
    >
      {#if isBuildingCircuit}
        <span class="tw-spinner" aria-hidden="true"></span>
        Creating...
      {:else}
        <RotateCcw size={16} /> New Circuit
      {/if}
    </button>

    <!-- Logs Button -->
    <button
      class="tw-button tw-button--neutral"
      on:click={() => dispatch("openLogs")}
      aria-label="Open logs"
    >
      <Activity size={16} /> Logs
    </button>

    <!-- Settings Button -->
    <button
      class="tw-button tw-button--neutral"
      on:click={() => dispatch("openSettings")}
      aria-label="Open settings"
    >
      <Settings size={16} /> Settings
    </button>
  </div>

  {#if actionError}
    <p class="mt-3 text-sm text-rose-200/90" role="status">{actionError}</p>
  {/if}
</div>

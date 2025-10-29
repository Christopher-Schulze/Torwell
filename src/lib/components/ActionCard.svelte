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
  import {
    connectionQueue,
    type ConnectionActionKey,
    type QueueState,
  } from "$lib/utils/actionQueue";

  const dispatch = createEventDispatcher();

  let actionError: string | null = null;
  let queueState: QueueState = {
    pending: null,
    queueDepth: 0,
    queuedKeys: [],
    lastError: null,
    lastSuccess: null,
  };

  const parseError = (error: unknown) =>
    error instanceof Error ? error.message : String(error ?? "Unknown error");

  const actionLabels: Record<ConnectionActionKey, string> = {
    connect: "Connect",
    disconnect: "Disconnect",
    build_circuit: "New circuit",
    new_identity: "New identity",
  };

  function formatRelative(timestamp: number) {
    const diff = Date.now() - timestamp;
    if (diff < 1_000) return "just now";
    const seconds = Math.round(diff / 1_000);
    if (seconds < 60) return `${seconds}s ago`;
    const minutes = Math.round(seconds / 60);
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.round(minutes / 60);
    return `${hours}h ago`;
  }

  const describeAction = (key: ConnectionActionKey) => actionLabels[key];

  const isActionBusy = (key: ConnectionActionKey) =>
    queueState.pending === key || queueState.queuedKeys.includes(key);

  async function scheduleCommand(
    key: ConnectionActionKey,
    action: () => Promise<void>,
    successMessage: string,
  ) {
    if (isActionBusy(key)) return;
    actionError = null;
    connectionQueue.clearError(key);
    try {
      const res = await connectionQueue.run(key, async () => {
        await action();
      });
      if (res.status === "completed") {
        addToast(successMessage);
      }
    } catch (error) {
      const message = parseError(error);
      actionError = message;
      addErrorToast("connection", message);
      console.error(`Failed to ${actionLabels[key].toLowerCase()}:`, error);
    }
  }

  async function handleConnect() {
    if (isConnecting) return;
    await scheduleCommand("connect", () => invoke("connect"), "Connecting to the Tor network…");
  }

  async function handleDisconnect() {
    if (isDisconnecting) return;
    await scheduleCommand("disconnect", () => invoke("disconnect"), "Disconnecting from Tor…");
  }

  async function handleNewCircuit() {
    if (!isConnected) return;
    await scheduleCommand("build_circuit", () => invoke("build_circuit"), "Requested a fresh Tor circuit.");
  }

  async function handleNewIdentity() {
    if (!isConnected) return;
    await scheduleCommand("new_identity", () => invoke("new_identity"), "Requested a new Tor identity.");
  }

  $: isConnected = $torStore.status === "CONNECTED";
  $: isConnecting =
    $torStore.status === "CONNECTING" || $torStore.status === "RETRYING";
  $: isRetrying = $torStore.status === "RETRYING";
  $: isDisconnecting = $torStore.status === "DISCONNECTING";
  $: hasError = $torStore.status === "ERROR";
  $: queueState = $connectionQueue;
  $: if (queueState.lastError && queueState.lastError.message !== actionError) {
    actionError = queueState.lastError.message;
  }
  $: lastSuccessLabel = queueState.lastSuccess
    ? `${actionLabels[queueState.lastSuccess.key]} · ${formatRelative(queueState.lastSuccess.at)}`
    : null;
  $: activeActionLabel = queueState.pending ? describeAction(queueState.pending) : null;
  $: queuedCountText =
    queueState.queueDepth > 0
      ? `${queueState.queueDepth} follow-up ${queueState.queueDepth === 1 ? "command" : "commands"} queued`
      : null;

  const statusHints: Record<string, { title: string; body: string; tone: string }> = {
    DISCONNECTED: {
      title: "Bereit",
      body: "Starte eine neue Tor-Sitzung, um geschützt zu surfen.",
      tone: "text-slate-200",
    },
    CONNECTING: {
      title: "Verbindung wird aufgebaut",
      body: "Arti initialisiert Relais und lädt Konsensusdaten.",
      tone: "text-amber-200",
    },
    RETRYING: {
      title: "Erneuter Versuch",
      body: "Backoff aktiv – Status unter Idle Panel prüfen.",
      tone: "text-amber-200",
    },
    CONNECTED: {
      title: "Verbunden",
      body: "Tor-Tunnel aktiv. Du kannst neue Identitäten erzwingen.",
      tone: "text-emerald-200",
    },
    DISCONNECTING: {
      title: "Trennung läuft",
      body: "Aktive Circuits werden geschlossen.",
      tone: "text-blue-200",
    },
    ERROR: {
      title: "Fehler",
      body: "Siehe Log-Ausgabe für Details. Du kannst den Connect erneut versuchen.",
      tone: "text-rose-200",
    },
  };

  $: hint = statusHints[$torStore.status] ?? statusHints.DISCONNECTED;

  type ControlVisual = {
    gradient: string;
    shadow: string;
    icon: typeof Play;
    label: string;
    caption: string;
  };

  const connectVisuals: {
    default: ControlVisual;
    connecting: ControlVisual;
    connected: ControlVisual;
    error: ControlVisual;
    disconnecting: ControlVisual;
  } = {
    default: {
      gradient: "from-emerald-500/30 via-sky-500/20 to-indigo-500/20",
      shadow: "shadow-[0_40px_95px_rgba(16,185,129,0.45)]",
      icon: Play,
      label: "Connect",
      caption: "Sichere Verbindung aufbauen",
    },
    connecting: {
      gradient: "from-amber-500/35 via-amber-400/20 to-purple-500/20",
      shadow: "shadow-[0_40px_95px_rgba(245,158,11,0.45)]",
      icon: RefreshCw,
      label: "Connecting…",
      caption: "Tor bootstrap läuft",
    },
    connected: {
      gradient: "from-rose-500/35 via-orange-400/25 to-amber-400/20",
      shadow: "shadow-[0_40px_95px_rgba(239,68,68,0.45)]",
      icon: Square,
      label: "Disconnect",
      caption: "Sitzung beenden",
    },
    error: {
      gradient: "from-rose-500/40 via-amber-400/25 to-purple-500/20",
      shadow: "shadow-[0_40px_95px_rgba(248,113,113,0.45)]",
      icon: RefreshCw,
      label: "Retry",
      caption: "Erneut verbinden",
    },
    disconnecting: {
      gradient: "from-blue-500/35 via-slate-500/20 to-slate-700/20",
      shadow: "shadow-[0_40px_95px_rgba(96,165,250,0.35)]",
      icon: RefreshCw,
      label: "Disconnecting…",
      caption: "Sitzung wird beendet",
    },
  };

  $: connectVisual = (() => {
    if (isDisconnecting) return connectVisuals.disconnecting;
    if (isConnecting) return connectVisuals.connecting;
    if (isConnected) return connectVisuals.connected;
    if (hasError) return connectVisuals.error;
    return connectVisuals.default;
  })();

  const connectActions: ConnectionActionKey[] = ["connect", "disconnect"];
  $: connectBusy =
    isConnecting ||
    isDisconnecting ||
    connectActions.some((key) => isActionBusy(key));
  $: circuitBusy = isActionBusy("build_circuit");
  $: identityBusy = isActionBusy("new_identity");
  $: connectDisabled = connectBusy;
  $: connectAction = isConnected ? handleDisconnect : handleConnect;
  $: circuitButtonTone = isConnected && !circuitBusy ? "" : "opacity-60";
  $: identityButtonTone = isConnected && !identityBusy ? "" : "opacity-60";
</script>

<div
  class="relative overflow-hidden rounded-[32px] border border-white/10 bg-slate-950/40 p-6 shadow-[0_40px_100px_rgba(8,12,32,0.65)]"
  role="region"
  aria-label="Tor controls"
>
  <div class="pointer-events-none absolute -inset-24 bg-gradient-to-br from-slate-800/60 via-indigo-500/30 to-purple-500/20 opacity-60 blur-3xl"></div>
  <div class="pointer-events-none absolute inset-0 bg-gradient-to-tr from-emerald-400/10 via-transparent to-indigo-500/15"></div>
  <div class="pointer-events-none absolute -inset-32 tw-orbit border border-emerald-300/20"></div>
  <div class="pointer-events-none absolute -inset-20 tw-orbit border border-indigo-400/15" style="animation-duration: 46s"></div>

  <div class="relative z-10 space-y-6">
    {#if $torStore.errorMessage}
      <div
        class="flex items-start gap-3 rounded-2xl border border-rose-300/40 bg-rose-500/15 p-4 text-sm text-rose-100 shadow-[0_25px_65px_rgba(248,113,113,0.4)]"
        role="alert"
        aria-live="assertive"
      >
        <AlertCircle class="mt-0.5 h-4 w-4" />
        <div class="space-y-1">
          <p class="font-medium">{$torStore.errorMessage}</p>
          <p class="text-xs text-rose-100/80">
            {#if $torStore.errorStep}
              {$torStore.errorStep}: {$torStore.errorSource}
            {/if}
            {#if isRetrying}
              · Retry {$torStore.retryCount} in {$torStore.retryDelay}s
            {/if}
          </p>
        </div>
      </div>
    {/if}

    <div>
      <p class={`text-[11px] uppercase tracking-[0.32em] ${hint.tone}`}>{hint.title}</p>
      <p class="mt-1 text-sm text-white/75">{hint.body}</p>
    </div>

    <div class="grid grid-cols-1 gap-3 md:grid-cols-4">
      <button
        type="button"
        class={`group relative flex h-full flex-col justify-between overflow-hidden rounded-3xl border border-white/15 bg-gradient-to-br ${connectVisual.gradient} p-5 text-left text-white transition-transform duration-200 ease-out ${connectVisual.shadow} disabled:cursor-not-allowed disabled:opacity-75 md:col-span-2 md:row-span-2`}
        on:click={connectAction}
        disabled={connectDisabled}
        aria-live="polite"
        aria-label={isConnected ? "Disconnect from Tor" : "Connect to Tor"}
      >
        <div class="flex items-center justify-between">
          <svelte:component this={connectVisual.icon} class="h-5 w-5 text-white/90" aria-hidden="true" />
          {#if connectBusy}
            <span class="tw-spinner border-white/30"></span>
          {/if}
        </div>
        <div class="space-y-1">
          <span class="text-2xl font-semibold tracking-wide">{connectVisual.label}</span>
          <span class="text-sm text-white/80">{connectVisual.caption}</span>
        </div>
        {#if isRetrying}
          <p class="text-[11px] text-amber-100/90">
            Retry in {$torStore.retryDelay}s · Attempt {$torStore.retryCount}
          </p>
        {/if}
      </button>

      <button
        type="button"
        class={`relative overflow-hidden rounded-2xl border border-white/12 bg-white/5 px-4 py-3 text-left text-white shadow-[0_25px_60px_rgba(12,18,46,0.55)] transition-transform duration-200 ease-out hover:-translate-y-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-300/60 ${circuitButtonTone}`}
        on:click={handleNewCircuit}
        disabled={!isConnected || circuitBusy}
        aria-label="Request new circuit"
        aria-busy={circuitBusy}
      >
        <div class="absolute inset-0 bg-gradient-to-br from-sky-400/30 via-indigo-400/25 to-purple-400/20"></div>
        <div class="relative flex items-center gap-3">
          <RotateCcw class="h-4 w-4" />
          <div class="flex flex-col text-sm">
            <span class="font-semibold">New Circuit</span>
            <span class="text-[11px] text-white/70">Fresh relay path</span>
          </div>
          {#if circuitBusy}
            <span class="tw-spinner ml-auto border-white/30"></span>
          {/if}
        </div>
      </button>

      <button
        type="button"
        class={`relative overflow-hidden rounded-2xl border border-white/12 bg-white/5 px-4 py-3 text-left text-white shadow-[0_25px_60px_rgba(12,18,46,0.55)] transition-transform duration-200 ease-out hover:-translate-y-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-300/60 ${identityButtonTone}`}
        on:click={handleNewIdentity}
        disabled={!isConnected || identityBusy}
        aria-label="Request new identity"
        aria-busy={identityBusy}
      >
        <div class="absolute inset-0 bg-gradient-to-br from-purple-400/30 via-pink-400/25 to-rose-400/20"></div>
        <div class="relative flex items-center gap-3">
          <RefreshCw class="h-4 w-4" />
          <div class="flex flex-col text-sm">
            <span class="font-semibold">New Identity</span>
            <span class="text-[11px] text-white/70">Retire circuits</span>
          </div>
          {#if identityBusy}
            <span class="tw-spinner ml-auto border-white/30"></span>
          {/if}
        </div>
      </button>

      <button
        type="button"
        class="relative overflow-hidden rounded-2xl border border-white/12 bg-white/5 px-4 py-3 text-left text-white shadow-[0_25px_60px_rgba(12,18,46,0.55)] transition-transform duration-200 ease-out hover:-translate-y-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-300/60"
        on:click={() => dispatch("openLogs")}
        aria-label="Open logs"
      >
        <div class="absolute inset-0 bg-gradient-to-br from-indigo-400/25 via-slate-500/20 to-slate-900/30"></div>
        <div class="relative flex items-center gap-3">
          <Activity class="h-4 w-4" />
          <div class="flex flex-col text-sm">
            <span class="font-semibold">Logs</span>
            <span class="text-[11px] text-white/70">Inspect events</span>
          </div>
        </div>
      </button>

      <button
        type="button"
        class="relative overflow-hidden rounded-2xl border border-white/12 bg-white/5 px-4 py-3 text-left text-white shadow-[0_25px_60px_rgba(12,18,46,0.55)] transition-transform duration-200 ease-out hover:-translate-y-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-violet-300/60"
        on:click={() => dispatch("openSettings")}
        aria-label="Open settings"
      >
        <div class="absolute inset-0 bg-gradient-to-br from-violet-400/30 via-indigo-400/20 to-blue-400/20"></div>
        <div class="relative flex items-center gap-3">
          <Settings class="h-4 w-4" />
          <div class="flex flex-col text-sm">
            <span class="font-semibold">Settings</span>
            <span class="text-[11px] text-white/70">Policy & bridges</span>
          </div>
        </div>
      </button>
    </div>

    {#if activeActionLabel}
      <div
        class="flex items-center gap-2 rounded-2xl border border-white/12 bg-white/10 px-4 py-2 text-xs text-white/85 shadow-[0_22px_55px_rgba(8,12,32,0.45)] backdrop-blur-xl"
        role="status"
        aria-live="polite"
      >
        <span class="inline-flex h-2.5 w-2.5 animate-pulse rounded-full bg-gradient-to-r from-emerald-300 to-sky-400 shadow-[0_0_12px_rgba(16,185,129,0.55)]"></span>
        <span>Processing {activeActionLabel}…</span>
      </div>
    {/if}

    {#if queuedCountText}
      <p class="text-[11px] uppercase tracking-[0.28em] text-slate-200/60">{queuedCountText}</p>
    {/if}

    {#if lastSuccessLabel}
      <p class="text-[11px] text-slate-300/70">Last action: {lastSuccessLabel}</p>
    {/if}

    {#if actionError}
      <div
        class="flex items-start gap-2 rounded-2xl border border-rose-300/40 bg-rose-500/15 p-3 text-sm text-rose-100 shadow-[0_20px_55px_rgba(248,113,113,0.35)]"
        role="status"
      >
        <AlertCircle class="mt-0.5 h-4 w-4" />
        <span>{actionError}</span>
      </div>
    {/if}
  </div>
</div>

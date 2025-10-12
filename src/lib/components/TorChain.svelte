<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { get } from "svelte/store";
  import { uiStore } from "$lib/stores/uiStore";
  import {
    COUNTRY_OPTIONS,
    DEFAULT_ROUTE_CODES,
    ensureUniqueRoute,
    getCountryFlag,
    getCountryLabel,
    isKnownCountry,
    normaliseCountryCode,
  } from "$lib/utils/countries";

  type TorNode = {
    nickname: string;
    ip_address: string;
    country: string | null;
    status?: "online" | "offline" | "building" | "unknown";
    latency?: number | null;
    bandwidth?: number | null;
  };

  type IsolatedCircuit = { domain: string; nodes: TorNode[] };

  export let isConnected = false;
  export let isActive = true;
  export let entryCountry: string | null = null;
  export let middleCountry: string | null = null;
  export let exitCountry: string | null = null;
  export let nodeData: TorNode[] = [];
  export let isolatedCircuits: IsolatedCircuit[] = [];

  type RouteSlot = "entry" | "middle" | "exit";

  type NodeStatus = "active" | "disconnected" | "pending" | "paused" | "degraded";

  type DiagnosticLevel = "info" | "warning" | "error";

  type Diagnostic = { level: DiagnosticLevel; message: string };

  type ChainSlot = {
    key: "client" | "entry" | "middle" | "exit" | "cloudflare";
    label: string;
    description: string;
    icon: string;
    index?: number;
    requiresCloudflare?: boolean;
  };

  type RenderedSlot = ChainSlot & {
    node: TorNode | null;
    available: boolean;
    status: NodeStatus;
  };

  const dispatch = createEventDispatcher<{
    activeChange: boolean;
    selectionChange: { slot: RouteSlot; code: string };
  }>();

  const fallbackCodes = COUNTRY_OPTIONS.map((option) => option.code);

  const STATUS_LABEL: Record<NodeStatus, string> = {
    active: "Active",
    disconnected: "Offline",
    pending: "Building",
    paused: "Paused",
    degraded: "Degraded",
  };

  const STATUS_ICON: Record<NodeStatus, string> = {
    active: "🟢",
    disconnected: "🔴",
    pending: "🟡",
    paused: "⏸️",
    degraded: "🟠",
  };

  const STATUS_STYLE: Record<NodeStatus, string> = {
    active: "bg-emerald-500/20 text-emerald-200 ring-emerald-400/40",
    disconnected: "bg-rose-500/20 text-rose-200 ring-rose-400/40",
    pending: "bg-amber-500/20 text-amber-100 ring-amber-400/40",
    paused: "bg-slate-500/20 text-slate-200 ring-slate-400/30",
    degraded: "bg-orange-500/20 text-orange-100 ring-orange-400/40",
  };

  const DIAGNOSTIC_STYLE: Record<DiagnosticLevel, string> = {
    info: "border-sky-400/40 bg-sky-500/10 text-sky-100",
    warning: "border-amber-400/40 bg-amber-500/10 text-amber-100",
    error: "border-rose-400/40 bg-rose-500/10 text-rose-100",
  };

  const DIAGNOSTIC_ICON: Record<DiagnosticLevel, string> = {
    info: "ℹ️",
    warning: "⚠️",
    error: "⛔",
  };

  let [entrySelection, middleSelection, exitSelection] = ensureUniqueRoute(
    [entryCountry, middleCountry, exitCountry],
    DEFAULT_ROUTE_CODES,
  ) as [string, string, string];

  $: storeRoute = ensureUniqueRoute(
    [
      $uiStore.settings.entryCountry,
      $uiStore.settings.middleCountry,
      $uiStore.settings.exitCountry,
    ],
    fallbackCodes,
  ) as [string, string, string];

  $: propRoute = ensureUniqueRoute(
    [entryCountry, middleCountry, exitCountry],
    fallbackCodes,
  ) as [string, string, string];

  $: {
    if (!entryCountry && entrySelection !== storeRoute[0]) entrySelection = storeRoute[0];
    if (!middleCountry && middleSelection !== storeRoute[1]) middleSelection = storeRoute[1];
    if (!exitCountry && exitSelection !== storeRoute[2]) exitSelection = storeRoute[2];
  }

  $: {
    if (entryCountry && entrySelection !== propRoute[0]) entrySelection = propRoute[0];
    if (middleCountry && middleSelection !== propRoute[1]) middleSelection = propRoute[1];
    if (exitCountry && exitSelection !== propRoute[2]) exitSelection = propRoute[2];
  }

  const syncRouteWithStore = (route: [string, string, string]) => {
    const current = get(uiStore).settings;
    const payload: { entry?: string | null; middle?: string | null; exit?: string | null } = {};
    if (route[0] !== current.entryCountry) payload.entry = route[0];
    if (route[1] !== current.middleCountry) payload.middle = route[1];
    if (route[2] !== current.exitCountry) payload.exit = route[2];
    if (Object.keys(payload).length > 0) {
      uiStore.actions.setCircuitCountries(payload);
    }
  };

  const handleActiveChange = (event: Event) => {
    const next = (event.target as HTMLInputElement).checked;
    dispatch("activeChange", next);
  };

  const applySelectionChange = (
    slot: RouteSlot,
    rawValue: string,
    options: { syncStore?: boolean } = { syncStore: true },
  ) => {
    const code = normaliseCountryCode(rawValue);
    if (!code) return;
    const nextRoute = ensureUniqueRoute(
      [
        slot === "entry" ? code : entrySelection,
        slot === "middle" ? code : middleSelection,
        slot === "exit" ? code : exitSelection,
      ],
      fallbackCodes,
    ) as [string, string, string];
    [entrySelection, middleSelection, exitSelection] = nextRoute;
    if (options.syncStore) {
      syncRouteWithStore(nextRoute);
    }
    const slotIndex = slot === "entry" ? 0 : slot === "middle" ? 1 : 2;
    dispatch("selectionChange", { slot, code: nextRoute[slotIndex] });
  };

  const handleSelectChange = (slot: RouteSlot) => (event: Event) =>
    applySelectionChange(slot, (event.target as HTMLSelectElement).value);

  $: cloudflareEnabled = $uiStore.cloudflareEnabled;

  let selectedExitCountry = "";
  $: preferredExitCountry = normaliseCountryCode($uiStore.settings.exitCountry) ?? "";
  $: if (selectedExitCountry !== preferredExitCountry) selectedExitCountry = preferredExitCountry;

  const changeExitCountry = (event: Event) => {
    const rawValue = (event.target as HTMLSelectElement).value;
    const code = normaliseCountryCode(rawValue);
    if (code) {
      applySelectionChange("exit", code, { syncStore: false });
    }
    uiStore.actions.setExitCountry(code ?? null);
  };

  const CHAIN_SLOTS: ReadonlyArray<ChainSlot> = [
    { key: "client", label: "You", icon: "🌐", description: "Local device" },
    {
      key: "entry",
      label: "Entry Node",
      icon: "🛡️",
      index: 0,
      description: "Guards your connection",
    },
    {
      key: "middle",
      label: "Middle Node",
      icon: "🔒",
      index: 1,
      description: "Blends traffic",
    },
    {
      key: "exit",
      label: "Exit Node",
      icon: "🚪",
      index: 2,
      description: "Final hop to destination",
    },
    {
      key: "cloudflare",
      label: "Cloudflare",
      icon: "☁️",
      index: 3,
      requiresCloudflare: true,
      description: "Cloudflare edge",
    },
  ];

  const resolveNodeForIndex = (index: number | undefined) =>
    typeof index === "number" && index >= 0 ? nodeData[index] ?? null : null;

  const computeSlotStatus = (slot: ChainSlot, node: TorNode | null): NodeStatus => {
    if (slot.key === "client") {
      return isActive ? "active" : "paused";
    }
    if (!isActive) return "paused";
    if (!isConnected) return "disconnected";
    if (!node) return "pending";
    if (node.status === "offline") return "degraded";
    if (node.status === "building") return "pending";
    return "active";
  };

  const getPlannedCodeForSlot = (slotKey: ChainSlot["key"]) => {
    if (slotKey === "entry") return entrySelection;
    if (slotKey === "middle") return middleSelection;
    if (slotKey === "exit") return exitSelection;
    return null;
  };

  const renderIp = (slot: { node: TorNode | null }) =>
    isConnected && slot.node?.ip_address ? slot.node.ip_address : "–";

  const renderNickname = (slot: { node: TorNode | null }) =>
    isConnected && slot.node?.nickname ? slot.node.nickname : "–";

  const renderCountry = (slot: (ChainSlot & { node: TorNode | null })) => {
    if (isConnected && slot.node?.country) {
      return getCountryFlag(slot.node.country) + " " + getCountryLabel(slot.node.country);
    }
    const planned = getPlannedCodeForSlot(slot.key);
    return planned ? getCountryFlag(planned) + " " + getCountryLabel(planned) : "–";
  };

  const renderLatency = (slot: { node: TorNode | null }) =>
    isConnected && typeof slot.node?.latency === "number" && Number.isFinite(slot.node.latency)
      ? String(Math.round(slot.node.latency)) + " ms"
      : null;

  const renderBandwidth = (slot: { node: TorNode | null }) =>
    isConnected && typeof slot.node?.bandwidth === "number" && Number.isFinite(slot.node.bandwidth)
      ? (slot.node.bandwidth / 1024).toFixed(1) + " MB/s"
      : null;

  let renderedSlots: RenderedSlot[] = [];

  $: renderedSlots = CHAIN_SLOTS.map((slot) => {
    const node = resolveNodeForIndex(slot.index);
    const available =
      !slot.requiresCloudflare || (slot.requiresCloudflare && cloudflareEnabled);
    const status = computeSlotStatus(slot, node);
    return {
      ...slot,
      node,
      available,
      status,
    } satisfies RenderedSlot;
  }).filter((slot) => slot.available);

  const describeCircuit = (circuit: IsolatedCircuit) =>
    circuit.nodes
      .map((node) => node.nickname + " (" + getCountryLabel(node.country) + ")")
      .join(" → ");

  const computeDiagnostics = (): Diagnostic[] => {
    const storeState = get(uiStore);
    const messages: Diagnostic[] = [];

    if (!isActive) {
      messages.push({
        level: "info",
        message: "The chain is paused. Resume to establish or maintain tunnels.",
      });
    }

    if (!isConnected) {
      messages.push({
        level: "info",
        message: "Connect to Tor to populate live node information.",
      });
    } else {
      const expectedNodes = cloudflareEnabled ? 4 : 3;
      const activeNodes = nodeData.filter((node) => !!node?.ip_address).length;
      if (activeNodes < expectedNodes) {
        messages.push({
          level: "warning",
          message: "Only " + activeNodes + " of " + expectedNodes + " nodes have reported details.",
        });
      }
    }

    const hopCountries = nodeData
      .slice(0, 3)
      .map((node) => normaliseCountryCode(node?.country ?? null))
      .filter((code): code is string => !!code);

    const repeatedCountries = [
      ...new Set(hopCountries.filter((code, idx) => hopCountries.indexOf(code) !== idx)),
    ];
    if (repeatedCountries.length > 0) {
      messages.push({
        level: "warning",
        message:
          "Multiple hops share the same country (" +
          repeatedCountries.map((code) => getCountryLabel(code)).join(", ") +
          ").",
      });
    }

    nodeData.forEach((node, index) => {
      if (isConnected && (!node?.ip_address || !node?.nickname)) {
        const missing =
          !node?.ip_address && !node?.nickname
            ? "identity details"
            : !node?.ip_address
            ? "an IP address"
            : "a nickname";
        messages.push({
          level: "info",
          message: "Hop " + (index + 1) + " is missing " + missing + ".",
        });
      }
    });

    const exitNode = nodeData[2];
    const preferredExit = normaliseCountryCode(storeState.settings.exitCountry);
    const exitCode = normaliseCountryCode(exitNode?.country ?? null);
    if (isConnected && preferredExit && exitCode && exitCode !== preferredExit) {
      messages.push({
        level: "info",
        message:
          "Current exit node (" +
          getCountryLabel(exitCode) +
          ") differs from your preferred selection (" +
          getCountryLabel(preferredExit) +
          ").",
      });
    }

    if (!cloudflareEnabled && nodeData[3]) {
      messages.push({
        level: "info",
        message: "A Cloudflare hop is available but Cloudflare acceleration is disabled.",
      });
    }

    const unknownSelections = [entrySelection, middleSelection, exitSelection].filter(
      (code) => !isKnownCountry(code),
    );
    if (unknownSelections.length > 0) {
      messages.push({
        level: "error",
        message: "One or more selected countries are invalid. Please choose recognised ISO codes.",
      });
    }

    return messages;
  };

  $: diagnostics = computeDiagnostics();
</script>


<div
  class="glass-md rounded-2xl p-6 shadow-lg border border-white/15"
  role="region"
  aria-label="Tor chain configuration"
>
  <div
    class="flex flex-col gap-4 md:grid md:grid-cols-[minmax(0,1fr)_repeat(3,minmax(0,160px))_auto] md:items-center"
  >
    <h3 class="text-base font-semibold text-white tracking-wide">Chain of Nodes</h3>

    <div class="flex gap-3 md:col-span-3 flex-wrap">
      <label class="flex items-center gap-2 text-xs text-white/80 font-medium">
        <span class="whitespace-nowrap">Entry node</span>
        <select
          class="h-8 rounded-lg bg-black/60 border border-white/20 px-2 text-xs text-white focus:border-white/50 focus:outline-none"
          value={entrySelection}
          on:change={handleSelectChange("entry")}
        >
          {#each COUNTRY_OPTIONS as option}
            <option value={option.code} class="text-xs text-gray-900">
              {getCountryFlag(option.code)} {option.name}
            </option>
          {/each}
        </select>
      </label>

      <label class="flex items-center gap-2 text-xs text-white/80 font-medium">
        <span class="whitespace-nowrap">Middle node</span>
        <select
          class="h-8 rounded-lg bg-black/60 border border-white/20 px-2 text-xs text-white focus:border-white/50 focus:outline-none"
          value={middleSelection}
          on:change={handleSelectChange("middle")}
        >
          {#each COUNTRY_OPTIONS as option}
            <option value={option.code} class="text-xs text-gray-900">
              {getCountryFlag(option.code)} {option.name}
            </option>
          {/each}
        </select>
      </label>

      <label class="flex items-center gap-2 text-xs text-white/80 font-medium">
        <span class="whitespace-nowrap">Exit node</span>
        <select
          class="h-8 rounded-lg bg-black/60 border border-white/20 px-2 text-xs text-white focus:border-white/50 focus:outline-none"
          value={exitSelection}
          on:change={handleSelectChange("exit")}
        >
          {#each COUNTRY_OPTIONS as option}
            <option value={option.code} class="text-xs text-gray-900">
              {getCountryFlag(option.code)} {option.name}
            </option>
          {/each}
        </select>
      </label>
    </div>

    <label class="ml-auto flex items-center gap-2 text-xs text-white/80 font-medium">
      <span>Active</span>
      <span class="relative inline-flex items-center">
        <input
          type="checkbox"
          class="sr-only"
          bind:checked={isActive}
          on:change={handleActiveChange}
          aria-label="Toggle chain active"
        />
        <span
          class="block h-4 w-8 rounded-full bg-gray-600 transition-colors"
          class:bg-emerald-500={isActive}
        ></span>
        <span
          class="absolute left-0.5 top-0.5 h-3 w-3 rounded-full bg-white transition-transform"
          class:translate-x-4={isActive}
        ></span>
      </span>
    </label>
  </div>

  <div class="mt-4 flex flex-wrap gap-3 items-center">
    <label class="text-xs text-white/80 font-medium" for="exit-country">Preferred exit</label>
    <div class="relative">
      <select
        id="exit-country"
        class="h-8 w-48 rounded-lg bg-black/60 border border-white/20 px-2 text-xs text-white focus:border-white/50 focus:outline-none"
        bind:value={selectedExitCountry}
        aria-label="Preferred exit country"
        on:change={changeExitCountry}
      >
        <option value="">Auto</option>
        {#each COUNTRY_OPTIONS as option}
          <option value={option.code}>{getCountryFlag(option.code)} {option.name}</option>
        {/each}
      </select>
      <svg
        class="pointer-events-none absolute right-2 top-1/2 h-3 w-3 -translate-y-1/2 text-white"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </div>
  </div>

  <div class="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
    {#each renderedSlots as slot (slot.key)}
      <article
        class="relative flex min-h-[220px] flex-col rounded-xl border border-white/10 bg-black/40 p-4 text-left shadow-lg backdrop-blur"
        aria-label={`${slot.label} node`}
      >
        <div class="absolute right-3 top-3">
          <span
            class={`inline-flex items-center gap-1 rounded-full px-2 py-1 text-[11px] font-medium ring-1 ${STATUS_STYLE[slot.status]}`}
          >
            <span aria-hidden="true">{STATUS_ICON[slot.status]}</span>
            <span class="tracking-wide text-white/90">{STATUS_LABEL[slot.status]}</span>
          </span>
        </div>
        <div class="mb-3 flex items-center gap-3">
          <div class="flex h-11 w-11 items-center justify-center rounded-lg bg-white/5 text-2xl">
            <span aria-hidden="true">{slot.icon}</span>
          </div>
          <div class="space-y-0.5">
            <p class="text-sm font-semibold text-white">{slot.label}</p>
            <p class="text-[11px] text-white/60">{slot.description}</p>
          </div>
        </div>
        <dl class="flex flex-1 flex-col gap-2 text-xs text-white/75">
          <div class="flex justify-between gap-3">
            <dt class="font-medium text-white/70">IP</dt>
            <dd class="text-right font-mono text-[11px]">{renderIp(slot)}</dd>
          </div>
          <div class="flex justify-between gap-3">
            <dt class="font-medium text-white/70">
              {isConnected && slot.node ? "Country" : "Planned country"}
            </dt>
            <dd class="text-right">{renderCountry(slot)}</dd>
          </div>
          <div class="flex justify-between gap-3">
            <dt class="font-medium text-white/70">Nickname</dt>
            <dd class="text-right">{renderNickname(slot)}</dd>
          </div>
          {#if renderLatency(slot)}
            <div class="flex justify-between gap-3">
              <dt class="font-medium text-white/70">Latency</dt>
              <dd class="text-right">{renderLatency(slot)}</dd>
            </div>
          {/if}
          {#if renderBandwidth(slot)}
            <div class="flex justify-between gap-3">
              <dt class="font-medium text-white/70">Throughput</dt>
              <dd class="text-right">{renderBandwidth(slot)}</dd>
            </div>
          {/if}
        </dl>
        {#if slot.key === "client"}
          <p class="mt-3 text-[11px] text-white/60">
            {isActive ? "Local device" : "Awaiting activation"}
          </p>
        {/if}
      </article>
    {/each}
  </div>
  {#if diagnostics.length > 0}
    <div class="mt-6 space-y-2" role="status" aria-live="polite">
      {#each diagnostics as diagnostic, index (diagnostic.message + index)}
        <div
          class={`flex items-start gap-3 rounded-lg border px-3 py-2 text-xs leading-relaxed ${DIAGNOSTIC_STYLE[diagnostic.level]}`}
        >
          <span class="text-sm" aria-hidden="true">{DIAGNOSTIC_ICON[diagnostic.level]}</span>
          <p class="flex-1 text-white/80">{diagnostic.message}</p>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if isolatedCircuits.length > 0}
  <div class="mt-6 rounded-xl border border-white/10 bg-black/30 p-4">
    <h4 class="text-sm font-semibold text-white">Isolated Circuits</h4>
    <ul class="mt-2 space-y-1 text-xs text-white/80">
      {#each isolatedCircuits as circuit}
        <li>
          <span class="font-medium text-white">{circuit.domain}</span>: {describeCircuit(circuit)}
        </li>
      {/each}
    </ul>
  </div>
{/if}

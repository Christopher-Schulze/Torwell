<script lang="ts">
  import { createEventDispatcher, tick } from "svelte";
  import { get } from "svelte/store";
  import { X, Edit3, Plus } from "lucide-svelte";
  import { uiStore } from "$lib/stores/uiStore";
  import { invoke } from "$lib/api";
  import TorrcEditorModal from "./TorrcEditorModal.svelte";
  import WorkerSetupModal from "./WorkerSetupModal.svelte";
  import { parseWorkerList } from "../../../scripts/import_workers";
  import type { TorrcProfile } from "$lib/types";
  import {
    COUNTRY_OPTIONS,
    DEFAULT_ROUTE_CODES,
    createFastCountrySet,
    ensureUniqueRoute,
    getCountryFlag,
    getCountryLabel,
    isFastCountry,
    normaliseCountryCode,
  } from "$lib/utils/countries";

  type CountryOption = { code: string; name: string };

  const presetURL = new URL("../bridge_presets.json", import.meta.url).href;

  const cloneCountryOptions = (): CountryOption[] =>
    COUNTRY_OPTIONS.map((option) => ({ ...option }));

  let availableBridges: string[] = [];
  let bridgePresets: { name: string; bridges: string[] }[] = [];

  let entryOptions: CountryOption[] = cloneCountryOptions();
  let middleOptions: CountryOption[] = cloneCountryOptions();
  let exitOptions: CountryOption[] = cloneCountryOptions();
  let baseFastCountryCodes = createFastCountrySet();
  let fastCountryCodes = createFastCountrySet();
  let showFastOnly = false;
  let customFastSelection: string[] = [];
  let fastOverrideCandidate = "";
  let fastOverrideOptions: CountryOption[] = [...COUNTRY_OPTIONS];
  let includeBridgesInTorrc = false;
  let torrcPreview: TorrcProfile | null = null;
  let torrcPreviewLoading = false;
  let torrcPreviewError: string | null = null;

  const normaliseOptionList = (
    options?: Array<{ code?: string; name?: string }>,
    fallback: CountryOption[] = cloneCountryOptions(),
  ): CountryOption[] => {
    if (!options || options.length === 0) {
      return fallback;
    }
    const list: CountryOption[] = [];
    for (const option of options) {
      const code = normaliseCountryCode(option?.code ?? null);
      if (!code) continue;
      const name = option?.name ?? getCountryLabel(code) ?? code;
      list.push({ code, name });
    }
    return list.length > 0 ? list : fallback;
  };

  async function loadPresets() {
    try {
      const res = await fetch(presetURL);
      const data = await res.json();
      availableBridges = data.bridges ?? [];
      bridgePresets = data.presets ?? [];
      const baseOptions = normaliseOptionList(data.countries ?? data.exitCountries);
      entryOptions = data.entryCountries
        ? normaliseOptionList(data.entryCountries, baseOptions)
        : baseOptions;
      middleOptions = data.middleCountries
        ? normaliseOptionList(data.middleCountries, baseOptions)
        : baseOptions;
      exitOptions = data.exitCountries
        ? normaliseOptionList(data.exitCountries, baseOptions)
        : baseOptions;
      if (Array.isArray(data.fastCountries)) {
        const additional = data.fastCountries
          .map((entry: any) =>
            typeof entry === "string" ? entry : entry?.code ?? null,
          )
          .filter(Boolean) as Array<string | null | undefined>;
        baseFastCountryCodes = createFastCountrySet(additional);
      }
      void refreshTorrcPreview();
    } catch (e) {
      console.error("Failed to load presets", e);
    }
  }

  let selectedBridges: string[] = [];
  let selectedPreset: string | null = null;
  let workerList: string[] = [];
  let newWorker = "";
  let workerToken = "";
  let insecureHosts: string[] = [];
  let newInsecureHost = "";
  let pendingInsecureHost: string | null = null;
  let showInsecureWarning = false;
  let maxLogLines = 1000;
  let updateInterval = 86400;
  let entrySelection = "";
  let middleSelection = "";
  let exitSelection = "";
  let hsmLib: string | null = null;
  let hsmSlot: number | null = null;
  let geoipPath: string | null = null;
  let filePicker: HTMLInputElement | null = null;
  $: importProgress = $uiStore.importProgress;

  $: fastCountryCodes = createFastCountrySet([
    ...baseFastCountryCodes,
    ...customFastSelection,
  ]);

  $: fastOverrideOptions = COUNTRY_OPTIONS.filter(
    (option) => !customFastSelection.includes(option.code),
  );

  export let show: boolean;

  const dispatch = createEventDispatcher();
  let showTorrcEditor = false;
  let showWorkerSetup = false;
  let closeButton: HTMLButtonElement | null = null;
  let modalEl: HTMLElement | null = null;
  let previouslyFocused: HTMLElement | null = null;

  const ROUTE_ROLES = ["Entry Node", "Middle Node", "Exit Node"] as const;

  $: if (show) {
    previouslyFocused = document.activeElement as HTMLElement;
    selectedBridges = [...$uiStore.settings.bridges];
    selectedPreset = $uiStore.settings.bridgePreset ?? null;
    workerList = [...$uiStore.settings.workerList];
    newWorker = "";
    workerToken = $uiStore.settings.workerToken;
    insecureHosts = [...$uiStore.settings.insecureAllowedHosts];
    newInsecureHost = "";
    pendingInsecureHost = null;
    showInsecureWarning = false;
    maxLogLines = $uiStore.settings.maxLogLines;
    updateInterval = $uiStore.settings.updateInterval;
    entrySelection = $uiStore.settings.entryCountry ?? "";
    middleSelection = $uiStore.settings.middleCountry ?? "";
    exitSelection = $uiStore.settings.exitCountry ?? "";
    hsmLib = $uiStore.settings.hsm_lib;
    hsmSlot = $uiStore.settings.hsm_slot;
    geoipPath = $uiStore.settings.geoipPath;
    showFastOnly = $uiStore.settings.fastRoutingOnly;
    customFastSelection = [...$uiStore.settings.preferredFastCountries];
    includeBridgesInTorrc = ($uiStore.settings.bridges?.length ?? 0) > 0;
    torrcPreview = null;
    torrcPreviewError = null;
    torrcPreviewLoading = false;
    if (availableBridges.length === 0) loadPresets();
    tick().then(() => {
      closeButton && closeButton.focus();
      void refreshTorrcPreview();
    });
  } else if (previouslyFocused) {
    tick().then(() => previouslyFocused && previouslyFocused.focus());
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      dispatch("close");
    }
  }

  function trapFocus(event: KeyboardEvent) {
    if (event.key !== "Tab" || !modalEl) return;
    const focusable = Array.from(
      modalEl.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      ),
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  const optionsForSelect = (base: CountryOption[], selection: string) => {
    const code = normaliseCountryCode(selection);
    const filtered = showFastOnly
      ? base.filter((option) => fastCountryCodes.has(option.code))
      : base;
    if (!code) {
      return filtered;
    }
    if (filtered.some((option) => option.code === code)) {
      return filtered;
    }
    const fallback = base.find((option) => option.code === code);
    return fallback ? [fallback, ...filtered] : filtered;
  };

  function updateEntrySelection(value: string) {
    entrySelection = normaliseCountryCode(value) ?? "";
  }

  function updateMiddleSelection(value: string) {
    middleSelection = normaliseCountryCode(value) ?? "";
  }

  function updateExitSelection(value: string) {
    exitSelection = normaliseCountryCode(value) ?? "";
  }

  function handleEntryChange(event: Event) {
    updateEntrySelection((event.currentTarget as HTMLSelectElement).value);
  }

  function handleMiddleChange(event: Event) {
    updateMiddleSelection((event.currentTarget as HTMLSelectElement).value);
  }

  function handleExitChange(event: Event) {
    updateExitSelection((event.currentTarget as HTMLSelectElement).value);
  }

  async function applyRoutePreferences() {
    const entry = normaliseCountryCode(entrySelection);
    const middle = normaliseCountryCode(middleSelection);
    const exit = normaliseCountryCode(exitSelection);
    const current = get(uiStore).settings;
    const currentEntry = normaliseCountryCode(current.entryCountry);
    const currentMiddle = normaliseCountryCode(current.middleCountry);
    const currentExit = normaliseCountryCode(current.exitCountry);
    const payload: {
      entry?: string | null;
      middle?: string | null;
      exit?: string | null;
    } = {};
    if (entry !== currentEntry) payload.entry = entry;
    if (middle !== currentMiddle) payload.middle = middle;
    if (exit !== currentExit) payload.exit = exit;
    if (Object.keys(payload).length === 0) {
      return;
    }
    await uiStore.actions.setCircuitCountries(payload);
    void refreshTorrcPreview();
  }

  function clearRoutePreferences() {
    entrySelection = "";
    middleSelection = "";
    exitSelection = "";
    void applyRoutePreferences();
  }

  function applyDefaultRoute() {
    const defaults = ensureUniqueRoute([...DEFAULT_ROUTE_CODES]);
    entrySelection = defaults[0] ?? "";
    middleSelection = defaults[1] ?? "";
    exitSelection = defaults[2] ?? "";
    void applyRoutePreferences();
  }

  function onFastOnlyToggle(event: Event) {
    const target = event.target as HTMLInputElement;
    showFastOnly = target.checked;
    void uiStore.actions.setFastRoutingOnly(showFastOnly);
    void refreshTorrcPreview();
  }

  function addFastOverride() {
    const code = normaliseCountryCode(fastOverrideCandidate);
    if (!code) return;
    if (!customFastSelection.includes(code)) {
      customFastSelection = [...customFastSelection, code];
      void refreshTorrcPreview();
    }
    fastOverrideCandidate = "";
  }

  function removeFastOverride(code: string) {
    customFastSelection = customFastSelection.filter((entry) => entry !== code);
    void refreshTorrcPreview();
  }

  async function saveFastOverrides() {
    await uiStore.actions.savePreferredFastCountries(customFastSelection);
    void refreshTorrcPreview();
  }

  async function refreshTorrcPreview() {
    if (!show) return;
    torrcPreviewLoading = true;
    torrcPreviewError = null;
    try {
      torrcPreview = await invoke<TorrcProfile>("generate_torrc_profile", {
        fastOnly: showFastOnly,
        preferredFastCountries: customFastSelection,
        includeBridges: includeBridgesInTorrc,
      });
    } catch (err) {
      torrcPreview = null;
      torrcPreviewError =
        err instanceof Error ? err.message : "Failed to generate torrc profile";
    } finally {
      torrcPreviewLoading = false;
    }
  }

  function applyTorrcPreviewConfig() {
    if (!torrcPreview) return;
    void uiStore.actions.saveTorrcConfig(torrcPreview.config);
  }

  async function copyTorrcPreview() {
    if (!torrcPreview) return;
    try {
      await navigator.clipboard.writeText(torrcPreview.config);
    } catch (err) {
      torrcPreviewError =
        err instanceof Error ? err.message : "Unable to copy torrc to clipboard";
    }
  }

  function toggleIncludeBridges(event: Event) {
    const target = event.target as HTMLInputElement;
    includeBridgesInTorrc = target.checked;
    void refreshTorrcPreview();
  }

  $: normalizedSelections = [
    normaliseCountryCode(entrySelection),
    normaliseCountryCode(middleSelection),
    normaliseCountryCode(exitSelection),
  ] as Array<string | null>;

  $: previewRoute = ensureUniqueRoute(normalizedSelections);

  $: routeSummary = previewRoute.map((code, index) => {
    const requested = normalizedSelections[index];
    return {
      role: ROUTE_ROLES[index],
      code,
      flag: getCountryFlag(code),
      label: getCountryLabel(code),
      status: requested ? (requested === code ? "locked" : "fallback") : "auto",
      requestedLabel: requested ? getCountryLabel(requested) : null,
      isFast: isFastCountry(code, fastCountryCodes),
    };
  });

  function saveWorkers() {
    const list = workerList
      .map((l) => l.trim())
      .filter((l) => l.length > 0);
    uiStore.actions.saveWorkerConfig(list, workerToken);
  }

  function addWorker() {
    const url = newWorker.trim();
    if (url && !workerList.includes(url)) {
      workerList = [...workerList, url];
      newWorker = "";
    }
  }

  function removeWorker(index: number) {
    workerList = workerList.filter((_, i) => i !== index);
  }

  function requestAddInsecureHost() {
    const host = newInsecureHost.trim();
    if (!host) return;
    if (insecureHosts.includes(host.toLowerCase())) {
      newInsecureHost = "";
      return;
    }
    pendingInsecureHost = host;
    showInsecureWarning = true;
  }

  async function confirmInsecureHost() {
    if (!pendingInsecureHost) {
      showInsecureWarning = false;
      return;
    }
    await uiStore.actions.addInsecureHost(pendingInsecureHost);
    insecureHosts = [...$uiStore.settings.insecureAllowedHosts];
    newInsecureHost = "";
    pendingInsecureHost = null;
    showInsecureWarning = false;
  }

  function cancelInsecureHost() {
    pendingInsecureHost = null;
    showInsecureWarning = false;
  }

  async function removeInsecureHost(host: string) {
    await uiStore.actions.removeInsecureHost(host);
    insecureHosts = [...$uiStore.settings.insecureAllowedHosts];
  }

  async function importFile(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;
    const text = await input.files[0].text();
    const { workers } = parseWorkerList(text);
    workerList = workers;
    await uiStore.actions.importWorkersFromText(text);
  }

  function exportFile() {
    const list = uiStore.actions.exportWorkerList();
    const blob = new Blob([list.join("\n")], {
      type: "text/plain",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "workers.txt";
    a.click();
    URL.revokeObjectURL(url);
  }

  function saveLogLimit() {
    const limit = parseInt(String(maxLogLines));
    if (!isNaN(limit) && limit > 0) {
      uiStore.actions.setLogLimit(limit);
    }
  }

  function saveUpdateInterval() {
    const val = parseInt(String(updateInterval));
    if (!isNaN(val) && val >= 0) {
      uiStore.actions.saveUpdateInterval(val);
    }
  }

  function saveGeoipPath() {
    uiStore.actions.saveGeoipPath(geoipPath);
  }

  function saveHsm() {
    const slotNum = hsmSlot === null ? null : Number(hsmSlot);
    uiStore.actions.saveHsmConfig(hsmLib, isNaN(slotNum as number) ? null : slotNum);
  }

  async function applyPreset() {
    const preset = bridgePresets.find((p) => p.name === selectedPreset);
    if (preset) {
      await uiStore.actions.setBridgePreset(preset.name, preset.bridges);
      includeBridgesInTorrc = true;
      void refreshTorrcPreview();
    }
  }

  async function applyBridgeSelection() {
    await uiStore.actions.setBridges(selectedBridges);
    includeBridgesInTorrc = selectedBridges.length > 0;
    void refreshTorrcPreview();
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

{#if show}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    role="button"
    tabindex="0"
    aria-label="Close settings"
    on:click={() => dispatch('close')}
    on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && dispatch('close')}
    on:keydown={trapFocus}
  >
    <section
      class="glass-md rounded-2xl w-[90%] max-w-2xl min-h-[500px] p-6 flex flex-col"
      on:pointerdown|stopPropagation
      bind:this={modalEl}
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-modal-title"
      >
      <div class="flex justify-between items-center mb-4 shrink-0">
        <h2 id="settings-modal-title" class="text-2xl font-semibold">
          Settings
        </h2>
        <button
          class="text-gray-100 hover:text-white transition-colors"
          on:click={() => dispatch('close')}
          aria-label="Close settings"
          bind:this={closeButton}
        >
          <X size={24} />
        </button>
      </div>
      <div class="overflow-y-auto flex-grow">
        <!-- Torrc Configuration -->
        <div class="mb-8">
          <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">
            Torrc Configuration
          </h3>
          <button
            class="text-sm py-2 px-4 rounded-xl border-transparent font-medium flex items-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
            on:click={() => (showTorrcEditor = true)}
            aria-label="Edit torrc"
          >
            <Edit3 size={16} /> Edit torrc
          </button>
        </div>

        <div class="mb-8">
          <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">
            Bridge Preset
          </h3>
          <select
            class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm"
            bind:value={selectedPreset}
            aria-label="Bridge preset"
          >
            <option value="">Custom</option>
            {#each bridgePresets as p}
              <option value={p.name}>{p.name}</option>
            {/each}
          </select>
          {#if selectedPreset}
            <ul class="text-sm mt-2">
              {#each bridgePresets.find((b) => b.name === selectedPreset)?.bridges ?? [] as line}
                <li>{line}</li>
              {/each}
            </ul>
            <button
              class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
              on:click={applyPreset}
              aria-label="Apply preset"
            >
              Apply Preset
            </button>
          {/if}
        </div>

        <div class="mb-8">
          <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">
            Bridges
          </h3>
          <p class="text-sm text-gray-100 mb-4">
            Select one or more bridges to use for connecting.
          </p>
          {#each availableBridges as bridge}
            <label class="flex items-center gap-2 mb-2">
          <input
            type="checkbox"
            value={bridge}
            bind:group={selectedBridges}
            aria-label={bridge}
          />
              <span class="text-sm">{bridge}</span>
            </label>
          {/each}
          <button
            class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
            on:click={applyBridgeSelection}
            aria-label="Apply bridge selection"
          >
            Apply
          </button>
        </div>

        <div class="mb-8">
          <div class="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
            <h3 class="text-lg font-semibold border-b border-white/10 pb-2 md:border-none md:pb-0">
              Circuit Routing Preferences
            </h3>
            <label class="flex items-center gap-2 text-xs text-gray-200">
              <input
                type="checkbox"
                checked={showFastOnly}
                class="h-4 w-4 rounded border-white/40 bg-black/50 text-blue-400 focus:ring-blue-400/50"
                aria-label="Show only fast-tier countries"
                on:change={onFastOnlyToggle}
              />
              Show fast-tier countries only
            </label>
          </div>
          <p class="text-sm text-gray-100 mt-2">
            Pin guard, middle, and exit relays to specific countries. Leave a field on “Auto” to let Tor choose the best option.
          </p>
          <div class="mt-4 grid gap-3 md:grid-cols-3">
            <div>
              <label class="block text-xs text-gray-300 mb-1" for="entry-country-select">Entry (Guard)</label>
              <select
                id="entry-country-select"
                class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm"
                bind:value={entrySelection}
                on:change={handleEntryChange}
                aria-label="Preferred entry country"
              >
                <option value="">Auto</option>
                {#each optionsForSelect(entryOptions, entrySelection) as option (option.code)}
                  <option value={option.code}>
                    {getCountryFlag(option.code)} {option.name}{isFastCountry(option.code, fastCountryCodes) ? " • Fast" : ""}
                  </option>
                {/each}
              </select>
            </div>
            <div>
              <label class="block text-xs text-gray-300 mb-1" for="middle-country-select">Middle</label>
              <select
                id="middle-country-select"
                class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm"
                bind:value={middleSelection}
                on:change={handleMiddleChange}
                aria-label="Preferred middle country"
              >
                <option value="">Auto</option>
                {#each optionsForSelect(middleOptions, middleSelection) as option (option.code)}
                  <option value={option.code}>
                    {getCountryFlag(option.code)} {option.name}{isFastCountry(option.code, fastCountryCodes) ? " • Fast" : ""}
                  </option>
                {/each}
              </select>
            </div>
            <div>
              <label class="block text-xs text-gray-300 mb-1" for="exit-country-select">Exit</label>
              <select
                id="exit-country-select"
                class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm"
                bind:value={exitSelection}
                on:change={handleExitChange}
                aria-label="Preferred exit country"
              >
                <option value="">Auto</option>
                {#each optionsForSelect(exitOptions, exitSelection) as option (option.code)}
                  <option value={option.code}>
                    {getCountryFlag(option.code)} {option.name}{isFastCountry(option.code, fastCountryCodes) ? " • Fast" : ""}
                  </option>
                {/each}
              </select>
            </div>
          </div>
          <div class="mt-4 flex flex-wrap gap-2">
            <button
              class="text-sm py-2 px-4 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
              on:click={applyRoutePreferences}
              aria-label="Save circuit routing preferences"
            >
              Save Route
            </button>
            <button
              class="text-sm py-2 px-4 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-emerald-500/20 text-emerald-300 hover:bg-emerald-500/30"
              on:click={applyDefaultRoute}
              aria-label="Apply recommended fast route"
            >
              Recommended Fast Route
            </button>
            <button
              class="text-sm py-2 px-4 rounded-xl border border-white/20 font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto text-gray-100 hover:bg-white/10"
              on:click={clearRoutePreferences}
              aria-label="Clear pinned countries"
            >
              Clear Pins
            </button>
          </div>
          <div class="mt-4 grid gap-3 sm:grid-cols-3">
            {#each routeSummary as detail (detail.role)}
              <div
                class="bg-black/40 border border-white/10 rounded-xl p-3"
                title={`Effective ${detail.role.toLowerCase()}: ${detail.label}`}
              >
                <div class="flex items-center justify-between text-[11px] uppercase tracking-wide text-gray-300">
                  <span>{detail.role}</span>
                  <span aria-hidden="true">{detail.flag}</span>
                </div>
                <p class="text-sm text-white font-semibold mt-1">{detail.label}</p>
                {#if detail.status === 'locked'}
                  <p class="text-[11px] text-emerald-300 mt-1">Pinned</p>
                {:else if detail.status === 'fallback'}
                  <p class="text-[11px] text-amber-300 mt-1">
                    Fallback{#if detail.requestedLabel} from {detail.requestedLabel}{/if}
                  </p>
                {:else}
                  <p class="text-[11px] text-slate-300 mt-1">Automatic</p>
                {/if}
                {#if detail.isFast}
                  <p class="text-[10px] text-sky-300 mt-1">Fast-tier relay</p>
                {/if}
              </div>
            {/each}
          </div>
          <div class="mt-6 grid gap-4 lg:grid-cols-2">
            <div class="bg-black/40 border border-white/10 rounded-xl p-4">
              <h4 class="text-sm font-semibold text-white">Fast-tier overrides</h4>
              <p class="text-xs text-gray-300 mt-1">
                Add extra countries to treat as fast-tier when filtering and generating the torrc profile.
              </p>
              <div class="mt-3 flex gap-2">
                <select
                  class="flex-1 bg-black/50 rounded border border-white/20 p-2 text-sm"
                  bind:value={fastOverrideCandidate}
                  aria-label="Add fast-tier country"
                >
                  <option value="">Select country</option>
                  {#each fastOverrideOptions as option (option.code)}
                    <option value={option.code}>
                      {getCountryFlag(option.code)} {option.name}
                    </option>
                  {/each}
                </select>
                <button
                  type="button"
                  class="text-xs py-2 px-3 rounded-xl border-transparent font-medium bg-blue-500/20 text-blue-300 hover:bg-blue-500/30 transition-all"
                  on:click={addFastOverride}
                  aria-label="Add fast-tier override"
                  disabled={!fastOverrideCandidate}
                >
                  Add
                </button>
              </div>
              <div class="mt-3 flex flex-wrap gap-2 min-h-[2rem]">
                {#if customFastSelection.length === 0}
                  <p class="text-xs text-gray-400">No overrides saved yet.</p>
                {:else}
                  {#each customFastSelection as code (code)}
                    <span class="inline-flex items-center gap-1 bg-blue-500/10 border border-blue-500/30 text-blue-200 text-xs px-2 py-1 rounded-full">
                      <span aria-hidden="true">{getCountryFlag(code)}</span>
                      <span>{getCountryLabel(code)}</span>
                      <button
                        type="button"
                        class="text-[10px] uppercase tracking-wide hover:text-red-300"
                        on:click={() => removeFastOverride(code)}
                        aria-label={`Remove ${getCountryLabel(code)} from fast-tier overrides`}
                      >
                        ✕
                      </button>
                    </span>
                  {/each}
                {/if}
              </div>
              <div class="mt-3 flex gap-2">
                <button
                  type="button"
                  class="text-xs py-2 px-3 rounded-xl border-transparent font-medium bg-emerald-500/20 text-emerald-200 hover:bg-emerald-500/30 transition-all"
                  on:click={saveFastOverrides}
                  aria-label="Save fast-tier overrides"
                  disabled={torrcPreviewLoading}
                >
                  Save overrides
                </button>
                <button
                  type="button"
                  class="text-xs py-2 px-3 rounded-xl border border-white/20 text-gray-100 hover:bg-white/10 transition-all"
                  on:click={() => {
                    customFastSelection = [];
                    void saveFastOverrides();
                  }}
                  aria-label="Reset fast-tier overrides"
                  disabled={customFastSelection.length === 0 || torrcPreviewLoading}
                >
                  Reset
                </button>
              </div>
            </div>
            <div class="bg-black/40 border border-white/10 rounded-xl p-4">
              <h4 class="text-sm font-semibold text-white">Torrc generator</h4>
              <p class="text-xs text-gray-300 mt-1">
                Generate a torrc fragment that respects your pinned countries and fast-tier policy. Apply it directly or copy it for manual review.
              </p>
              <label class="mt-3 flex items-center gap-2 text-xs text-gray-200">
                <input
                  type="checkbox"
                  checked={includeBridgesInTorrc}
                  class="h-4 w-4 rounded border-white/40 bg-black/50 text-blue-400 focus:ring-blue-400/50"
                  on:change={toggleIncludeBridges}
                  aria-label="Include saved bridges in torrc preview"
                />
                Include configured bridges
              </label>
              <div class="mt-3">
                {#if torrcPreviewLoading}
                  <div class="text-xs text-gray-300 italic">Calculating recommended torrc…</div>
                {:else}
                  <textarea
                    class="w-full bg-black/60 rounded border border-white/20 p-2 text-xs font-mono min-h-[160px]"
                    readonly
                    aria-label="Recommended torrc configuration"
                  >{torrcPreview?.config ?? ""}</textarea>
                {/if}
              </div>
              {#if torrcPreviewError}
                <p class="text-xs text-amber-300 mt-2">{torrcPreviewError}</p>
              {/if}
              <div class="mt-3 flex flex-wrap gap-2">
                <button
                  type="button"
                  class="text-xs py-2 px-3 rounded-xl border border-white/20 text-gray-100 hover:bg-white/10 transition-all"
                  on:click={refreshTorrcPreview}
                  aria-label="Refresh torrc preview"
                >
                  Refresh
                </button>
                <button
                  type="button"
                  class="text-xs py-2 px-3 rounded-xl border-transparent font-medium bg-blue-500/20 text-blue-200 hover:bg-blue-500/30 transition-all disabled:opacity-40"
                  on:click={applyTorrcPreviewConfig}
                  aria-label="Apply torrc preview"
                  disabled={!torrcPreview}
                >
                  Apply to torrc
                </button>
                <button
                  type="button"
                  class="text-xs py-2 px-3 rounded-xl border-transparent font-medium bg-slate-500/20 text-slate-200 hover:bg-slate-500/30 transition-all disabled:opacity-40"
                  on:click={copyTorrcPreview}
                  aria-label="Copy torrc preview"
                  disabled={!torrcPreview}
                >
                  Copy
                </button>
              </div>
              {#if torrcPreview}
                <p class="text-[11px] text-gray-300 mt-2">
                  Route {torrcPreview.entry} → {torrcPreview.middle} → {torrcPreview.exit}
                  {#if torrcPreview.fast_only}
                    • fast-tier enforced
                  {/if}
                </p>
                {#if torrcPreview.bridges.length > 0}
                  <p class="text-[11px] text-gray-400 mt-1">
                    Includes {torrcPreview.bridges.length} bridge{torrcPreview.bridges.length === 1 ? "" : "s"}.
                  </p>
                {/if}
              {/if}
            </div>
          </div>
        </div>

        <div class="mb-8">
          <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">
            Worker List
          </h3>
          {#each workerList as w, i}
            <div class="flex items-center gap-2 mb-2">
              <input
                type="text"
                class="flex-grow bg-black/50 rounded border border-white/20 p-2 text-sm"
                bind:value={workerList[i]}
                aria-label={`Worker URL ${i}`}
              />
              <button
                class="p-1 rounded hover:bg-red-600/40"
                on:click={() => removeWorker(i)}
                aria-label="Remove worker"
              >
                <X size={16} />
              </button>
            </div>
          {/each}
          <div class="flex items-center gap-2 mb-2">
            <input
              type="text"
              class="flex-grow bg-black/50 rounded border border-white/20 p-2 text-sm"
              placeholder="https://proxy.example.com"
              bind:value={newWorker}
              aria-label="New worker URL"
            />
            <button
              class="p-1 rounded hover:bg-green-600/40"
              on:click={addWorker}
              aria-label="Add worker"
            >
              <Plus size={16} />
            </button>
          </div>
          <input
            type="file"
            accept="text/*"
            class="hidden"
            on:change={importFile}
            bind:this={filePicker}
          />
          <button
            class="text-sm py-2 px-4 mb-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
          on:click={() => filePicker && filePicker.click()}
          aria-label="Import worker list"
        >
          Import Worker List
        </button>
        {#if importProgress !== null}
          <div
            class="w-full bg-gray-700/50 rounded-full h-2 mt-1"
            role="progressbar"
            aria-valuemin="0"
            aria-valuemax="100"
            aria-valuenow={importProgress}
          >
            <div
              class="bg-white h-2 rounded-full transition-all duration-500 ease-out"
              style="width: {importProgress}%"
            ></div>
          </div>
          <p class="text-xs mt-1">{importProgress}%</p>
        {/if}
        <button
            class="text-sm py-2 px-4 mb-2 ml-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
            on:click={exportFile}
            aria-label="Export worker list"
          >
            Export Worker List
          </button>
          <input
            type="text"
            class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm mt-2"
            bind:value={workerToken}
            placeholder="Worker token"
            aria-label="Worker token"
          />
          <button
            class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
            on:click={saveWorkers}
            aria-label="Save worker list"
          >
            Save
          </button>
          <p class="text-xs text-gray-200 mt-2">Multiple workers improve reliability.</p>
          <button
            class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
            on:click={() => (showWorkerSetup = true)}
            aria-label="Open worker setup help"
          >
            Worker Setup Help
          </button>
        </div>

        <div class="mb-8">
          <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">
            Insecure HTTP Allowlist
          </h3>
          <p class="text-sm text-gray-200 mb-3">
            Only add diagnostic endpoints that require HTTP. Traffic to these hosts is not encrypted.
          </p>
          {#if insecureHosts.length === 0}
            <p class="text-xs text-gray-300 italic mb-3">No insecure HTTP hosts are currently allowed.</p>
          {/if}
          {#each insecureHosts as host}
            <div class="flex items-center gap-2 mb-2">
              <span class="flex-grow bg-black/40 border border-white/15 rounded px-3 py-2 text-sm">
                {host}
              </span>
              <button
                class="p-1 rounded hover:bg-red-600/40"
                on:click={() => removeInsecureHost(host)}
                aria-label={`Remove insecure host ${host}`}
              >
                <X size={16} />
              </button>
            </div>
          {/each}
          <div class="flex items-center gap-2 mb-2">
            <input
              type="text"
              class="flex-grow bg-black/50 rounded border border-white/20 p-2 text-sm"
              placeholder="127.0.0.1"
              bind:value={newInsecureHost}
              aria-label="New insecure host"
            />
            <button
              class="p-1 rounded hover:bg-amber-500/40"
              on:click={requestAddInsecureHost}
              aria-label="Add insecure host"
            >
              <Plus size={16} />
            </button>
          </div>
          <p class="text-xs text-amber-200">
            Use HTTPS whenever possible. HTTP entries should be limited to localhost diagnostics.
          </p>
        </div>

        <div class="mb-8">
          <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">
            Max Log Lines
          </h3>
          <input
            type="number"
            min="1"
            class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm"
            bind:value={maxLogLines}
            aria-label="Maximum log lines"
          />
          <button
            class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
            on:click={saveLogLimit}
            aria-label="Save log limit"
          >
            Save
          </button>
        </div>

        <div class="mb-8">
          <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">
            Update Interval
          </h3>
          <input
            type="number"
            min="0"
            class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm"
            bind:value={updateInterval}
            aria-label="Update interval"
          />
          <button
            class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
            on:click={saveUpdateInterval}
            aria-label="Save update interval"
          >
            Save
          </button>
        </div>

        <div class="mb-8">
          <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">
            GeoIP Directory
          </h3>
          <input
            type="text"
            class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm"
            bind:value={geoipPath}
            placeholder="/path/to/geoip_dir"
            aria-label="GeoIP directory"
          />
          <button
            class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
            on:click={saveGeoipPath}
            aria-label="Save GeoIP directory"
          >
            Save
          </button>
        </div>

        <div class="mb-8">
          <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">
            HSM Configuration
          </h3>
          <input
            type="text"
            class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm mb-2"
            placeholder="/usr/lib/softhsm/libsofthsm2.so"
            bind:value={hsmLib}
            aria-label="HSM library path"
          />
          <input
            type="number"
            min="0"
            class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm"
            placeholder="0"
            bind:value={hsmSlot}
            aria-label="HSM slot"
          />
          <button
            class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
            on:click={saveHsm}
            aria-label="Save HSM configuration"
          >
            Save
          </button>
        </div>

        <!-- Worker Management section has been removed as it was placeholder functionality. -->
      </div>
    </section>
  </div>
{/if}

<!-- TORRC Editor Modal -->
<TorrcEditorModal
  bind:show={showTorrcEditor}
  on:close={() => (showTorrcEditor = false)}
/>

<!-- Worker Setup Help Modal -->
<WorkerSetupModal
  bind:show={showWorkerSetup}
  on:close={() => (showWorkerSetup = false)}
/>

{#if showInsecureWarning}
  <div
    class="fixed inset-0 bg-black/60 flex items-center justify-center z-60"
    role="dialog"
    aria-modal="true"
    aria-label="Confirm insecure host"
  >
    <div class="glass-md rounded-2xl max-w-md w-[90%] p-6 space-y-4">
      <h3 class="text-xl font-semibold text-amber-200">Allow insecure HTTP?</h3>
      <p class="text-sm text-gray-100">
        Adding <span class="font-mono">{pendingInsecureHost ?? ""}</span> to the allowlist permits unencrypted HTTP
        requests. Only continue if this host is a trusted local diagnostic endpoint.
      </p>
      <div class="flex justify-end gap-3">
        <button
          class="px-4 py-2 rounded-xl border border-white/20 text-gray-100 hover:bg-white/10 transition"
          on:click={cancelInsecureHost}
        >
          Cancel
        </button>
        <button
          class="px-4 py-2 rounded-xl border-transparent font-semibold bg-amber-500/30 text-amber-100 hover:bg-amber-500/50 transition"
          on:click={confirmInsecureHost}
        >
          Allow HTTP
        </button>
      </div>
    </div>
  </div>
{/if}

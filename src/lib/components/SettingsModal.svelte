<script lang="ts">
  import { createEventDispatcher, tick } from "svelte";
  import { X, Edit3, Plus } from "lucide-svelte";
  import { uiStore } from "$lib/stores/uiStore";
  import TorrcEditorModal from './TorrcEditorModal.svelte';
  import WorkerSetupModal from './WorkerSetupModal.svelte';
  import { parseWorkerList } from '../../../scripts/import_workers';

  const presetURL = new URL('../bridge_presets.json', import.meta.url).href;

  let availableBridges: string[] = [];
  let bridgePresets: { name: string; bridges: string[] }[] = [];
  let exitCountries: { code: string; name: string }[] = [];

  async function loadPresets() {
    try {
      const res = await fetch(presetURL);
      const data = await res.json();
      availableBridges = data.bridges ?? [];
      bridgePresets = data.presets ?? [];
      exitCountries = data.exitCountries ?? [];
    } catch (e) {
      console.error('Failed to load presets', e);
    }
  }

  let selectedBridges: string[] = [];
  let selectedPreset: string | null = null;
  let workerList: string[] = [];
  let newWorker = "";
  let workerToken = "";
  let maxLogLines = 1000;
  let updateInterval = 86400;
  let exitCountry: string | null = null;
  let hsmLib: string | null = null;
  let hsmSlot: number | null = null;
  let geoipPath: string | null = null;
  let filePicker: HTMLInputElement | null = null;
  $: importProgress = $uiStore.importProgress;

  export let show: boolean;

  const dispatch = createEventDispatcher();
  let showTorrcEditor = false;
  let showWorkerSetup = false;
  let closeButton: HTMLButtonElement | null = null;
  let modalEl: HTMLElement | null = null;
  let previouslyFocused: HTMLElement | null = null;

  $: if (show) {
    previouslyFocused = document.activeElement as HTMLElement;
    selectedBridges = [...$uiStore.settings.bridges];
    selectedPreset = $uiStore.settings.bridgePreset ?? null;
    workerList = [...$uiStore.settings.workerList];
    newWorker = "";
    workerToken = $uiStore.settings.workerToken;
    maxLogLines = $uiStore.settings.maxLogLines;
    updateInterval = $uiStore.settings.updateInterval;
    exitCountry = $uiStore.settings.exitCountry ?? null;
    hsmLib = $uiStore.settings.hsm_lib;
    hsmSlot = $uiStore.settings.hsm_slot;
    geoipPath = $uiStore.settings.geoipPath;
    uiStore.actions.setExitCountry(exitCountry);
    if (availableBridges.length === 0) loadPresets();
    tick().then(() => closeButton && closeButton.focus());
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
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      )
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
    const blob = new Blob([list.join('\n')], {
      type: 'text/plain',
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'workers.txt';
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

  function saveExitCountry() {
    uiStore.actions.setExitCountry(exitCountry);
  }

  function saveHsm() {
    const slotNum = hsmSlot === null ? null : Number(hsmSlot);
    uiStore.actions.saveHsmConfig(hsmLib, isNaN(slotNum as number) ? null : slotNum);
  }

  function applyPreset() {
    const preset = bridgePresets.find((p) => p.name === selectedPreset);
    if (preset) {
      uiStore.actions.setBridgePreset(preset.name, preset.bridges);
    }
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
      class="glass-lg rounded-2xl w-[90%] max-w-2xl min-h-[500px] p-6 flex flex-col"
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
            on:click={() => uiStore.actions.setBridges(selectedBridges)}
            aria-label="Apply bridge selection"
          >
            Apply
          </button>
        </div>

        <div class="mb-8">
          <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">
            Preferred Exit Country
          </h3>
          <select
            class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm"
            bind:value={exitCountry}
            aria-label="Exit country"
          >
            <option value="">Auto</option>
            {#each exitCountries as c}
              <option value={c.code}>{c.name}</option>
            {/each}
          </select>
          <button
            class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
            on:click={saveExitCountry}
            aria-label="Save exit country"
          >
            Save
          </button>
          {#if !exitCountry}
            <p class="text-xs text-yellow-400 mt-2">
              No exit country selected. A random exit node will be used.
            </p>
          {/if}
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

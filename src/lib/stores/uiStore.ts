import { writable, get } from "svelte/store";
import { db } from "$lib/database";
import type { Settings } from "$lib/database";
import { invoke } from "$lib/api";
import { normaliseCountryCode } from "$lib/utils/countries";

type AppSettings = {
  workerList: string[];
  torrcConfig: string;
  workerToken: string;
  exitCountry: string | null;
  entryCountry: string | null;
  middleCountry: string | null;
  bridges: string[];
  bridgePreset: string | null;
  maxLogLines: number;
  hsm_lib: string | null;
  hsm_slot: number | null;
  updateInterval: number;
  geoipPath: string | null;
  fastRoutingOnly: boolean;
  preferredFastCountries: string[];
};

type UIState = {
  isLogsModalOpen: boolean;
  isSettingsModalOpen: boolean;
  cloudflareEnabled: boolean;
  settings: AppSettings;
  error: string | null;
  importProgress: number | null;
};

function createUIStore() {
  const { subscribe, update, set } = writable<UIState>({
    isLogsModalOpen: false,
    isSettingsModalOpen: false,
    cloudflareEnabled: false,
    settings: {
      workerList: ["worker1.example.com", "worker2.example.com"], // Default values
      torrcConfig: "# Default torrc config\n",
      workerToken: "",
      exitCountry: null,
      entryCountry: null,
      middleCountry: null,
      bridges: [],
      bridgePreset: null,
      maxLogLines: 1000,
      hsm_lib: null,
      hsm_slot: null,
      updateInterval: 86400,
      geoipPath: null,
      fastRoutingOnly: false,
      preferredFastCountries: [],
    },
    error: null,
    importProgress: null,
  });

  const persistCircuitCountries = async (
    countries: {
      entry?: string | null;
      middle?: string | null;
      exit?: string | null;
    },
  ) => {
    const current = get({ subscribe });
    const next: AppSettings = {
      ...current.settings,
      entryCountry:
        countries.entry !== undefined ? countries.entry : current.settings.entryCountry,
      middleCountry:
        countries.middle !== undefined ? countries.middle : current.settings.middleCountry,
      exitCountry:
        countries.exit !== undefined ? countries.exit : current.settings.exitCountry,
    };

    try {
      if (countries.entry !== undefined) {
        await invoke("set_entry_country", { country: next.entryCountry ?? null });
      }
      if (countries.middle !== undefined) {
        await invoke("set_middle_country", { country: next.middleCountry ?? null });
      }
      if (countries.exit !== undefined) {
        await invoke("set_exit_country", { country: next.exitCountry ?? null });
      }
      await db.settings.put({ id: 1, ...next });
      update((state) => ({ ...state, settings: next, error: null }));
    } catch (err) {
      const message = err instanceof Error ? err.message : "Unknown error";
      update((state) => ({
        ...state,
        error: `Failed to set circuit countries: ${message}`,
      }));
    }
  };

  const actions = {
    toggleLogsModal: () =>
      update((state) => ({
        ...state,
        isLogsModalOpen: !state.isLogsModalOpen,
      })),
    toggleSettingsModal: () =>
      update((state) => ({
        ...state,
        isSettingsModalOpen: !state.isSettingsModalOpen,
      })),
    openLogsModal: () =>
      update((state) => ({ ...state, isLogsModalOpen: true })),
    closeLogsModal: () =>
      update((state) => ({ ...state, isLogsModalOpen: false })),
    openSettingsModal: () =>
      update((state) => ({ ...state, isSettingsModalOpen: true })),
    closeSettingsModal: () =>
      update((state) => ({ ...state, isSettingsModalOpen: false })),

    setCloudflareEnabled: (val: boolean) =>
      update((state) => ({ ...state, cloudflareEnabled: val })),

    loadSettings: async () => {
      try {
        const storedSettings = await db.settings.get(1);
        if (storedSettings) {
          update((state) => ({
            ...state,
            settings: {
              workerList: storedSettings.workerList,
              torrcConfig: storedSettings.torrcConfig,
              workerToken: storedSettings.workerToken ?? "",
              exitCountry: storedSettings.exitCountry ?? null,
              entryCountry: storedSettings.entryCountry ?? null,
              middleCountry: storedSettings.middleCountry ?? null,
              bridges: storedSettings.bridges ?? [],
              bridgePreset: storedSettings.bridgePreset ?? null,
              maxLogLines: storedSettings.maxLogLines ?? 1000,
              hsm_lib: storedSettings.hsm_lib ?? null,
              hsm_slot: storedSettings.hsm_slot ?? null,
              updateInterval: storedSettings.updateInterval ?? 86400,
              geoipPath: storedSettings.geoipPath ?? null,
              fastRoutingOnly: storedSettings.fastRoutingOnly ?? false,
              preferredFastCountries: storedSettings.preferredFastCountries ?? [],
            },
          }));

          // Apply settings to backend so configuration is used on start
          await invoke("set_bridges", {
            bridges: storedSettings.bridges ?? [],
          });
          await invoke("set_entry_country", {
            country: storedSettings.entryCountry ?? null,
          });
          await invoke("set_middle_country", {
            country: storedSettings.middleCountry ?? null,
          });
          await invoke("set_exit_country", {
            country: storedSettings.exitCountry ?? null,
          });
          await invoke("set_log_limit", {
            limit: storedSettings.maxLogLines ?? 1000,
          });
          await invoke("set_worker_config", {
            workers: storedSettings.workerList,
            token: storedSettings.workerToken ?? "",
          });
          await invoke("set_hsm_config", {
            lib: storedSettings.hsm_lib ?? null,
            slot: storedSettings.hsm_slot ?? null,
          });
          await invoke("set_update_interval", {
            interval: storedSettings.updateInterval ?? 86400,
          });
          await invoke("set_geoip_path", { path: storedSettings.geoipPath ?? null });
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to load settings: ${message}`,
        }));
      }
    },

    saveSettings: async (newSettings: AppSettings) => {
      try {
        await db.settings.put({ id: 1, ...newSettings });
        update((state) => ({ ...state, settings: newSettings, error: null }));
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to save settings: ${message}`,
        }));
      }
    },

    setCircuitCountries: persistCircuitCountries,
    setEntryCountry: async (country: string | null) =>
      persistCircuitCountries({ entry: country }),
    setMiddleCountry: async (country: string | null) =>
      persistCircuitCountries({ middle: country }),

    setBridges: async (bridges: string[]) => {
      try {
        await invoke("set_bridges", { bridges });
        const current = get({ subscribe });
        const newSettings: AppSettings = {
          ...current.settings,
          bridges,
          bridgePreset: null,
        };
        await db.settings.put({ id: 1, ...newSettings });
        update((state) => ({ ...state, settings: newSettings, error: null }));
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to set bridges: ${message}`,
        }));
      }
    },

    setFastRoutingOnly: async (fastOnly: boolean) => {
      try {
        const current = get({ subscribe });
        const newSettings: AppSettings = {
          ...current.settings,
          fastRoutingOnly: fastOnly,
        };
        await db.settings.put({ id: 1, ...newSettings });
        update((state) => ({ ...state, settings: newSettings, error: null }));
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to save fast routing preference: ${message}`,
        }));
      }
    },

    savePreferredFastCountries: async (countries: string[]) => {
      try {
        const normalised = Array.from(
          new Set(
            countries
              .map((code) => normaliseCountryCode(code))
              .filter((code): code is string => Boolean(code)),
          ),
        );
        const current = get({ subscribe });
        const newSettings: AppSettings = {
          ...current.settings,
          preferredFastCountries: normalised,
        };
        await db.settings.put({ id: 1, ...newSettings });
        update((state) => ({ ...state, settings: newSettings, error: null }));
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to save fast-tier overrides: ${message}`,
        }));
      }
    },

    setBridgePreset: async (preset: string | null, bridges: string[]) => {
      try {
        await invoke("set_bridges", { bridges });
        const current = get({ subscribe });
        const newSettings: AppSettings = {
          ...current.settings,
          bridges,
          bridgePreset: preset,
        };
        await db.settings.put({ id: 1, ...newSettings });
        update((state) => ({ ...state, settings: newSettings, error: null }));
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to set bridges: ${message}`,
        }));
      }
    },

    setExitCountry: async (country: string | null) => persistCircuitCountries({ exit: country }),

    loadTorrcConfig: async () => {
      try {
        const stored = await db.settings.get(1);
        if (stored) {
          update((state) => ({
            ...state,
            settings: {
              ...state.settings,
              torrcConfig: stored.torrcConfig,
            },
          }));
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to load torrc config: ${message}`,
        }));
      }
    },

    saveTorrcConfig: async (config: string) => {
      try {
        const current = get({ subscribe });
        const newSettings: AppSettings = {
          ...current.settings,
          torrcConfig: config,
        };
        await db.settings.put({ id: 1, ...newSettings });
        await invoke("set_torrc_config", { config });
        update((state) => ({ ...state, settings: newSettings, error: null }));
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to save torrc config: ${message}`,
        }));
      }
    },

    loadWorkerList: async () => {
      try {
        const stored = await db.settings.get(1);
        if (stored) {
          update((state) => ({
            ...state,
            settings: {
              ...state.settings,
              workerList: stored.workerList,
              workerToken: stored.workerToken ?? "",
            },
          }));
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to load worker list: ${message}`,
        }));
      }
    },

    saveWorkerConfig: async (workers: string[], token: string) => {
      try {
        const current = get({ subscribe });
        const newSettings: AppSettings = {
          ...current.settings,
          workerList: workers,
          workerToken: token,
        };
        await invoke("set_worker_config", { workers, token });
        const valid = await invoke<boolean>("validate_worker_token");
        if (!valid) {
          await invoke("set_worker_config", {
            workers: current.settings.workerList,
            token: current.settings.workerToken,
          });
          update((state) => ({
            ...state,
            error: "Invalid worker token",
          }));
          return;
        }
        await db.settings.put({ id: 1, ...newSettings });
        update((state) => ({ ...state, settings: newSettings, error: null }));
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to save worker config: ${message}`,
        }));
      }
    },

    saveHsmConfig: async (lib: string | null, slot: number | null) => {
      try {
        const current = get({ subscribe });
        const newSettings: AppSettings = {
          ...current.settings,
          hsm_lib: lib,
          hsm_slot: slot,
        };
        await invoke("set_hsm_config", { lib, slot });
        await db.settings.put({ id: 1, ...newSettings });
        update((state) => ({ ...state, settings: newSettings, error: null }));
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to save HSM config: ${message}`,
        }));
      }
    },

    addWorker: async (url: string) => {
      const current = get({ subscribe });
      if (!current.settings.workerList.includes(url)) {
        const workers = [...current.settings.workerList, url];
        await actions.saveWorkerConfig(workers, current.settings.workerToken);
      }
    },

    removeWorker: async (url: string) => {
      const current = get({ subscribe });
      const workers = current.settings.workerList.filter((w) => w !== url);
      await actions.saveWorkerConfig(workers, current.settings.workerToken);
    },

    setWorkerToken: async (token: string) => {
      const current = get({ subscribe });
      await actions.saveWorkerConfig(current.settings.workerList, token);
    },

    importWorkerList: async (workers: string[]) => {
      const current = get({ subscribe });
      await actions.saveWorkerConfig(workers, current.settings.workerToken);
    },

    exportWorkerList: () => {
      const current = get({ subscribe });
      return [...current.settings.workerList];
    },

    setImportProgress: (val: number | null) =>
      update((state) => ({ ...state, importProgress: val })),

    importWorkersFromText: async (text: string) => {
      const lines = text.split(/\r?\n/);
      const workers: string[] = [];
      const seen = new Set<string>();
      let processed = 0;
      const total = lines.filter((l) => l.trim().length > 0).length;
      actions.setImportProgress(0);
      for (const line of lines) {
        const url = line.trim();
        if (!url) {
          processed++;
          continue;
        }
        try {
          new URL(url);
          if (!seen.has(url)) {
            seen.add(url);
            workers.push(url);
          }
        } catch {
          // ignore invalid here, UI just imports valid entries
        }
        processed++;
        actions.setImportProgress(Math.round((processed / lines.length) * 100));
      }
      const current = get({ subscribe });
      await actions.saveWorkerConfig(workers, current.settings.workerToken);
      actions.setImportProgress(100);
      setTimeout(() => actions.setImportProgress(null), 500);
      return workers.length;
    },

    setLogLimit: async (limit: number) => {
      try {
        await invoke("set_log_limit", { limit });
        const current = get({ subscribe });
        const newSettings: AppSettings = {
          ...current.settings,
          maxLogLines: limit,
        };
        await db.settings.put({ id: 1, ...newSettings });
        update((state) => ({ ...state, settings: newSettings, error: null }));
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to set log limit: ${message}`,
        }));
      }
    },

    saveUpdateInterval: async (interval: number) => {
      try {
        await invoke("set_update_interval", { interval });
        const current = get({ subscribe });
        const newSettings: AppSettings = {
          ...current.settings,
          updateInterval: interval,
        };
        await db.settings.put({ id: 1, ...newSettings });
        update((state) => ({ ...state, settings: newSettings, error: null }));
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to set update interval: ${message}`,
        }));
      }
    },

    saveGeoipPath: async (path: string | null) => {
      try {
        await invoke("set_geoip_path", { path });
        const current = get({ subscribe });
        const newSettings: AppSettings = {
          ...current.settings,
          geoipPath: path,
        };
        await db.settings.put({ id: 1, ...newSettings });
        update((state) => ({ ...state, settings: newSettings, error: null }));
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to set geoip path: ${message}`,
        }));
      }
    },
  };

  // Load settings on initialization
  actions.loadSettings();

  return {
    subscribe,
    update,
    set,
    actions,
  };
}

export const uiStore = createUIStore();

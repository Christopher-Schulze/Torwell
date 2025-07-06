import { writable, get } from "svelte/store";
import { db } from "$lib/database";
import type { Settings } from "$lib/database";
import { invoke } from "@tauri-apps/api/tauri";

type AppSettings = {
  workerList: string[];
  torrcConfig: string;
  workerToken: string;
  exitCountry: string | null;
  bridges: string[];
  bridgePreset: string | null;
  maxLogLines: number;
};

type UIState = {
  isLogsModalOpen: boolean;
  isSettingsModalOpen: boolean;
  cloudflareEnabled: boolean;
  settings: AppSettings;
  error: string | null;
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
      bridges: [],
      bridgePreset: null,
      maxLogLines: 1000,
    },
    error: null,
  });

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
              bridges: storedSettings.bridges ?? [],
              bridgePreset: storedSettings.bridgePreset ?? null,
              maxLogLines: storedSettings.maxLogLines ?? 1000,
            },
          }));

          // Apply settings to backend so configuration is used on start
          await invoke("set_bridges", {
            bridges: storedSettings.bridges ?? [],
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

    setExitCountry: async (country: string | null) => {
      try {
        await invoke("set_exit_country", { country });
        const current = get({ subscribe });
        const newSettings: AppSettings = {
          ...current.settings,
          exitCountry: country,
        };
        await db.settings.put({ id: 1, ...newSettings });
        update((state) => ({ ...state, settings: newSettings, error: null }));
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        update((state) => ({
          ...state,
          error: `Failed to set exit country: ${message}`,
        }));
      }
    },

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

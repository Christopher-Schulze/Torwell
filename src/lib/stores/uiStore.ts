import { writable, get } from 'svelte/store';
import { db } from '$lib/database';
import type { Settings } from '$lib/database';
import { invoke } from '@tauri-apps/api/tauri';

type AppSettings = {
    workerList: string[];
    torrcConfig: string;
    exitCountry: string | null;
    bridges: string[];
};

type UIState = {
    isLogsModalOpen: boolean;
    isSettingsModalOpen: boolean;
    settings: AppSettings;
    error: string | null;
};

function createUIStore() {
    const { subscribe, update, set } = writable<UIState>({
        isLogsModalOpen: false,
        isSettingsModalOpen: false,
        settings: {
            workerList: ['worker1.example.com', 'worker2.example.com'], // Default values
            torrcConfig: '# Default torrc config\n',
            exitCountry: null,
            bridges: [],
        },
        error: null,
    });

    const actions = {
        toggleLogsModal: () => update(state => ({ ...state, isLogsModalOpen: !state.isLogsModalOpen })),
        toggleSettingsModal: () => update(state => ({ ...state, isSettingsModalOpen: !state.isSettingsModalOpen })),
        openLogsModal: () => update(state => ({ ...state, isLogsModalOpen: true })),
        closeLogsModal: () => update(state => ({ ...state, isLogsModalOpen: false })),
        openSettingsModal: () => update(state => ({ ...state, isSettingsModalOpen: true })),
        closeSettingsModal: () => update(state => ({ ...state, isSettingsModalOpen: false })),

        loadSettings: async () => {
            try {
                const storedSettings = await db.settings.get(1);
                if (storedSettings) {
                    update(state => ({
                        ...state,
                        settings: {
                            workerList: storedSettings.workerList,
                            torrcConfig: storedSettings.torrcConfig,
                            exitCountry: storedSettings.exitCountry ?? null,
                            bridges: storedSettings.bridges ?? [],
                        }
                    }));
                }
            } catch (err) {
                const message = err instanceof Error ? err.message : 'Unknown error';
                update(state => ({ ...state, error: `Failed to load settings: ${message}` }));
            }
        },

        saveSettings: async (newSettings: AppSettings) => {
            try {
                await db.settings.put({ id: 1, ...newSettings });
                update(state => ({ ...state, settings: newSettings, error: null }));
            } catch (err) {
                const message = err instanceof Error ? err.message : 'Unknown error';
                update(state => ({ ...state, error: `Failed to save settings: ${message}` }));
            }
        },

        setBridges: async (bridges: string[]) => {
            try {
                await invoke('set_bridges', { bridges });
                const current = get({ subscribe });
                const newSettings: AppSettings = {
                    ...current.settings,
                    bridges,
                };
                await db.settings.put({ id: 1, ...newSettings });
                update(state => ({ ...state, settings: newSettings, error: null }));
            } catch (err) {
                const message = err instanceof Error ? err.message : 'Unknown error';
                update(state => ({ ...state, error: `Failed to set bridges: ${message}` }));
            }
        },

        setExitCountry: async (country: string | null) => {
            try {
                await invoke('set_exit_country', { country });
                const current = get({ subscribe });
                const newSettings: AppSettings = {
                    ...current.settings,
                    exitCountry: country,
                };
                await db.settings.put({ id: 1, ...newSettings });
                update(state => ({ ...state, settings: newSettings, error: null }));
            } catch (err) {
                const message = err instanceof Error ? err.message : 'Unknown error';
                update(state => ({ ...state, error: `Failed to set exit country: ${message}` }));
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
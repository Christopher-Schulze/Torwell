import { writable } from 'svelte/store';
import { db } from '$lib/database';
import type { Settings } from '$lib/database';

type AppSettings = {
    workerList: string[];
    torrcConfig: string;
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
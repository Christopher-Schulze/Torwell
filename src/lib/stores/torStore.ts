import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';

export type TorStatus = 'DISCONNECTED' | 'CONNECTING' | 'CONNECTED' | 'DISCONNECTING' | 'ERROR';

export interface TorState {
    status: TorStatus;
    bootstrapProgress: number;
    errorMessage: string | null;
}

function createTorStore() {
    const initialState: TorState = {
        status: 'DISCONNECTED',
        bootstrapProgress: 0,
        errorMessage: null,
    };

    const { subscribe, update, set } = writable<TorState>(initialState);

    // Listen for status updates from the Rust backend
    listen<TorState>('tor-status-update', (event) => {
        update(state => ({ ...state, ...event.payload }));
    });

    return {
        subscribe,
        set,
        update,
    };
}

export const torStore = createTorStore();
import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';

export type TorStatus =
    | 'DISCONNECTED'
    | 'CONNECTING'
    | 'RETRYING'
    | 'CONNECTED'
    | 'DISCONNECTING'
    | 'ERROR';

export interface TorState {
    status: TorStatus;
    bootstrapProgress: number;
    errorMessage: string | null;
    retryCount: number;
}

function createTorStore() {
        const initialState: TorState = {
            status: 'DISCONNECTED',
            bootstrapProgress: 0,
            errorMessage: null,
            retryCount: 0,
        };

    const { subscribe, update, set } = writable<TorState>(initialState);

    // Listen for status updates from the Rust backend
    listen<TorState>('tor-status-update', (event) => {
        update(state => ({
            ...state,
            ...event.payload,
            retryCount: event.payload.retryCount ?? (event.payload.status === 'CONNECTED' ? 0 : state.retryCount)
        }));
    });

    return {
        subscribe,
        set,
        update,
    };
}

export const torStore = createTorStore();

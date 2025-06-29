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
    retryDelay: number;
}

function createTorStore() {
        const initialState: TorState = {
            status: 'DISCONNECTED',
            bootstrapProgress: 0,
            errorMessage: null,
            retryCount: 0,
            retryDelay: 0,
        };

    const { subscribe, update, set } = writable<TorState>(initialState);

    // Listen for status updates from the Rust backend
    listen<TorState>('tor-status-update', (event) => {
        update(state => ({
            ...state,
            ...event.payload,
            retryCount: event.payload.retryCount ?? ([ 'CONNECTED', 'DISCONNECTED', 'ERROR' ].includes(event.payload.status as string) ? 0 : state.retryCount),
            retryDelay: event.payload.retryDelay ?? ([ 'CONNECTED', 'DISCONNECTED', 'ERROR' ].includes(event.payload.status as string) ? 0 : state.retryDelay)
        }));
    });

    return {
        subscribe,
        set,
        update,
    };
}

export const torStore = createTorStore();

import { writable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

export type TorStatus =
  | "DISCONNECTED"
  | "CONNECTING"
  | "RETRYING"
  | "CONNECTED"
  | "DISCONNECTING"
  | "ERROR";

export interface TorState {
  status: TorStatus;
  bootstrapProgress: number;
  bootstrapMessage: string;
  errorMessage: string | null;
  retryCount: number;
  retryDelay: number;
  memoryUsageMB: number;
  circuitCount: number;
  pingMs: number | undefined;
}

function createTorStore() {
  const initialState: TorState = {
    status: "DISCONNECTED",
    bootstrapProgress: 0,
    bootstrapMessage: "",
    errorMessage: null,
    retryCount: 0,
    retryDelay: 0,
    memoryUsageMB: 0,
    circuitCount: 0,
    pingMs: undefined,
  };

  const { subscribe, update, set } = writable<TorState>(initialState);

  // Listen for metrics updates from the Rust backend
  listen<any>("metrics-update", (event) => {
    update((state) => ({
      ...state,
      memoryUsageMB: Math.round(event.payload.memory_bytes / 1_000_000),
      circuitCount: event.payload.circuit_count,
      pingMs: event.payload.latency_ms,
    }));
  });

  // Listen for status updates from the Rust backend
  listen<TorState>("tor-status-update", (event) => {
    update((state) => ({
      ...state,
      ...event.payload,
      retryCount:
        event.payload.retryCount ??
        (["CONNECTED", "DISCONNECTED", "ERROR"].includes(
          event.payload.status as string,
        )
          ? 0
          : state.retryCount),
      retryDelay:
        event.payload.retryDelay ??
        (["CONNECTED", "DISCONNECTED", "ERROR"].includes(
          event.payload.status as string,
        )
          ? 0
          : state.retryDelay),
      bootstrapMessage:
        event.payload.bootstrapMessage ??
        (["CONNECTED", "DISCONNECTED", "ERROR"].includes(
          event.payload.status as string,
        )
          ? ""
          : state.bootstrapMessage),
    }));

    const newStatus =
      (event.payload.status as TorStatus) ?? initialState.status;
    if (newStatus !== "CONNECTED") {
      update((state) => ({
        ...state,
        memoryUsageMB: 0,
        circuitCount: 0,
        pingMs: undefined,
      }));
    }
  });

  return {
    subscribe,
    set,
    update,
  };
}

export const torStore = createTorStore();

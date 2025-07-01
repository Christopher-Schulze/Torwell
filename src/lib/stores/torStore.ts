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
  };

  const { subscribe, update, set } = writable<TorState>(initialState);

  // Periodic metrics polling
  let metricsInterval: ReturnType<typeof setInterval> | null = null;

  async function fetchMetrics() {
    try {
      const metrics = await invoke<any>("get_metrics");
      update((state) => ({
        ...state,
        memoryUsageMB: Math.round(metrics.memory_bytes / 1_000_000),
        circuitCount: metrics.circuit_count,
      }));
    } catch (err) {
      console.error("Failed to get metrics", err);
    }
  }

  function startMetrics() {
    if (!metricsInterval) {
      fetchMetrics();
      metricsInterval = setInterval(fetchMetrics, 5000);
    }
  }

  function stopMetrics() {
    if (metricsInterval) {
      clearInterval(metricsInterval);
      metricsInterval = null;
    }
    update((state) => ({ ...state, memoryUsageMB: 0, circuitCount: 0 }));
  }

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
    if (newStatus === "CONNECTED") {
      startMetrics();
    } else {
      stopMetrics();
    }
  });

  return {
    subscribe,
    set,
    update,
  };
}

export const torStore = createTorStore();

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
  errorStep: string | null;
  errorSource: string | null;
  securityWarning: string | null;
  retryCount: number;
  retryDelay: number;
  memoryUsageMB: number;
  circuitCount: number;
  pingMs: number | undefined;
  metrics: MetricPoint[];
}

export interface MetricPoint {
  time: number;
  memoryMB: number;
  circuitCount: number;
  latencyMs: number;
  oldestAge: number;
  avgCreateMs: number;
  failedAttempts: number;
  cpuPercent: number;
  networkBytes: number;
  networkTotal: number;
  complete: boolean;
}

function createTorStore() {
  const initialState: TorState = {
    status: "DISCONNECTED",
    bootstrapProgress: 0,
    bootstrapMessage: "",
    errorMessage: null,
    errorStep: null,
    errorSource: null,
    securityWarning: null,
    retryCount: 0,
    retryDelay: 0,
    memoryUsageMB: 0,
    circuitCount: 0,
    pingMs: undefined,
    metrics: [],
  };

  const { subscribe, update, set } = writable<TorState>(initialState);

  // Listen for metrics updates from the Rust backend
  const MAX_POINTS = 30;
  listen<any>("metrics-update", (event) => {
    const point: MetricPoint = {
      time: Date.now(),
      memoryMB: Math.round(event.payload.memory_bytes / 1_000_000),
      circuitCount: event.payload.circuit_count,
      latencyMs: event.payload.latency_ms,
      oldestAge: event.payload.oldest_age ?? 0,
      avgCreateMs: event.payload.avg_create_ms ?? 0,
      failedAttempts: event.payload.failed_attempts ?? 0,
      cpuPercent: event.payload.cpu_percent ?? 0,
      networkBytes: event.payload.network_bytes ?? 0,
      networkTotal: event.payload.total_network_bytes ?? 0,
      complete: event.payload.complete ?? true,
    };
    update((state) => {
      const metrics = [...state.metrics, point].slice(-MAX_POINTS);
      return {
        ...state,
        memoryUsageMB: point.memoryMB,
        circuitCount: point.circuitCount,
        pingMs: point.latencyMs,
        metrics,
      };
    });
  });

  listen<string>("security-warning", (event) => {
    update((state) => ({ ...state, securityWarning: event.payload }));
  });

  // Listen for status updates from the Rust backend
  listen<TorState>("tor-status-update", (event) => {
    update((state) => {
      const statusStr = event.payload.status as string;
      const clearError = !["RETRYING", "ERROR"].includes(statusStr);
      return {
        ...state,
        ...event.payload,
        retryCount:
          event.payload.retryCount ??
          (["CONNECTED", "DISCONNECTED", "ERROR"].includes(statusStr)
            ? 0
            : state.retryCount),
        retryDelay:
          event.payload.retryDelay ??
          (["CONNECTED", "DISCONNECTED", "ERROR"].includes(statusStr)
            ? 0
            : state.retryDelay),
        bootstrapMessage:
          event.payload.bootstrapMessage ??
          (["CONNECTED", "DISCONNECTED", "ERROR"].includes(statusStr)
            ? ""
            : state.bootstrapMessage),
        errorMessage:
          event.payload.errorMessage ??
          (["CONNECTED", "DISCONNECTED"].includes(statusStr)
            ? null
            : state.errorMessage),
        errorStep: event.payload.errorStep ?? (clearError ? null : state.errorStep),
        errorSource:
          event.payload.errorSource ?? (clearError ? null : state.errorSource),
      };
    });

    const newStatus =
      (event.payload.status as TorStatus) ?? initialState.status;
    if (newStatus !== "CONNECTED") {
      update((state) => ({
        ...state,
        memoryUsageMB: 0,
        circuitCount: 0,
        pingMs: undefined,
        metrics: [],
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

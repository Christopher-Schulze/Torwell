import { writable } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

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
  lastTransition: string | null;
}

export interface MetricPoint {
  time: number;
  memoryMB: number;
  circuitCount: number;
  latencyMs: number | undefined;
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
    lastTransition: null,
  };

  const listeners: Promise<UnlistenFn>[] = [];

  const { subscribe, update, set } = writable<TorState>(initialState, () => {
    const MAX_POINTS = 720;

    const metricsListener = listen<Record<string, any>>("metrics-update", (event) => {
      const payload = event.payload ?? {};
      const memoryBytes = Number(payload.memory_bytes ?? payload.memoryBytes ?? 0);
      const rawCircuitCount = payload.circuit_count ?? payload.circuitCount ?? 0;
      const circuitCountValue =
        typeof rawCircuitCount === "number"
          ? rawCircuitCount
          : Number(rawCircuitCount);
      const safeCircuitCount: number = Number.isFinite(circuitCountValue)
        ? (circuitCountValue as number)
        : 0;
      const latency = payload.latency_ms ?? payload.latencyMs;
      const point: MetricPoint = {
        time: Date.now(),
        memoryMB: Math.max(0, Math.round(memoryBytes / 1_000_000)),
        circuitCount: safeCircuitCount,
        latencyMs: typeof latency === "number" ? latency : undefined,
        oldestAge: Number(payload.oldest_age ?? payload.oldestAge ?? 0),
        avgCreateMs: Number(payload.avg_create_ms ?? payload.avgCreateMs ?? 0),
        failedAttempts: Number(payload.failed_attempts ?? payload.failedAttempts ?? 0),
        cpuPercent: Number(payload.cpu_percent ?? payload.cpuPercent ?? 0),
        networkBytes: Number(payload.network_bytes ?? payload.networkBytes ?? 0),
        networkTotal: Number(payload.total_network_bytes ?? payload.totalNetworkBytes ?? 0),
        complete: payload.complete ?? true,
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

    const securityListener = listen<string>("security-warning", (event) => {
      update((state) => ({ ...state, securityWarning: event.payload ?? null }));
    });

    const statusListener = listen<Record<string, any>>("tor-status-update", (event) => {
      const payload = event.payload ?? {};
      update((state) => {
        const rawStatus = (payload.status as string) ?? initialState.status;
        const isEphemeral = ["NEW_IDENTITY", "NEW_CIRCUIT"].includes(rawStatus);
        const statusStr = (isEphemeral ? "CONNECTED" : rawStatus) as TorStatus;
        const clearError = !["RETRYING", "ERROR"].includes(statusStr);
        return {
          ...state,
          ...payload,
          status: statusStr,
          lastTransition: rawStatus,
          retryCount:
            payload.retryCount ??
            (["CONNECTED", "DISCONNECTED", "ERROR"].includes(statusStr)
              ? 0
              : state.retryCount),
          retryDelay:
            payload.retryDelay ??
            (["CONNECTED", "DISCONNECTED", "ERROR"].includes(statusStr)
              ? 0
              : state.retryDelay),
          bootstrapMessage:
            payload.bootstrapMessage ??
            (["CONNECTED", "DISCONNECTED", "ERROR"].includes(statusStr)
              ? ""
              : state.bootstrapMessage),
          errorMessage:
            payload.errorMessage ??
            (["CONNECTED", "DISCONNECTED"].includes(statusStr)
              ? null
              : state.errorMessage),
          errorStep: payload.errorStep ?? (clearError ? null : state.errorStep),
          errorSource:
            payload.errorSource ?? (clearError ? null : state.errorSource),
        };
      });

      update((state) => {
        if (state.status === "CONNECTED") {
          return {
            ...state,
            securityWarning: null,
            errorMessage: null,
            errorStep: null,
            errorSource: null,
          };
        }
        return {
          ...state,
          memoryUsageMB: 0,
          circuitCount: 0,
          pingMs: undefined,
          metrics: [],
        };
      });
    });

    listeners.push(metricsListener, securityListener, statusListener);

    return () => {
      listeners.splice(0).forEach(async (promise) => {
        try {
          const unlisten = await promise;
          unlisten();
        } catch (error) {
          console.error("Failed to unlisten from Tauri event", error);
        }
      });
      set(initialState);
    };
  });

  return {
    subscribe,
    set,
    update,
  };
}

export const torStore = createTorStore();

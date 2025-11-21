import { writable } from "svelte/store";
import { invalidateConnectionCaches } from "../../cache";

// Dynamically import Tauri API to prevent SSR issues
let listen: typeof import("@tauri-apps/api/event").listen;
if (typeof window !== "undefined") {
  import("@tauri-apps/api/event").then((module) => {
    listen = module.listen;
  });
}

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
  systemProxyEnabled: boolean;
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
    systemProxyEnabled: true,
  };

  // const listeners: Promise<UnlistenFn>[] = []; // Removed unused variable

  const { subscribe, update, set } = writable<TorState>(initialState, () => {
    if (typeof window === "undefined") return; // SSR check

    const MAX_POINTS = 720;

    // We need to wait for the dynamic import to resolve
    const setupListeners = async () => {
        if (!listen) {
            const module = await import("@tauri-apps/api/event");
            listen = module.listen;
        }

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
            if (["DISCONNECTED", "ERROR"].includes(statusStr)) {
              invalidateConnectionCaches();
            }
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

          // Additional update logic if needed based on status
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
             // Reset metrics on disconnect
             if (state.status === "DISCONNECTED") {
                return {
                  ...state,
                  memoryUsageMB: 0,
                  circuitCount: 0,
                  pingMs: undefined,
                  metrics: [],
                };
             }
            return state;
          });
        });

        const proxyListener = listen<Record<string, any>>("system-proxy-update", (event) => {
            const payload = event.payload ?? {};
            if (typeof payload.enabled === "boolean") {
                update((state) => ({ ...state, systemProxyEnabled: payload.enabled }));
            }
        });

        return [metricsListener, securityListener, statusListener, proxyListener];
    };

    const listenersPromise = setupListeners();

    return () => {
      listenersPromise.then((promises) => {
        promises.forEach(async (promise) => {
            try {
              const unlisten = await promise;
              unlisten();
            } catch (error) {
              console.error("Failed to unlisten from Tauri event", error);
            }
        });
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

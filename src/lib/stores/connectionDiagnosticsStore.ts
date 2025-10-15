import { browser } from '$app/environment';
import { writable } from 'svelte/store';
import { getConnectionHealthSummary, getConnectionTimeline } from '$lib/api';
import type { ConnectionEvent, ConnectionHealthSummary } from '$lib/types';

export interface ConnectionDiagnosticsState {
  timeline: ConnectionEvent[];
  summary: ConnectionHealthSummary | null;
  loading: boolean;
  error: string | null;
  lastUpdated: number | null;
}

const INITIAL_STATE: ConnectionDiagnosticsState = {
  timeline: [],
  summary: null,
  loading: false,
  error: null,
  lastUpdated: null,
};

const REFRESH_INTERVAL_MS = 10_000;
const TIMELINE_LIMIT = 120;

function createConnectionDiagnosticsStore() {
  const { subscribe, set, update } = writable<ConnectionDiagnosticsState>(INITIAL_STATE);
  let timer: ReturnType<typeof setInterval> | null = null;
  let inFlight = false;

  async function refresh() {
    if (!browser || inFlight) {
      return;
    }
    inFlight = true;
    update((state) => ({ ...state, loading: state.timeline.length === 0, error: null }));
    try {
      const [summary, timeline] = await Promise.all([
        getConnectionHealthSummary(),
        getConnectionTimeline(TIMELINE_LIMIT),
      ]);
      set({
        summary,
        timeline,
        loading: false,
        error: null,
        lastUpdated: Date.now(),
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update((state) => ({ ...state, loading: false, error: message }));
    } finally {
      inFlight = false;
    }
  }

  function start() {
    if (!browser) {
      return;
    }
    if (timer) {
      return;
    }
    refresh();
    timer = setInterval(refresh, REFRESH_INTERVAL_MS);
  }

  function stop() {
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
  }

  function reset() {
    stop();
    set(INITIAL_STATE);
  }

  return {
    subscribe,
    refresh,
    start,
    stop,
    reset,
  };
}

export const connectionDiagnosticsStore = createConnectionDiagnosticsStore();

import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';

let statusCallback: (event: any) => void = () => {};
let metricsCallback: (event: any) => void = () => {};
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event: string, cb: any) => {
    if (event === 'tor-status-update') statusCallback = cb;
    if (event === 'metrics-update') metricsCallback = cb;
  })
}));
vi.mock('@tauri-apps/api/tauri', () => ({ invoke: vi.fn() }));

import { torStore } from '../lib/stores/torStore';

describe('torStore', () => {
  it('sets status to ERROR on failed connection', () => {
    statusCallback({ payload: { status: 'ERROR', errorMessage: 'fail' } });
    expect(get(torStore).status).toBe('ERROR');
  });

  it('updates metrics from backend events', () => {
    metricsCallback({
      payload: {
        memory_bytes: 1_000_000,
        circuit_count: 1,
        latency_ms: 50,
        oldest_age: 2,
        build_ms: 100,
        connect_ms: 200,
      }
    });
    const state = get(torStore);
    expect(state.memoryUsageMB).toBe(1);
    expect(state.circuitCount).toBe(1);
    expect(state.pingMs).toBe(50);
    expect(state.metrics.at(-1)?.buildMs).toBe(100);
    expect(state.metrics.at(-1)?.connectMs).toBe(200);
  });
});

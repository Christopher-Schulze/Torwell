import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { vi } from 'vitest';

let metricsCallback: (e: any) => void = () => {};
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (ev: string, cb: any) => {
    if (ev === 'metrics-update') metricsCallback = cb;
    return () => {};
  })
}));
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === 'request_token') return 42;
    if (cmd === 'load_metrics') return [{
      time: 0,
      memoryMB: 0,
      circuitCount: 0,
      latencyMs: 0,
      oldestAge: 0,
      avgCreateMs: 0,
      failedAttempts: 0,
      cpuPercent: 1,
      networkBytes: 0,
      networkTotal: 100,
      complete: true
    }];
  })
}));
import { invoke } from '@tauri-apps/api/tauri';

import NetworkMonitor from '../lib/components/NetworkMonitor.svelte';

describe('NetworkMonitor', () => {
  it('loads metrics and updates on events', async () => {
    const { getByText } = render(NetworkMonitor);
    await tick();
    await tick();

    metricsCallback({
      payload: {
        memory_bytes: 0,
        circuit_count: 0,
        latency_ms: 0,
        oldest_age: 0,
        avg_create_ms: 0,
        failed_attempts: 0,
        cpu_percent: 2.5,
        network_bytes: 0,
        total_network_bytes: 200,
        complete: true
      }
    });
    await tick();
    expect(getByText(/CPU: 2.5 %/)).toBeInTheDocument();
  });
});

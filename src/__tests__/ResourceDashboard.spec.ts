import { render } from '@testing-library/svelte';
import { vi, describe, it, expect } from 'vitest';
import { tick } from 'svelte';

let metricsCallback: (e: any) => void = () => {};
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((ev: string, cb: any) => {
    if (ev === 'metrics-update') metricsCallback = cb;
  })
}));

import ResourceDashboard from '../lib/components/ResourceDashboard.svelte';

describe('ResourceDashboard', () => {

  it('renders charts snapshot', async () => {
    const { container } = render(ResourceDashboard);
    await tick();

    metricsCallback({
      payload: {
        memory_bytes: 500_000_000,
        circuit_count: 5,
        latency_ms: 0,
        oldest_age: 0,
        avg_create_ms: 10,
        failed_attempts: 0,
        cpu_percent: 7.2,
      network_bytes: 1024,
      },
    });
    await tick();
    await tick();

    expect(container).toMatchSnapshot();
  });
});

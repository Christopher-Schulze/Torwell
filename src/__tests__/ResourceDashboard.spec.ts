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
  it('updates metrics and shows warnings', async () => {
    const { getByText, getAllByRole } = render(ResourceDashboard);

    metricsCallback({ payload: { memory_bytes: 1500_000_000, circuit_count: 25, latency_ms: 0, oldest_age: 0 } });
    await tick();

    expect(getByText('Memory: 1500 MB')).toBeInTheDocument();
    expect(getByText('Circuits: 25')).toBeInTheDocument();
    expect(getAllByRole('alert').length).toBe(2);
  });
});

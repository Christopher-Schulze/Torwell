import { render } from '@testing-library/svelte';
import MetricsChart from '../lib/components/MetricsChart.svelte';

describe('MetricsChart', () => {
  it('renders build time path', () => {
    const metrics = [
      { time: 0, memoryMB: 1, circuitCount: 1, latencyMs: 1, oldestAge: 1, buildMs: 5, buildFailures: 0 }
    ];
    const { container } = render(MetricsChart, { props: { metrics } });
    const path = container.querySelector('path[stroke="purple"]');
    expect(path).toBeTruthy();
  });
});

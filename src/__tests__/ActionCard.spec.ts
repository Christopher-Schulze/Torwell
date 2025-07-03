import { vi, describe, it, expect } from 'vitest';
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));
vi.mock('@tauri-apps/api', () => ({ invoke: vi.fn() }));
import { render, fireEvent } from '@testing-library/svelte';
import ActionCard from '../lib/components/ActionCard.svelte';
import { invoke } from '@tauri-apps/api';

// Reset store between tests by importing after test to ensure fresh instance
import { torStore } from '../lib/stores/torStore';

describe('ActionCard', () => {
  it('renders Connect button when stopped', () => {
    torStore.set({
      status: 'DISCONNECTED',
      bootstrapProgress: 0,
      bootstrapMessage: '',
      errorMessage: null,
      retryCount: 0,
      retryDelay: 0,
      memoryUsageMB: 0,
      circuitCount: 0,
      pingMs: undefined,
    });

    const { getByRole } = render(ActionCard);
    expect(getByRole('button', { name: /connect to tor/i })).toBeInTheDocument();
  });

  it('dispatches openLogs event when Logs button is clicked', async () => {
    const { getByRole, component } = render(ActionCard);
    const handler = vi.fn();
    component.$on('openLogs', handler);
    await fireEvent.click(getByRole('button', { name: /open logs/i }));
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it('calls disconnect when Disconnect button clicked', async () => {
    torStore.set({
      status: 'CONNECTED',
      bootstrapProgress: 0,
      bootstrapMessage: '',
      errorMessage: null,
      retryCount: 0,
      retryDelay: 0,
      memoryUsageMB: 0,
      circuitCount: 0,
      pingMs: undefined,
    });

    const { getByRole } = render(ActionCard);
    await fireEvent.click(getByRole('button', { name: /disconnect from tor/i }));
    expect(invoke).toHaveBeenCalledWith('disconnect');
  });
});

import { render, fireEvent } from '@testing-library/svelte';
import { vi } from 'vitest';

vi.mock('@tauri-apps/api/tauri', () => ({ invoke: vi.fn(async () => 42) }));

import StatusCard from '../lib/components/StatusCard.svelte';
import { invoke } from '@tauri-apps/api/tauri';

describe('StatusCard', () => {
  it('formats traffic as GB', () => {
    const { getByText } = render(StatusCard, {
      props: {
        status: 'CONNECTED',
        totalTrafficMB: 1500,
        memoryMB: 50,
        circuitCount: 2,
        pingMs: undefined
      }
    });

    expect(getByText('1.5 GB')).toBeInTheDocument();
  });

  it('invokes ping_host when ping button clicked', async () => {
    const { getByRole, findByText } = render(StatusCard, {
      props: {
        status: 'CONNECTED',
        totalTrafficMB: 10,
        memoryMB: 5,
        circuitCount: 1,
        pingMs: undefined
      }
    });

    await fireEvent.click(getByRole('button', { name: /run ping test/i }));

    expect(invoke).toHaveBeenNthCalledWith(2, 'ping_host', {
      token: 42,
      host: 'google.com',
      count: 5
    });
    await findByText('42 ms');
  });
});

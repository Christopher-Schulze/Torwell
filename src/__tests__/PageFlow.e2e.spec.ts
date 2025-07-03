import { render, fireEvent, screen } from '@testing-library/svelte';
import { vi } from 'vitest';

vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === 'get_active_circuit') return [];
    if (cmd === 'get_traffic_stats') return { bytes_sent: 0, bytes_received: 0 };
    return null;
  })
}));

import Page from '../routes/+page.svelte';

describe('Main page flow', () => {
  it('opens and closes logs modal via ActionCard', async () => {
    render(Page);

    await fireEvent.click(screen.getByRole('button', { name: /logs/i }));
    await screen.findByRole('dialog');
    expect(screen.getByRole('dialog')).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: /close logs/i }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });
});

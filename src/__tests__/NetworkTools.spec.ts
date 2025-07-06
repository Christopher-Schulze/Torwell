import { render, fireEvent } from '@testing-library/svelte';
import { vi } from 'vitest';

vi.mock('@tauri-apps/api/tauri', () => ({ invoke: vi.fn(async () => 42) }));

import NetworkTools from '../lib/components/NetworkTools.svelte';
import { invoke } from '@tauri-apps/api/tauri';

describe('NetworkTools', () => {
  it('calls dns_lookup on button click', async () => {
    const { getByText, getByLabelText } = render(NetworkTools);
    const input = getByLabelText('Host') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'example.com' } });
    await fireEvent.click(getByText('DNS Lookup'));
    expect(invoke).toHaveBeenNthCalledWith(2, 'dns_lookup', { token: 42, host: 'example.com' });
  });

  it('shows traceroute output', async () => {
    (invoke as any).mockReset();
    (invoke as any)
      .mockResolvedValueOnce(42)
      .mockResolvedValueOnce(['hop1', 'hop2']);

    const { getByText, getByLabelText, findByText } = render(NetworkTools);
    const input = getByLabelText('Host') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'example.com' } });
    await fireEvent.click(getByText('Traceroute'));

    await findByText('Route: hop1 -> hop2');
    expect(invoke).toHaveBeenNthCalledWith(2, 'traceroute_host', { token: 42, host: 'example.com', maxHops: 8 });
  });
});

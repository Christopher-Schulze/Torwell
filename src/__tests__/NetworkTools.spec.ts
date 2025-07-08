import { render, fireEvent } from '@testing-library/svelte';
import { vi } from 'vitest';

var rawInvoke: any;
vi.mock('@tauri-apps/api/tauri', () => {
  rawInvoke = vi.fn(async (cmd: string) => {
    if (cmd === 'request_token') return 42;
    if (cmd === 'dns_lookup') return ['1.1.1.1'];
    if (cmd === 'traceroute_host') return ['1.1.1.1', '2.2.2.2'];
    if (cmd === 'lookup_country') return 'US';
    return [];
  });
  return { invoke: rawInvoke };
});
global.fetch = vi.fn(async () => ({ ok: true, text: async () => 'US' })) as any;

import NetworkTools from '../lib/components/NetworkTools.svelte';
import { invoke as tauriInvoke } from '@tauri-apps/api/tauri';

describe('NetworkTools', () => {
  it('calls dns_lookup on button click', async () => {
    const { getByText, getByLabelText } = render(NetworkTools);
    const input = getByLabelText('Host') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'example.com' } });
    await fireEvent.click(getByText('DNS Lookup'));
    expect(tauriInvoke).toHaveBeenNthCalledWith(2, 'dns_lookup', { token: 42, host: 'example.com' });
  });

  it('copies dns results', async () => {
    (navigator as any).clipboard = { writeText: vi.fn() };
    const { getByRole, getByLabelText, findByRole } = render(NetworkTools);
    const input = getByLabelText('Host') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'example.com' } });
    await fireEvent.click(getByRole('button', { name: 'DNS Lookup' }));
    const copyBtn = await findByRole('button', { name: 'Copy DNS results' });
    await fireEvent.click(copyBtn);
    expect((navigator as any).clipboard.writeText).toHaveBeenCalledWith('1.1.1.1');
  });

  it('shows traceroute output', async () => {
    (tauriInvoke as any).mockReset();
    (tauriInvoke as any)
      .mockResolvedValueOnce(42)
      .mockResolvedValueOnce(['hop1', 'hop2'])
      .mockResolvedValue('US');

    const { getByText, getByLabelText, findByText } = render(NetworkTools);
    const input = getByLabelText('Host') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'example.com' } });
    await fireEvent.click(getByText('Traceroute'));

    await findByText('hop2');
    expect(tauriInvoke).toHaveBeenNthCalledWith(2, 'traceroute_host', { token: 42, host: 'example.com', maxHops: 8 });
    expect(tauriInvoke).toHaveBeenNthCalledWith(3, 'lookup_country', { ip: 'hop1' });
    expect(tauriInvoke).toHaveBeenNthCalledWith(4, 'lookup_country', { ip: 'hop2' });
  });

  it('copies traceroute results', async () => {
    (tauriInvoke as any).mockReset();
    (tauriInvoke as any)
      .mockResolvedValueOnce(42)
      .mockResolvedValueOnce(['1.1.1.1', '2.2.2.2']);
    (navigator as any).clipboard = { writeText: vi.fn() };
    const { getByRole, getByLabelText, findByRole } = render(NetworkTools);
    const input = getByLabelText('Host') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'example.com' } });
    await fireEvent.click(getByRole('button', { name: 'Traceroute' }));
    const copyBtn = await findByRole('button', { name: 'Copy traceroute results' });
    await fireEvent.click(copyBtn);
    expect((navigator as any).clipboard.writeText).toHaveBeenCalledWith('1. 1.1.1.1\n2. 2.2.2.2');
  });
});

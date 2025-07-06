import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { vi } from 'vitest';

let circuits = [1, 2];
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(async (cmd: string, args: any) => {
    if (cmd === 'request_token') return 42;
    if (cmd === 'list_circuits') return circuits;
    if (cmd === 'close_circuit') {
      circuits = circuits.filter((id) => id !== args.id);
      return;
    }
  })
}));

import CircuitManager from '../lib/components/CircuitManager.svelte';
import { invoke } from '@tauri-apps/api/tauri';

describe('CircuitManager', () => {
  it('lists circuits and closes them', async () => {
    const { findByText, getAllByText, queryByText } = render(CircuitManager);

    await findByText('#1');
    await findByText('#2');
    expect(invoke).toHaveBeenNthCalledWith(2, 'list_circuits', { token: 42 });

    await fireEvent.click(getAllByText('Close')[0]);
    await waitFor(() => expect(queryByText('#1')).not.toBeInTheDocument());

    expect(invoke).toHaveBeenNthCalledWith(3, 'close_circuit', { token: 42, id: 1 });
    expect(invoke).toHaveBeenNthCalledWith(4, 'list_circuits', { token: 42 });
  });
});

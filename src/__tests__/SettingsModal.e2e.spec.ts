import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, beforeEach, afterEach, expect, vi } from 'vitest';
import 'fake-indexeddb/auto';

vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));
vi.mock('@tauri-apps/api/tauri', () => ({ invoke: vi.fn() }));

import SettingsModal from '../lib/components/SettingsModal.svelte';
import { db } from '../lib/database';

const BRIDGE =
  'Bridge obfs4 192.0.2.1:443 0123456789ABCDEF0123456789ABCDEF01234567 cert=AAAA iat-mode=0';
const PRESETS = {
  bridges: [BRIDGE],
  exitCountries: [{ code: 'DE', name: 'Germany' }],
};

describe('SettingsModal persistence', () => {
  beforeEach(async () => {
    await db.delete();
    await db.open();
    vi.stubGlobal('fetch', vi.fn(async () => ({ json: async () => PRESETS } as any)));
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('saves and reloads settings', async () => {
    const { getByLabelText, getByRole, unmount } = render(SettingsModal, { props: { show: true } });

    await fireEvent.click(getByLabelText(BRIDGE));
    await fireEvent.change(getByLabelText('Exit country'), { target: { value: 'DE' } });
    await fireEvent.input(getByLabelText('Maximum log lines'), { target: { value: '123' } });
    await fireEvent.click(getByRole('button', { name: 'Apply bridge selection' }));
    await fireEvent.click(getByRole('button', { name: 'Save exit country' }));
    await fireEvent.click(getByRole('button', { name: 'Save log limit' }));

    unmount();

    const stored = await db.settings.get(1);
    expect(stored?.bridges).toContain(BRIDGE);
    expect(stored?.exitCountry).toBe('DE');
    expect(stored?.maxLogLines).toBe(123);

    const { getByLabelText: getByLabelText2 } = render(SettingsModal, { props: { show: true } });
    const input = getByLabelText2('Maximum log lines') as HTMLInputElement;
    const select = getByLabelText2('Exit country') as HTMLSelectElement;
    expect(input.value).toBe('123');
    expect(select.value).toBe('DE');
  });
});

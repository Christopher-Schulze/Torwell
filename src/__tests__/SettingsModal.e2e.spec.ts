import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, beforeEach, afterEach, expect, vi } from 'vitest';
import 'fake-indexeddb/auto';

vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));
vi.mock('@tauri-apps/api/tauri', () => {
  const store = new Map<string, any>();
  return {
    invoke: vi.fn((cmd: string, args?: any) => {
      if (cmd === 'get_secure_key') {
        return Promise.resolve(store.get('aes-key') ?? null);
      }
      if (cmd === 'set_secure_key') {
        store.set('aes-key', args.value);
        return Promise.resolve();
      }
      if (cmd === 'set_bridges') {
        store.set('bridges', args.bridges);
        return Promise.resolve();
      }
      if (cmd === 'set_exit_country') {
        store.set('exitCountry', args.country);
        return Promise.resolve();
      }
      if (cmd === 'set_log_limit') {
        store.set('logLimit', args.limit);
        return Promise.resolve();
      }
      return Promise.resolve(null);
    }),
  };
});

import SettingsModal from '../lib/components/SettingsModal.svelte';
import { db } from '../lib/database';

const BRIDGE =
  'Bridge obfs4 192.0.2.1:443 0123456789ABCDEF0123456789ABCDEF01234567 cert=AAAA iat-mode=0';
const PRESETS = {
  bridges: [BRIDGE],
  presets: [
    { name: 'Default', bridges: [BRIDGE] }
  ],
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
    await Promise.resolve();

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

  it('applies bridges via store and backend', async () => {
  const { getByLabelText, getByRole } = render(SettingsModal, { props: { show: true } });
  await Promise.resolve();

  await fireEvent.click(getByLabelText(BRIDGE));
  await fireEvent.click(getByRole('button', { name: 'Apply bridge selection' }));

  const { uiStore } = await import('../lib/stores/uiStore');
  await Promise.resolve();

  const { get } = await import('svelte/store');
  expect(get(uiStore).settings.bridges).toContain(BRIDGE);

  const { invoke } = await import('@tauri-apps/api/tauri');
  expect(invoke).toHaveBeenCalledWith('set_bridges', { bridges: [BRIDGE] });

  const stored = await db.settings.get(1);
  expect(stored?.bridges).toContain(BRIDGE);
});

  it('loads and saves bridge selection', async () => {
    const { getByLabelText, getByRole, unmount } = render(SettingsModal, { props: { show: true } });
    await Promise.resolve();

    await fireEvent.click(getByLabelText(BRIDGE));
    await fireEvent.click(getByRole('button', { name: 'Apply bridge selection' }));

    unmount();

    const stored = await db.settings.get(1);
  expect(stored?.bridges).toEqual([BRIDGE]);

  const { getByLabelText: getAgain } = render(SettingsModal, { props: { show: true } });
  const bridgeCheckbox = getAgain(BRIDGE) as HTMLInputElement;
  expect(bridgeCheckbox.checked).toBe(true);
  });

  it('applies preset and persists selection', async () => {
    const { getByLabelText, getByText, getByRole, unmount } = render(SettingsModal, { props: { show: true } });
    await Promise.resolve();

    await fireEvent.change(getByLabelText('Bridge preset'), { target: { value: 'Default' } });
    await fireEvent.click(getByRole('button', { name: 'Apply Preset' }));

    unmount();

    const stored = await db.settings.get(1);
    expect(stored?.bridgePreset).toBe('Default');
    expect(stored?.bridges).toEqual([BRIDGE]);

    const { getByLabelText: again } = render(SettingsModal, { props: { show: true } });
    const select = again('Bridge preset') as HTMLSelectElement;
    expect(select.value).toBe('Default');
  });

  it('calls setBridgePreset action', async () => {
    const { uiStore } = await import('../lib/stores/uiStore');
    const spy = vi.spyOn(uiStore.actions, 'setBridgePreset');

    const { getByLabelText, getByRole } = render(SettingsModal, { props: { show: true } });
    await Promise.resolve();

    await fireEvent.change(getByLabelText('Bridge preset'), { target: { value: 'Default' } });
    await fireEvent.click(getByRole('button', { name: 'Apply Preset' }));

    expect(spy).toHaveBeenCalledWith('Default', [BRIDGE]);

    const stored = await db.settings.get(1);
    expect(stored?.bridgePreset).toBe('Default');
    expect(stored?.bridges).toEqual([BRIDGE]);
  });

  it('selects exit country and persists', async () => {
    const { getByLabelText, getByRole, unmount } = render(SettingsModal, { props: { show: true } });
    await Promise.resolve();

    await fireEvent.change(getByLabelText('Exit country'), { target: { value: 'DE' } });
    await fireEvent.click(getByRole('button', { name: 'Save exit country' }));

    unmount();

    const stored = await db.settings.get(1);
    expect(stored?.exitCountry).toBe('DE');

    const { getByLabelText: get2 } = render(SettingsModal, { props: { show: true } });
    const select = get2('Exit country') as HTMLSelectElement;
    expect(select.value).toBe('DE');

    const { invoke } = await import('@tauri-apps/api/tauri');
    expect(invoke).toHaveBeenCalledWith('set_exit_country', { country: 'DE' });
  });

  it('calls setLogLimit action', async () => {
    const { uiStore } = await import('../lib/stores/uiStore');
    const spy = vi.spyOn(uiStore.actions, 'setLogLimit');

    const { getByLabelText, getByRole } = render(SettingsModal, { props: { show: true } });

    await fireEvent.input(getByLabelText('Maximum log lines'), { target: { value: '200' } });
    await fireEvent.click(getByRole('button', { name: 'Save log limit' }));

    expect(spy).toHaveBeenCalledWith(200);

    const stored = await db.settings.get(1);
    expect(stored?.maxLogLines).toBe(200);
  });
});

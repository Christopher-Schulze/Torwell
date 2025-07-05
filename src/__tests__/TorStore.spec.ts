import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';

let statusCallback: (event: any) => void = () => {};
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event: string, cb: any) => {
    if (event === 'tor-status-update') statusCallback = cb;
  })
}));
vi.mock('@tauri-apps/api/tauri', () => ({ invoke: vi.fn() }));

import { torStore } from '../lib/stores/torStore';

describe('torStore', () => {
  it('sets status to ERROR on failed connection', () => {
    statusCallback({ payload: { status: 'ERROR', errorMessage: 'fail' } });
    expect(get(torStore).status).toBe('ERROR');
  });
});

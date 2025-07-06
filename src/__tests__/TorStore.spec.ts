import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

let statusCallback: (event: any) => void = () => {};
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event: string, cb: any) => {
    if (event === 'tor-status-update') statusCallback = cb;
  })
}));
var invoke: any;
vi.mock('@tauri-apps/api/tauri', () => {
  invoke = vi.fn();
  return { invoke };
});

import { torStore } from '../lib/stores/torStore';
import { invoke as call } from '../lib/api';

beforeEach(() => {
  invoke.mockReset();
  statusCallback = () => {};
});

describe('torStore', () => {
  it('sets status to ERROR on failed connection', () => {
    statusCallback({ payload: { status: 'ERROR', errorMessage: 'fail' } });
    expect(get(torStore).status).toBe('ERROR');
  });

  it('requests a new token and retries on InvalidToken', async () => {
    let tokens = ['t1', 't2'];
    invoke.mockImplementation(async (cmd: string, args: any) => {
      if (cmd === 'request_token') return tokens.shift();
      if (args.token === 't1') throw new Error('Invalid session token');
      return 'ok';
    });

    const result = await call('list_circuits');
    expect(result).toBe('ok');
    expect(invoke.mock.calls).toEqual([
      ['request_token'],
      ['list_circuits', { token: 't1' }],
      ['request_token'],
      ['list_circuits', { token: 't2' }],
    ]);
  });
});

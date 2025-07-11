import { describe, it, expect, vi, beforeEach } from 'vitest';
import 'fake-indexeddb/auto';
import { get } from 'svelte/store';

vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

var settings = { put: vi.fn(), get: vi.fn().mockResolvedValue(undefined) };
vi.mock('../lib/database', () => ({ db: { settings } }));

vi.mock('@tauri-apps/api/tauri', () => {
  const store = { workers: [] as string[], token: '' };
  return {
    invoke: vi.fn(async (cmd: string, args?: any) => {
      if (cmd === 'set_worker_config') {
        store.workers = args.workers;
        store.token = args.token;
        return;
      }
      if (cmd === 'validate_worker_token') {
        return store.token === 'valid';
      }
      return;
    }),
  };
});

import { uiStore } from '../lib/stores/uiStore';

beforeEach(async () => {
  settings.put.mockClear();
  const mod = await import('@tauri-apps/api/tauri');
  (mod.invoke as any).mockClear();
});

it('restores previous config on invalid token', async () => {
  await Promise.resolve();
  const initial = get(uiStore).settings.workerList;
  await uiStore.actions.saveWorkerConfig(['https://new'], 'invalid');
  const { settings: s } = get(uiStore);
  expect(s.workerList).toEqual(initial);
  expect(s.workerToken).toBe('');
  const { invoke } = await import('@tauri-apps/api/tauri');
  expect(invoke).toHaveBeenNthCalledWith(1, 'set_worker_config', {
    workers: ['https://new'],
    token: 'invalid',
  });
  expect(invoke).toHaveBeenNthCalledWith(2, 'set_worker_config', {
    workers: initial,
    token: '',
  });
  expect(settings.put).not.toHaveBeenCalled();
});

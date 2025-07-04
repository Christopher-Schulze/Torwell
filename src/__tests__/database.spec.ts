import { describe, it, expect, beforeEach } from 'vitest';
import { indexedDB, IDBKeyRange } from 'fake-indexeddb';
import Dexie from 'dexie';
import { vi } from 'vitest';

vi.mock('@tauri-apps/api/tauri', () => {
  const store = new Map<string, string | null>();
  return {
    invoke: vi.fn((cmd: string, args?: any) => {
      if (cmd === 'get_secure_key') {
        return Promise.resolve(store.get('aes-key') ?? null);
      }
      if (cmd === 'set_secure_key') {
        store.set('aes-key', args.value);
        return Promise.resolve();
      }
      return Promise.resolve(null);
    }),
  };
});

// Configure Dexie to use the in-memory IndexedDB provided by fake-indexeddb
Dexie.dependencies.indexedDB = indexedDB as any;
Dexie.dependencies.IDBKeyRange = IDBKeyRange as any;

import { db } from '../lib/database';

function openRaw() {
  const raw = new Dexie('Torwell84DatabaseV2');
  raw.version(1).stores({
    settings: '++id, workerList, torrcConfig, exitCountry, bridges, maxLogLines',
  });
  raw.version(2).stores({ meta: '&id' });
  return raw.open().then(() => raw);
}

describe('database encryption', () => {
  beforeEach(async () => {
    await db.delete();
    await db.open();
  });

  it('encrypts sensitive fields on write and decrypts on read', async () => {
    await db.settings.put({
      id: 1,
      workerList: [],
      torrcConfig: '',
      exitCountry: 'US',
      bridges: ['b1'],
      maxLogLines: 10,
    });

    const stored = await db.settings.get(1);
    expect(stored?.exitCountry).toBe('US');
    expect(stored?.bridges).toEqual(['b1']);

    const raw = await openRaw();
    const rawData = await raw.table('settings').get(1);
    expect(rawData.exitCountry).not.toBe('US');
    expect(rawData.bridges[0]).not.toBe('b1');

    await raw.close();
  });

  it('stores AES key in secure storage', async () => {
    const api = await import('@tauri-apps/api/tauri');
    await db.settings.put({ id: 2, workerList: [], torrcConfig: '' });
    expect(api.invoke).toHaveBeenCalledWith('set_secure_key', { value: expect.any(String) });
    const key = await api.invoke('get_secure_key');
    expect(typeof key).toBe('string');
  });

  it('migrates AES key from IndexedDB', async () => {
    const raw = await openRaw();
    await raw.table('meta').put({ id: 'aes-key', value: 'oldkey' });
    await raw.close();

    const api = await import('@tauri-apps/api/tauri');
    await db.settings.put({ id: 3, workerList: [], torrcConfig: '' });

    expect(await api.invoke('get_secure_key')).toBe('oldkey');
    const check = await openRaw();
    const migrated = await check.table('meta').get('aes-key');
    expect(migrated).toBeUndefined();
    await check.close();
  });
});

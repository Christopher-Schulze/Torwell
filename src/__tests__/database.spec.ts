import { describe, it, expect, beforeEach } from 'vitest';
import { indexedDB, IDBKeyRange } from 'fake-indexeddb';
import Dexie from 'dexie';

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
});

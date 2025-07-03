import { describe, it, expect, beforeEach } from 'vitest';
import 'fake-indexeddb/auto';
import Dexie from 'dexie';

import { db } from '../lib/database';

function openRaw() {
  const raw = new Dexie('Torwell84DatabaseV2');
  raw.version(1).stores({
    settings: '++id, workerList, torrcConfig, exitCountry, bridges, maxLogLines',
  });
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

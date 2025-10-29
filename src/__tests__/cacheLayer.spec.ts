import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { AdaptiveCache } from '../cache/adaptiveCache';
import type { ConnectionEvent } from '$lib/types';

function advanceBy(ms: number) {
  vi.advanceTimersByTime(ms);
}

describe('AdaptiveCache', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('tracks hits and misses and respects TTL', async () => {
    const cache = new AdaptiveCache<string, number>({
      name: 'ttl-test',
      maxEntries: 4,
      defaultTtlMs: 1000,
    });

    cache.set('a', 42);
    expect(cache.get('a')).toBe(42);
    advanceBy(1100);
    expect(cache.get('a')).toBeUndefined();

    const stats = cache.getStats();
    expect(stats.hits).toBe(1);
    expect(stats.misses).toBe(1);
  });

  it('evicts least recently used entries when full', () => {
    const cache = new AdaptiveCache<string, number>({
      name: 'lru-test',
      maxEntries: 2,
      evictionPolicy: 'lru',
    });

    cache.set('a', 1);
    cache.set('b', 2);
    expect(cache.get('a')).toBe(1);
    cache.set('c', 3);

    expect(cache.has('b')).toBe(false);
    expect(cache.get('a')).toBe(1);
    expect(cache.get('c')).toBe(3);
    expect(cache.getStats().evictions).toBe(1);
  });

  it('warms up entries using provided plan', async () => {
    const cache = new AdaptiveCache<string, number>({
      name: 'warmup-test',
      maxEntries: 4,
      warmup: {
        entries: [
          { key: 'one', loader: () => 1 },
          { key: 'two', loader: async () => 2 },
        ],
      },
    });

    await cache.warmup();
    expect(cache.get('one')).toBe(1);
    expect(cache.get('two')).toBe(2);
  });
});

function createStorage() {
  const store = new Map<string, string>();
  return {
    getItem: (key: string) => (store.has(key) ? store.get(key)! : null),
    setItem: (key: string, value: string) => {
      store.set(key, value);
    },
    removeItem: (key: string) => {
      store.delete(key);
    },
    clear: () => store.clear(),
    _dump: () => store,
  };
}

describe('cache layer integration', () => {
  const timelineKey = 'torwell.cache.connection.timeline';
  let warnSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.resetModules();
    const storage = createStorage();
    (globalThis as any).window = {
      localStorage: storage,
    };
    warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
  });

  afterEach(() => {
    warnSpy.mockRestore();
  });

  it('persists and hydrates connection timeline snapshots', async () => {
    const module = await import('../cache');
    const sample: ConnectionEvent[] = [
      {
        timestamp: new Date().toISOString(),
        status: 'CONNECTED',
        message: 'ok',
        detail: null,
        retryCount: 0,
        latencyMs: 12,
        memoryBytes: 1024,
        circuitCount: 3,
      },
    ];

    module.cacheConnectionTimeline('timeline:default', sample, { ttlMs: 50_000 });
    const storedRaw = window.localStorage.getItem(timelineKey);
    expect(storedRaw).toBeTruthy();

    // Reload module to trigger warmup from storage snapshot
    vi.resetModules();
    const moduleRehydrated = await import('../cache');
    await moduleRehydrated.warmupCaches();

    const hydrated = moduleRehydrated.connectionTimelineCache.get('timeline:default');
    expect(hydrated).toEqual(sample);
  });
});

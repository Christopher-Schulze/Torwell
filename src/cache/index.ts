import type { ConnectionEvent, ConnectionHealthSummary } from '$lib/types';
import { AdaptiveCache } from './adaptiveCache';
import type { CacheConfig, CacheWarmupEntry } from './types';

const TIMELINE_STORAGE_KEY = 'torwell.cache.connection.timeline';
const SUMMARY_STORAGE_KEY = 'torwell.cache.connection.summary';
const COUNTRY_STORAGE_KEY = 'torwell.cache.geo.country';

type StoredSnapshot<V> = Array<{ key: string; value: V }>;

function isBrowser(): boolean {
  return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined';
}

function readSnapshot<V>(storageKey: string): StoredSnapshot<V> {
  if (!isBrowser()) return [];
  try {
    const raw = window.localStorage.getItem(storageKey);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as StoredSnapshot<V>;
    if (!Array.isArray(parsed)) return [];
    return parsed;
  } catch (error) {
    console.warn(`Failed to parse cache snapshot for ${storageKey}`, error);
    return [];
  }
}

function writeSnapshot<V>(storageKey: string, entries: StoredSnapshot<V>): void {
  if (!isBrowser()) return;
  try {
    if (!entries.length) {
      window.localStorage.removeItem(storageKey);
      return;
    }
    window.localStorage.setItem(storageKey, JSON.stringify(entries));
  } catch (error) {
    console.warn(`Failed to persist cache snapshot for ${storageKey}`, error);
  }
}

function warmupEntriesFromStorage<V>(storageKey: string): CacheWarmupEntry<string, V>[] {
  return readSnapshot<V>(storageKey).map((entry) => ({
    key: entry.key,
    loader: () => entry.value,
    ttlMs: 5_000,
    tags: ['hydrated'],
  }));
}

const timelineWarmup = warmupEntriesFromStorage<ConnectionEvent[]>(TIMELINE_STORAGE_KEY);
const summaryWarmup = warmupEntriesFromStorage<ConnectionHealthSummary>(SUMMARY_STORAGE_KEY);
const countryWarmup = warmupEntriesFromStorage<string>(COUNTRY_STORAGE_KEY);

const timelineCacheConfig: CacheConfig<string, ConnectionEvent[]> = {
  name: 'connection-timeline',
  maxEntries: 16,
  maxCost: 256,
  defaultTtlMs: 20_000,
  evictionPolicy: 'lru',
  warmup: {
    entries: timelineWarmup,
    parallelism: 2,
    onError: (error, entry) => {
      console.warn('Failed to warmup timeline cache for key', entry.key, error);
    },
  },
};

const summaryCacheConfig: CacheConfig<string, ConnectionHealthSummary> = {
  name: 'connection-summary',
  maxEntries: 4,
  defaultTtlMs: 15_000,
  evictionPolicy: 'lru',
  warmup: {
    entries: summaryWarmup,
    parallelism: 1,
    onError: (error, entry) => {
      console.warn('Failed to warmup summary cache for key', entry.key, error);
    },
  },
};

const countryCacheConfig: CacheConfig<string, string> = {
  name: 'geo-country-lookup',
  maxEntries: 512,
  defaultTtlMs: 3600_000,
  evictionPolicy: 'lfu',
  warmup: {
    entries: countryWarmup,
    parallelism: 4,
    onError: (error, entry) => {
      console.warn('Failed to warmup country cache for key', entry.key, error);
    },
  },
};

export const connectionTimelineCache = new AdaptiveCache<string, ConnectionEvent[]>(timelineCacheConfig);
export const connectionSummaryCache = new AdaptiveCache<string, ConnectionHealthSummary>(summaryCacheConfig);
export const countryLookupCache = new AdaptiveCache<string, string>(countryCacheConfig);

function persistConnectionTimelineSnapshot(): void {
  const snapshot = connectionTimelineCache
    .snapshot()
    .map(({ key, value }) => ({ key: String(key), value }));
  writeSnapshot(TIMELINE_STORAGE_KEY, snapshot);
}

function persistConnectionSummarySnapshot(): void {
  const snapshot = connectionSummaryCache
    .snapshot()
    .map(({ key, value }) => ({ key: String(key), value }));
  writeSnapshot(SUMMARY_STORAGE_KEY, snapshot);
}

function persistCountrySnapshot(): void {
  const snapshot = countryLookupCache
    .snapshot()
    .map(({ key, value }) => ({ key: String(key), value }));
  writeSnapshot(COUNTRY_STORAGE_KEY, snapshot);
}

export function cacheConnectionTimeline(
  key: string,
  value: ConnectionEvent[],
  options?: { ttlMs?: number }
): ConnectionEvent[] {
  const tags = ['connection'];
  const stored = connectionTimelineCache.set(key, value, { ttlMs: options?.ttlMs, tags });
  persistConnectionTimelineSnapshot();
  return stored;
}

export function cacheConnectionSummary(
  key: string,
  value: ConnectionHealthSummary,
  options?: { ttlMs?: number }
): ConnectionHealthSummary {
  const tags = ['connection'];
  const stored = connectionSummaryCache.set(key, value, { ttlMs: options?.ttlMs, tags });
  persistConnectionSummarySnapshot();
  return stored;
}

export function cacheCountryLookup(
  key: string,
  value: string,
  options?: { ttlMs?: number }
): string {
  const stored = countryLookupCache.set(key, value, {
    ttlMs: options?.ttlMs,
    tags: ['geoip'],
  });
  persistCountrySnapshot();
  return stored;
}

export function warmupCaches(): Promise<void[]> {
  return Promise.all([
    connectionTimelineCache.warmup(),
    connectionSummaryCache.warmup(),
    countryLookupCache.warmup(),
  ]);
}

export function invalidateConnectionCaches(): void {
  connectionTimelineCache.clear();
  connectionSummaryCache.clear();
  writeSnapshot(TIMELINE_STORAGE_KEY, []);
  writeSnapshot(SUMMARY_STORAGE_KEY, []);
}

export function resetGeoCache(): void {
  countryLookupCache.clear();
  writeSnapshot(COUNTRY_STORAGE_KEY, []);
}

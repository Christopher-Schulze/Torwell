export type EvictionPolicy = 'lru' | 'lfu' | 'fifo';

export interface CacheWarmupEntry<K, V> {
  key: K;
  loader: () => Promise<V> | V;
  ttlMs?: number;
  tags?: string[];
}

export interface CacheWarmupPlan<K, V> {
  entries: CacheWarmupEntry<K, V>[];
  parallelism?: number;
  onError?: (error: unknown, entry: CacheWarmupEntry<K, V>) => void;
}

export interface CacheConfig<K = unknown, V = unknown> {
  name: string;
  maxEntries: number;
  maxCost?: number;
  defaultTtlMs?: number;
  evictionPolicy?: EvictionPolicy;
  warmup?: CacheWarmupPlan<K, V>;
}

export interface CacheEntry<V> {
  value: V;
  expiresAt: number | null;
  frequency: number;
  lastAccessed: number;
  cost: number;
  tags: string[] | null;
}

export interface CacheStats {
  hits: number;
  misses: number;
  warmups: number;
  evictions: number;
  loads: number;
}

export interface SnapshotEntry<K, V> {
  key: K;
  value: V;
  expiresAt: number | null;
  frequency: number;
  lastAccessed: number;
  cost: number;
  tags: string[] | null;
}

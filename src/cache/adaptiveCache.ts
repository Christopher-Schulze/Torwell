import type { CacheConfig, CacheEntry, CacheStats, SnapshotEntry, EvictionPolicy } from './types';

export interface SetOptions {
  ttlMs?: number;
  cost?: number;
  tags?: string[] | null;
}

function now(): number {
  return Date.now();
}

function normaliseTags(tags?: string[] | null): string[] | null {
  if (!tags || tags.length === 0) return null;
  return Array.from(new Set(tags.map((tag) => tag.trim()).filter(Boolean)));
}

export class AdaptiveCache<K, V> {
  private readonly config: CacheConfig<K, V>;
  private readonly entries = new Map<K, CacheEntry<V>>();
  private stats: CacheStats = { hits: 0, misses: 0, warmups: 0, evictions: 0, loads: 0 };
  private totalCost = 0;

  constructor(config: CacheConfig<K, V>) {
    if (config.maxEntries <= 0) {
      throw new Error(`Cache ${config.name} must allow at least one entry`);
    }
    this.config = {
      evictionPolicy: 'lru',
      defaultTtlMs: undefined,
      ...config,
    };
  }

  get size(): number {
    return this.entries.size;
  }

  getStats(): CacheStats {
    return { ...this.stats };
  }

  resetStats(): void {
    this.stats = { hits: 0, misses: 0, warmups: 0, evictions: 0, loads: 0 };
  }

  clear(): void {
    this.entries.clear();
    this.totalCost = 0;
  }

  has(key: K): boolean {
    const entry = this.entries.get(key);
    if (!entry) return false;
    if (this.isExpired(entry)) {
      this.entries.delete(key);
      this.totalCost -= entry.cost;
      return false;
    }
    return true;
  }

  delete(key: K): boolean {
    const entry = this.entries.get(key);
    if (!entry) return false;
    this.entries.delete(key);
    this.totalCost -= entry.cost;
    return true;
  }

  get(key: K): V | undefined {
    const entry = this.entries.get(key);
    if (!entry) {
      this.stats.misses += 1;
      return undefined;
    }
    if (this.isExpired(entry)) {
      this.entries.delete(key);
      this.totalCost -= entry.cost;
      this.stats.misses += 1;
      return undefined;
    }
    this.touchEntry(key, entry);
    this.stats.hits += 1;
    return entry.value;
  }

  async getOrLoad(key: K, loader: () => Promise<V> | V, options?: SetOptions): Promise<V> {
    const cached = this.get(key);
    if (cached !== undefined) {
      return cached;
    }
    this.stats.loads += 1;
    const value = await loader();
    this.set(key, value, options);
    return value;
  }

  set(key: K, value: V, options?: SetOptions): V {
    const ttlMs = options?.ttlMs ?? this.config.defaultTtlMs ?? null;
    const entry: CacheEntry<V> = {
      value,
      expiresAt: ttlMs ? now() + ttlMs : null,
      frequency: 0,
      lastAccessed: now(),
      cost: Math.max(1, Math.floor(options?.cost ?? 1)),
      tags: normaliseTags(options?.tags ?? undefined),
    };
    const existing = this.entries.get(key);
    if (existing) {
      this.totalCost -= existing.cost;
    }
    this.entries.set(key, entry);
    this.totalCost += entry.cost;
    this.evictIfNeeded();
    return value;
  }

  snapshot(): SnapshotEntry<K, V>[] {
    const items: SnapshotEntry<K, V>[] = [];
    for (const [key, entry] of this.entries.entries()) {
      if (this.isExpired(entry)) {
        continue;
      }
      items.push({
        key,
        value: entry.value,
        expiresAt: entry.expiresAt,
        frequency: entry.frequency,
        lastAccessed: entry.lastAccessed,
        cost: entry.cost,
        tags: entry.tags,
      });
    }
    return items;
  }

  async warmup(): Promise<void> {
    const plan = this.config.warmup;
    if (!plan || plan.entries.length === 0) return;
    const queue = [...plan.entries];
    const parallelism = Math.max(1, Math.min(queue.length, plan.parallelism ?? queue.length));

    const execute = async () => {
      for (;;) {
        const next = queue.shift();
        if (!next) break;
        try {
          const value = await next.loader();
          this.set(next.key, value, { ttlMs: next.ttlMs, tags: next.tags });
          this.stats.warmups += 1;
        } catch (error) {
          plan.onError?.(error, next);
        }
      }
    };

    await Promise.all(Array.from({ length: parallelism }, execute));
  }

  invalidateByTag(tag: string): void {
    const normalised = tag.trim();
    if (!normalised) return;
    for (const [key, entry] of this.entries.entries()) {
      if (entry.tags && entry.tags.includes(normalised)) {
        this.entries.delete(key);
        this.totalCost -= entry.cost;
      }
    }
  }

  purgeExpired(): void {
    for (const [key, entry] of this.entries.entries()) {
      if (this.isExpired(entry)) {
        this.entries.delete(key);
        this.totalCost -= entry.cost;
      }
    }
  }

  private isExpired(entry: CacheEntry<V>): boolean {
    return entry.expiresAt !== null && entry.expiresAt <= now();
  }

  private touchEntry(key: K, entry: CacheEntry<V>): void {
    entry.frequency += 1;
    entry.lastAccessed = now();
    if (this.config.evictionPolicy === 'lru') {
      this.entries.delete(key);
      this.entries.set(key, entry);
    }
  }

  private evictIfNeeded(): void {
    const limit = this.config.maxEntries;
    const maxCost = this.config.maxCost ?? Number.POSITIVE_INFINITY;
    while (this.entries.size > limit || this.totalCost > maxCost) {
      const evicted = this.evictOne();
      if (!evicted) break;
    }
  }

  private evictOne(): boolean {
    if (this.entries.size === 0) return false;
    let candidateKey: K | undefined;
    let candidateEntry: CacheEntry<V> | undefined;

    const policy: EvictionPolicy = this.config.evictionPolicy ?? 'lru';
    if (policy === 'fifo') {
      const iterator = this.entries.entries().next();
      if (!iterator.done) {
        candidateKey = iterator.value[0];
        candidateEntry = iterator.value[1];
      }
    } else {
      let bestScore = Number.POSITIVE_INFINITY;
      for (const [key, entry] of this.entries.entries()) {
        if (this.isExpired(entry)) {
          candidateKey = key;
          candidateEntry = entry;
          break;
        }
        let score: number;
        if (policy === 'lfu') {
          score = entry.frequency + entry.lastAccessed / 1_000_000;
        } else {
          score = entry.lastAccessed;
        }
        if (score < bestScore) {
          bestScore = score;
          candidateKey = key;
          candidateEntry = entry;
        }
      }
    }

    if (candidateKey === undefined || !candidateEntry) {
      return false;
    }

    this.entries.delete(candidateKey);
    this.totalCost -= candidateEntry.cost;
    this.stats.evictions += 1;
    return true;
  }
}

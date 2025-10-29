import { invoke as tauriInvoke } from '@tauri-apps/api/tauri';
import { errorStore } from '$lib/stores/errorStore';
import type { ConnectionEvent, ConnectionHealthSummary } from '$lib/types';
import {
  cacheConnectionSummary,
  cacheConnectionTimeline,
  cacheCountryLookup,
  connectionSummaryCache,
  connectionTimelineCache,
  countryLookupCache,
  warmupCaches,
} from '../cache';

const RETRYABLE_PATTERNS = [
  /Network.+unreachable/i,
  /Network.+timeout/i,
  /operation timed out/i,
  /temporarily unavailable/i,
  /rate limit/i,
  /os error 11/i,
];

function shouldRetry(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error ?? '');
  return RETRYABLE_PATTERNS.some((pattern) => pattern.test(message));
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

let token: string | null = null;

if (typeof window !== 'undefined') {
  void warmupCaches().catch((error) => {
    console.warn('Cache warmup failed', error);
  });
}

export async function ensureToken(force = false): Promise<string> {
  if (!force && token) return token;
  token = await tauriInvoke<string>('request_token');
  return token;
}

export async function invoke<T = any>(
  cmd: string,
  args: Record<string, any> = {},
  retried = false,
  attempt = 0
): Promise<T> {
  const t = await ensureToken(retried && attempt === 0);
  try {
    return await tauriInvoke<T>(cmd, { token: t, ...args });
  } catch (err: any) {
    if (err && err.toString().includes('Invalid session token')) {
      if (retried) {
        errorStore.set(new Error('Session expired. Please retry.'));
      } else {
        await ensureToken(true);
        return invoke<T>(cmd, args, true);
      }
    }
    if (shouldRetry(err) && attempt < 3) {
      const nextAttempt = attempt + 1;
      const backoff = Math.min(500 * 2 ** attempt, 3000);
      const jitter = Math.random() * 150;
      await delay(backoff + jitter);
      return invoke<T>(cmd, args, retried, nextAttempt);
    }
    throw err;
  }
}

export function lookupCountry(ip: string) {
  const normalised = ip.trim();
  if (!normalised) {
    return Promise.resolve('');
  }
  const cached = countryLookupCache.get(normalised);
  if (cached) {
    return Promise.resolve(cached);
  }
  return invoke('lookup_country', { ip: normalised }).then((result) => {
    cacheCountryLookup(normalised, result, { ttlMs: 86_400_000 });
    return result;
  });
}

export function getConnectionTimeline(limit?: number) {
  const args = typeof limit === 'number' ? { limit } : {};
  const cacheKey = `timeline:${typeof limit === 'number' ? limit : 'default'}`;
  const cached = connectionTimelineCache.get(cacheKey);
  if (cached) {
    return Promise.resolve(cached);
  }
  return invoke<ConnectionEvent[]>('get_connection_timeline', args).then((events) => {
    cacheConnectionTimeline(cacheKey, events, { ttlMs: 25_000 });
    return events;
  });
}

export function getConnectionHealthSummary() {
  const cacheKey = 'summary:default';
  const cached = connectionSummaryCache.get(cacheKey);
  if (cached) {
    return Promise.resolve(cached);
  }
  return invoke<ConnectionHealthSummary>('get_connection_health_summary').then((summary) => {
    cacheConnectionSummary(cacheKey, summary, { ttlMs: 20_000 });
    return summary;
  });
}

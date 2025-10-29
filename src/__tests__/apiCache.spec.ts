import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

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
  };
}

describe('API cache integration', () => {
  let invokeMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.resetModules();
    invokeMock = vi.fn(async (cmd: string) => {
      if (cmd === 'request_token') {
        return 'token-test';
      }
      if (cmd === 'get_connection_timeline') {
        return [
          {
            timestamp: new Date().toISOString(),
            status: 'CONNECTED',
            message: null,
            detail: null,
            retryCount: 0,
            latencyMs: 10,
            memoryBytes: 2048,
            circuitCount: 2,
          },
        ];
      }
      if (cmd === 'get_connection_health_summary') {
        return {
          totalEvents: 5,
          connectedEvents: 4,
          errorEvents: 1,
          disconnectEvents: 0,
          lastEvent: null,
          lastConnectedAt: null,
          lastErrorAt: null,
          currentUptimeSeconds: 120,
          longestUptimeSeconds: 600,
          availabilityPercent: 92.5,
          retryAttemptsLastHour: 1,
        };
      }
      if (cmd === 'lookup_country') {
        return 'DE';
      }
      throw new Error(`Unexpected command ${cmd}`);
    });

    (globalThis as any).window = {
      localStorage: createStorage(),
    };

    vi.doMock('@tauri-apps/api/tauri', () => ({
      invoke: invokeMock,
    }));
  });

  afterEach(() => {
    vi.doUnmock('@tauri-apps/api/tauri');
  });

  it('memoizes timeline and summary responses and reuses token', async () => {
    const api = await import('../lib/api');
    const cacheModule = await import('../cache');

    const timeline1 = await api.getConnectionTimeline();
    const timeline2 = await api.getConnectionTimeline();
    expect(timeline1).toEqual(timeline2);

    const summary1 = await api.getConnectionHealthSummary();
    const summary2 = await api.getConnectionHealthSummary();
    expect(summary1).toEqual(summary2);

    const country1 = await api.lookupCountry('1.1.1.1');
    const country2 = await api.lookupCountry('1.1.1.1');
    expect(country1).toBe(country2);

    const timelineCalls = invokeMock.mock.calls.filter(
      (call) => call[0] === 'get_connection_timeline'
    );
    const summaryCalls = invokeMock.mock.calls.filter(
      (call) => call[0] === 'get_connection_health_summary'
    );
    const countryCalls = invokeMock.mock.calls.filter(
      (call) => call[0] === 'lookup_country'
    );
    const tokenCalls = invokeMock.mock.calls.filter((call) => call[0] === 'request_token');
    expect(timelineCalls).toHaveLength(1);
    expect(summaryCalls).toHaveLength(1);
    expect(countryCalls).toHaveLength(1);
    expect(tokenCalls).toHaveLength(1);

    cacheModule.invalidateConnectionCaches();
    await api.getConnectionTimeline();
    const timelineAfter = invokeMock.mock.calls.filter(
      (call) => call[0] === 'get_connection_timeline'
    );
    expect(timelineAfter).toHaveLength(2);
  });
});

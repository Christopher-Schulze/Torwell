import { writable, get } from 'svelte/store';

export type ConnectionActionKey =
  | 'connect'
  | 'disconnect'
  | 'build_circuit'
  | 'new_identity';

export type QueueExecution<T> =
  | { status: 'skipped'; reason: 'duplicate' }
  | { status: 'completed'; result: T };

export interface QueueState {
  pending: ConnectionActionKey | null;
  queueDepth: number;
  queuedKeys: ConnectionActionKey[];
  lastError: { key: ConnectionActionKey; message: string } | null;
  lastSuccess: { key: ConnectionActionKey; at: number } | null;
}

type QueueTask<T> = () => Promise<T>;

interface QueueEntry<T> {
  key: ConnectionActionKey;
  task: QueueTask<T>;
  resolve: (value: QueueExecution<T>) => void;
  reject: (reason?: unknown) => void;
}

const INITIAL_STATE: QueueState = {
  pending: null,
  queueDepth: 0,
  queuedKeys: [],
  lastError: null,
  lastSuccess: null,
};

function createConnectionQueue() {
  const state = writable<QueueState>({ ...INITIAL_STATE });

  const queue: QueueEntry<unknown>[] = [];
  const enqueued = new Set<ConnectionActionKey>();
  let active: ConnectionActionKey | null = null;

  const setQueuedKeys = () => {
    state.update((current) => ({
      ...current,
      queueDepth: queue.length,
      queuedKeys: queue.map((entry) => entry.key),
    }));
  };

  const resetActive = () => {
    state.update((current) => ({
      ...current,
      pending: null,
      queueDepth: queue.length,
      queuedKeys: queue.map((entry) => entry.key),
    }));
  };

  const runNext = () => {
    if (active !== null) {
      return;
    }
    const entry = queue.shift();
    if (!entry) {
      resetActive();
      return;
    }

    enqueued.delete(entry.key);
    active = entry.key;
    state.update((current) => ({
      ...current,
      pending: entry.key,
      queueDepth: queue.length,
      queuedKeys: queue.map((item) => item.key),
    }));

    (async () => {
      try {
        const result = await entry.task();
        state.update((current) => ({
          ...current,
          lastSuccess: { key: entry.key, at: Date.now() },
          lastError: current.lastError && current.lastError.key === entry.key ? null : current.lastError,
        }));
        entry.resolve({ status: 'completed', result });
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error ?? '');
        state.update((current) => ({
          ...current,
          lastError: { key: entry.key, message },
        }));
        entry.reject(error);
      } finally {
        active = null;
        resetActive();
        runNext();
      }
    })();
  };

  const run = <T>(key: ConnectionActionKey, task: QueueTask<T>): Promise<QueueExecution<T>> => {
    if (active === key || enqueued.has(key)) {
      return Promise.resolve({ status: 'skipped', reason: 'duplicate' });
    }

    return new Promise<QueueExecution<T>>((resolve, reject) => {
      const entry: QueueEntry<T> = { key, task, resolve, reject };
      queue.push(entry as QueueEntry<unknown>);
      enqueued.add(key);
      setQueuedKeys();
      runNext();
    });
  };

  const clearError = (key?: ConnectionActionKey) => {
    state.update((current) => {
      if (!current.lastError) {
        return current;
      }
      if (!key || current.lastError.key === key) {
        return { ...current, lastError: null };
      }
      return current;
    });
  };

  const statusFor = (key: ConnectionActionKey): 'idle' | 'queued' | 'active' => {
    const current = get(state);
    if (current.pending === key) {
      return 'active';
    }
    if (current.queuedKeys.includes(key)) {
      return 'queued';
    }
    return 'idle';
  };

  const reset = () => {
    queue.splice(0, queue.length);
    enqueued.clear();
    active = null;
    state.set({ ...INITIAL_STATE });
  };

  return {
    subscribe: state.subscribe,
    run,
    clearError,
    statusFor,
    reset,
  };
}

export const connectionQueue = createConnectionQueue();

import { writable } from 'svelte/store';

export interface Toast {
  id: number;
  message: string;
  type?: 'error' | 'info';
}

export const toasts = writable<Toast[]>([]);

export function addToast(message: string, type: 'error' | 'info' = 'info', duration = 3000) {
  const id = Date.now();
  toasts.update((ts) => [...ts, { id, message, type }]);
  setTimeout(() => {
    toasts.update((ts) => ts.filter((t) => t.id !== id));
  }, duration);
}

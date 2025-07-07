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

export type ErrorToastType = 'dns' | 'traceroute' | 'connection';

export function addErrorToast(kind: ErrorToastType, detail?: string, duration = 5000) {
  let message = '';
  switch (kind) {
    case 'dns':
      message = 'DNS lookup failed';
      break;
    case 'traceroute':
      message = 'Traceroute failed';
      break;
    case 'connection':
      message = 'Connection error';
      break;
    default:
      message = 'Error';
  }
  if (detail) {
    message += `: ${detail}`;
  }
  addToast(message, 'error', duration);
}

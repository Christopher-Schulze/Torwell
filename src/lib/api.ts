import { invoke as tauriInvoke } from '@tauri-apps/api/tauri';
import { errorStore } from '$lib/components/AppErrorBoundary.svelte';

let token: string | null = null;

export async function ensureToken(): Promise<string> {
  if (token) return token;
  token = await tauriInvoke<string>('request_token');
  return token;
}

export async function invoke(cmd: string, args: Record<string, any> = {}) {
  const t = await ensureToken();
  try {
    return await tauriInvoke(cmd, { token: t, ...args });
  } catch (err: any) {
    if (err && err.toString().includes('Invalid session token')) {
      token = null;
      errorStore.set(new Error('Session expired. Please retry.'));
    }
    throw err;
  }
}

import { invoke as tauriInvoke } from '@tauri-apps/api/tauri';
import { errorStore } from '$lib/stores/errorStore';

let token: string | null = null;

export async function ensureToken(force = false): Promise<string> {
  if (!force && token) return token;
  token = await tauriInvoke<string>('request_token');
  return token;
}

export async function invoke<T = any>(
  cmd: string,
  args: Record<string, any> = {},
  retried = false
): Promise<T> {
  const t = await ensureToken();
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
    throw err;
  }
}

export function lookupCountry(ip: string) {
  return invoke('lookup_country', { ip }) as Promise<string>;
}

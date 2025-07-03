import { invoke as tauriInvoke } from '@tauri-apps/api/tauri';

let token: string | null = null;

export async function ensureToken(): Promise<string> {
  if (token) return token;
  token = await tauriInvoke<string>('request_token');
  return token;
}

export async function invoke(cmd: string, args: Record<string, any> = {}) {
  const t = await ensureToken();
  return tauriInvoke(cmd, { token: t, ...args });
}

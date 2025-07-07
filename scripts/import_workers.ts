import { invoke } from '@tauri-apps/api/tauri';

export async function importWorkers(content: string, token: string = '') {
  const workers = content
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
  await invoke('set_worker_config', { workers, token });
  return workers.length;
}

export async function importWorkersFromFile(path: string, token = '') {
  const { readFileSync } = await import('fs');
  const content = readFileSync(path, 'utf-8');
  return importWorkers(content, token);
}

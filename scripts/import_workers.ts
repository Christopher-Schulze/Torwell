#!/usr/bin/env bun
import { invoke } from '@tauri-apps/api/tauri';
import { readFileSync } from 'fs';

export async function importWorkers(content: string, token: string = '') {
  const workers = content
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
  await invoke('set_worker_config', { workers, token });
  return workers.length;
}

async function main() {
  const file = process.argv[2];
  const token = process.argv[3] ?? '';
  if (!file) {
    console.error('Usage: import_workers.ts <file> [token]');
    process.exit(1);
  }
  const content = readFileSync(file, 'utf-8');
  const count = await importWorkers(content, token);
  console.log(`Imported ${count} workers`);
}

main();

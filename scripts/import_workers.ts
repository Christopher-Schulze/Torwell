#!/usr/bin/env bun
import { invoke } from '@tauri-apps/api/tauri';
import { readFileSync } from 'fs';

async function main() {
  const file = process.argv[2];
  const token = process.argv[3] ?? '';
  if (!file) {
    console.error('Usage: import_workers.ts <file> [token]');
    process.exit(1);
  }
  const content = readFileSync(file, 'utf-8');
  const workers = content
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
  await invoke('set_worker_config', { workers, token });
  console.log(`Imported ${workers.length} workers`);
}

main();

#!/usr/bin/env bun
import { importWorkersFromFile } from './import_workers.ts';

async function main() {
  const file = process.argv[2];
  const token = process.argv[3] ?? '';
  if (!file) {
    console.error('Usage: import_workers_cli.ts <file> [token]');
    process.exit(1);
  }
  const count = await importWorkersFromFile(file, token);
  console.log(`Imported ${count} workers`);
}

main();

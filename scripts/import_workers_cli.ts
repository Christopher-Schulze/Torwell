#!/usr/bin/env bun
import { importWorkersFromFile, parseWorkerList } from './import_workers.ts';

async function main() {
  const file = process.argv[2];
  const token = process.argv[3] ?? '';
  if (!file) {
    console.error('Usage: import_workers_cli.ts <file> [token]');
    process.exit(1);
  }
  const { readFileSync } = await import('fs');
  const content = readFileSync(file, 'utf-8');
  const { invalid } = parseWorkerList(content);
  const result = await importWorkersFromFile(file, token);
  if (invalid.length > 0) {
    console.warn(`Ignored ${invalid.length} invalid entries`);
  }
  console.log(`Imported ${result.imported} workers`);
}

main();

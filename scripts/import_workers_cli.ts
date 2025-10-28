#!/usr/bin/env bun
import { importWorkersFromFile, parseWorkerList } from "./import_workers.ts";

async function main() {
  const file = process.argv[2];
  const token = process.argv[3] ?? "";
  if (!file) {
    console.error("Usage: import_workers_cli.ts <file> [token]");
    process.exit(1);
  }
  try {
    const { readFileSync } = await import("fs");
    const content = readFileSync(file, "utf-8");
    const { invalid, duplicates, migrated } = parseWorkerList(content);
    const result = await importWorkersFromFile(file, token);
    if (invalid.length > 0) {
      console.warn(
        `Ignored ${invalid.length} invalid URLs: ${invalid.join(", ")}`,
      );
    }
    if (duplicates.length > 0) {
      console.warn(
        `Ignored ${duplicates.length} duplicate entries: ${duplicates.join(", ")}`,
      );
    }
    if (migrated.length > 0) {
      console.warn(
        `Upgraded ${migrated.length} HTTP entries to HTTPS: ${migrated.join(", ")}`,
      );
    }
    console.log(`Imported ${result.imported} workers`);
  } catch (err) {
    console.error(
      `Failed to import workers: ${err instanceof Error ? err.message : err}`,
    );
    process.exit(1);
  }
}

main();

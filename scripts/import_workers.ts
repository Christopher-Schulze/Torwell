import { invoke } from "@tauri-apps/api/tauri";

export type ImportResult = {
  workers: string[];
  invalid: string[];
  duplicates: string[];
};

export function parseWorkerList(content: string): ImportResult {
  const seen = new Set<string>();
  const workers: string[] = [];
  const invalid: string[] = [];
  const duplicates: string[] = [];
  for (const line of content.split(/\r?\n/)) {
    const url = line.trim();
    if (!url) continue;
    try {
      new URL(url);
      if (!seen.has(url)) {
        seen.add(url);
        workers.push(url);
      } else {
        duplicates.push(url);
      }
    } catch {
      invalid.push(url);
    }
  }
  return { workers, invalid, duplicates };
}

export async function importWorkers(content: string, token = "") {
  const { workers, invalid, duplicates } = parseWorkerList(content);
  const isMobile = typeof window !== "undefined" && (window as any).Capacitor;
  try {
    if (isMobile) {
      const res = await fetch("http://127.0.0.1:1421/workers", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ workers, token }),
      });
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}`);
      }
      const val = await fetch("http://127.0.0.1:1421/validate");
      if (!val.ok) {
        throw new Error(`HTTP ${val.status}`);
      }
      const ok = await val.json();
      if (!ok) {
        throw new Error("Invalid worker token");
      }
    } else {
      await invoke("set_worker_config", { workers, token });
      const valid = await invoke<boolean>("validate_worker_token");
      if (!valid) {
        throw new Error("Invalid worker token");
      }
    }
    return { imported: workers.length, invalid, duplicates };
  } catch (err) {
    console.error("Failed to import workers:", err);
    throw err;
  }
}

export async function importWorkersFromFile(path: string, token = "") {
  try {
    const { readFileSync } = await import("fs");
    const content = readFileSync(path, "utf-8");
    return await importWorkers(content, token);
  } catch (err) {
    console.error("Failed to read worker file:", err);
    throw err;
  }
}

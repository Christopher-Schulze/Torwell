import { invoke } from "@tauri-apps/api/tauri";

export async function importWorkers(content: string, token: string = "") {
  const workers = content
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
  const isMobile = typeof window !== "undefined" && (window as any).Capacitor;
  if (isMobile) {
    await fetch("http://127.0.0.1:1421/workers", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ workers, token }),
    });
  } else {
    await invoke("set_worker_config", { workers, token });
  }
  return workers.length;
}

export async function importWorkersFromFile(path: string, token = "") {
  const { readFileSync } = await import("fs");
  const content = readFileSync(path, "utf-8");
  return importWorkers(content, token);
}

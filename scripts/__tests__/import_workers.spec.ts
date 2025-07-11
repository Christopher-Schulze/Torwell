import { describe, it, expect, vi, beforeEach } from "vitest";
import fs from "fs";
import path from "path";

vi.mock("@tauri-apps/api/tauri", () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === "validate_worker_token") return true;
    return undefined;
  }),
}));

import { parseWorkerList, importWorkers, importWorkersFromFile } from "../import_workers.ts";
import { invoke } from "@tauri-apps/api/tauri";

describe("parseWorkerList", () => {
  it("filters invalid urls and duplicates", () => {
    const input = [
      "https://a.com",
      "invalid",
      "https://b.com",
      "https://a.com",
      "",
    ].join("\n");
    const result = parseWorkerList(input);
    expect(result.workers).toEqual(["https://a.com", "https://b.com"]);
    expect(result.invalid).toEqual(["invalid"]);
    expect(result.duplicates).toEqual(["https://a.com"]);
  });

  it("handles large lists", () => {
    let lines: string[] = [];
    for (let i = 0; i < 100; i++) lines.push(`https://v${i}.com`);
    for (let i = 0; i < 50; i++) lines.push(`bad-${i}`);
    for (let i = 0; i < 50; i++) lines.push(`https://v${i}.com`);
    const { workers, invalid, duplicates } = parseWorkerList(lines.join("\n"));
    expect(workers.length).toBe(100);
    expect(invalid.length).toBe(50);
    expect(duplicates.length).toBe(50);
  });
});

describe("import functions", () => {
  beforeEach(() => vi.clearAllMocks());

  it("invokes backend with parsed list", async () => {
    const content = "https://x.com\ninvalid\nhttps://y.com";
    const res = await importWorkers(content, "tok");
    expect(invoke).toHaveBeenCalledWith("set_worker_config", {
      workers: ["https://x.com", "https://y.com"],
      token: "tok",
    });
    expect(invoke).toHaveBeenCalledWith("validate_worker_token");
    expect(res.imported).toBe(2);
    expect(res.invalid).toEqual(["invalid"]);
  });

  it("reads from file and forwards token", async () => {
    const file = path.join(process.cwd(), "tmp-workers.txt");
    fs.writeFileSync(file, "https://a\nhttps://b\n");
    await importWorkersFromFile(file, "secret");
    expect(invoke).toHaveBeenCalledWith("set_worker_config", {
      workers: ["https://a", "https://b"],
      token: "secret",
    });
    expect(invoke).toHaveBeenCalledWith("validate_worker_token");
    fs.unlinkSync(file);
  });
});

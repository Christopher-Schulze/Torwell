import { describe, it, expect, vi, beforeEach } from "vitest";
import fs from "fs";
import path from "path";

vi.mock("@tauri-apps/api/tauri", () => ({ invoke: vi.fn(async () => undefined) }));

beforeEach(() => vi.clearAllMocks());

it("runs CLI and prints summary", async () => {
  const file = path.join(process.cwd(), "cli-workers.txt");
  fs.writeFileSync(file, "https://a\ninvalid\nhttps://b\nhttps://a\n");
  const log = vi.spyOn(console, "log").mockImplementation(() => {});
  const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
  const orig = process.argv;
  process.argv = ["bun", "import_workers_cli.ts", file, "tok"];
  await import("../import_workers_cli.ts?test" + Date.now());
  process.argv = orig;
  expect(log).toHaveBeenCalledWith("Imported 2 workers");
  expect(warn).toHaveBeenCalledTimes(2);
  fs.unlinkSync(file);
});

#!/usr/bin/env node
import fs from "fs";
import path from "path";

const root = path.resolve(
  path.dirname(decodeURIComponent(new URL(import.meta.url).pathname)),
  "..",
  "..",
);
const workerDir = path.join(root, "cf worker");
const configPath = path.join(root, "src-tauri", "app_config.json");

let failed = false;

function ensureHttps(url, context) {
  try {
    const parsed = new URL(url);
    if (parsed.protocol !== "https:") {
      console.error(`[TLS] ${context}: ${url} does not use HTTPS`);
      failed = true;
    }
  } catch (err) {
    console.error(`[TLS] ${context}: ${url} is not a valid URL (${err})`);
    failed = true;
  }
}

if (fs.existsSync(workerDir)) {
  for (const file of fs.readdirSync(workerDir)) {
    if (!file.endsWith(".txt")) continue;
    const content = fs.readFileSync(path.join(workerDir, file), "utf-8");
    for (const line of content.split(/\r?\n/)) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith("#") || trimmed.startsWith("//")) continue;
      if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
        ensureHttps(trimmed, `Worker list ${file}`);
        continue;
      }
      if (/^[\w.-]+(:\d+)?(\/[^\s]*)?$/.test(trimmed)) {
        ensureHttps(`https://${trimmed}`, `Worker list ${file}`);
      }
    }
  }
}

if (fs.existsSync(configPath)) {
  try {
    const cfg = JSON.parse(fs.readFileSync(configPath, "utf-8"));
    if (cfg.cert_url) {
      ensureHttps(cfg.cert_url, "app_config.cert_url");
    }
    if (cfg.fallback_cert_url) {
      ensureHttps(cfg.fallback_cert_url, "app_config.fallback_cert_url");
    }
  } catch (err) {
    console.error(`[TLS] Failed to parse app_config.json: ${err}`);
    failed = true;
  }
}

if (failed) {
  process.exit(1);
}

console.log("[TLS] All URLs use HTTPS");

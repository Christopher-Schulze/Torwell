// lib/database.ts
import Dexie, { type Table } from "dexie";
import { invoke } from "@tauri-apps/api/tauri";

interface MetaEntry {
  id: string;
  value: string;
}

// Utility helpers for base64 conversion
function bufToB64(buf: ArrayBuffer): string {
  const bytes = new Uint8Array(buf);
  let str = "";
  for (const b of bytes) str += String.fromCharCode(b);
  return btoa(str);
}

function b64ToBuf(b64: string): ArrayBuffer {
  const bin = atob(b64);
  const buf = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) buf[i] = bin.charCodeAt(i);
  return buf.buffer;
}

// Load or create the persistent AES key
async function loadKey(db: AppDatabase): Promise<CryptoKey> {
  let keyB64: string | null = null;
  try {
    keyB64 = await invoke<string | null>("get_secure_key");
  } catch {
    // command may not be available
  }

  if (!keyB64) {
    const entry = await db.meta.get("aes-key");
    if (entry) {
      keyB64 = entry.value;
      try {
        await invoke("set_secure_key", { value: keyB64 });
        await db.meta.delete("aes-key");
      } catch {
        // fallback: keep key in IndexedDB if secure storage fails
      }
    }
  }

  if (!keyB64) {
    const raw = crypto.getRandomValues(new Uint8Array(32));
    keyB64 = bufToB64(raw.buffer);
    try {
      await invoke("set_secure_key", { value: keyB64 });
    } catch {
      await db.meta.put({ id: "aes-key", value: keyB64 });
    }
  }

  return crypto.subtle.importKey(
    "raw",
    b64ToBuf(keyB64),
    "AES-GCM",
    true,
    ["encrypt", "decrypt"]
  );
}

async function encryptString(db: AppDatabase, value: string): Promise<string> {
  const key = await loadKey(db);
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encoded = new TextEncoder().encode(value);
  const cipher = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, encoded);
  const combined = new Uint8Array(iv.byteLength + cipher.byteLength);
  combined.set(iv, 0);
  combined.set(new Uint8Array(cipher), iv.byteLength);
  return bufToB64(combined.buffer);
}

async function decryptString(db: AppDatabase, value: string): Promise<string> {
  const data = new Uint8Array(b64ToBuf(value));
  const iv = data.slice(0, 12);
  const cipher = data.slice(12);
  const key = await loadKey(db);
  const plain = await crypto.subtle.decrypt({ name: "AES-GCM", iv }, key, cipher);
  return new TextDecoder().decode(plain);
}

async function encryptFields(db: AppDatabase, obj: Partial<Settings>): Promise<void> {
  if (obj.bridges) {
    obj.bridges = await Promise.all(obj.bridges.map((b) => encryptString(db, b)));
  }
  if (obj.exitCountry != null) {
    obj.exitCountry = await encryptString(db, obj.exitCountry);
  }
}

async function decryptFields(db: AppDatabase, obj: Settings | undefined): Promise<Settings | undefined> {
  if (!obj) return obj;
  if (obj.bridges) {
    obj.bridges = await Promise.all(obj.bridges.map((b) => decryptString(db, b)));
  }
  if (obj.exitCountry != null) {
    obj.exitCountry = await decryptString(db, obj.exitCountry);
  }
  return obj;
}

export interface Settings {
  id?: number;
  workerList: string[];
  torrcConfig: string;
  exitCountry?: string | null;
  bridges?: string[];
  maxLogLines?: number;
}

export class AppDatabase extends Dexie {
  settings!: Table<Settings>;

  meta!: Table<MetaEntry>;

  constructor() {
    super("Torwell84DatabaseV2");
    this.version(1).stores({
      settings:
        "++id, workerList, torrcConfig, exitCountry, bridges, maxLogLines",
    });
    this.version(2).stores({ meta: "&id" });

    this.settings.hook("creating", async (_pk, obj) => {
      await encryptFields(this, obj);
    });
    this.settings.hook("updating", async (mods) => {
      await encryptFields(this, mods as Partial<Settings>);
    });
    this.settings.hook("reading", (obj) => decryptFields(this, obj));
  }
}

export const db = new AppDatabase();

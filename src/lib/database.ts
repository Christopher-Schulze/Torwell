// lib/database.ts
import Dexie, { type Table } from "dexie";

const SECRET = "torwell-key";

function xor(str: string): string {
  return Array.from(str)
    .map((c, i) =>
      String.fromCharCode(c.charCodeAt(0) ^ SECRET.charCodeAt(i % SECRET.length))
    )
    .join("");
}

function encryptString(value: string): string {
  return btoa(xor(value));
}

function decryptString(value: string): string {
  return xor(atob(value));
}

function encryptFields(obj: Partial<Settings>): void {
  if (obj.bridges) {
    obj.bridges = obj.bridges.map((b) => encryptString(b));
  }
  if (obj.exitCountry != null) {
    obj.exitCountry = encryptString(obj.exitCountry);
  }
}

function decryptFields(obj: Settings | undefined): Settings | undefined {
  if (!obj) return obj;
  if (obj.bridges) {
    obj.bridges = obj.bridges.map((b) => decryptString(b));
  }
  if (obj.exitCountry != null) {
    obj.exitCountry = decryptString(obj.exitCountry);
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

  constructor() {
    super("Torwell84DatabaseV2");
    this.version(1).stores({
      settings:
        "++id, workerList, torrcConfig, exitCountry, bridges, maxLogLines", // Primary key and indexed props
    });

    this.settings.hook("creating", (_pk, obj) => {
      encryptFields(obj);
    });
    this.settings.hook("updating", (mods) => {
      encryptFields(mods as Partial<Settings>);
    });
    this.settings.hook("reading", (obj) => decryptFields(obj));
  }
}

export const db = new AppDatabase();

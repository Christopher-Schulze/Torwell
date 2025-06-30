// lib/database.ts
import Dexie, { type Table } from 'dexie';

export interface Settings {
  id?: number;
  workerList: string[];
  torrcConfig: string;
  exitCountry?: string | null;
  bridges?: string[];
}

export class AppDatabase extends Dexie {
  settings!: Table<Settings>;

  constructor() {
    super('Torwell84DatabaseV2');
    this.version(1).stores({
      settings: '++id, workerList, torrcConfig, exitCountry, bridges', // Primary key and indexed props
    });
  }
}

export const db = new AppDatabase();
import '@testing-library/jest-dom/vitest';
import Dexie from 'dexie';
import { indexedDB, IDBKeyRange } from 'fake-indexeddb';

// Provide a minimal Tauri IPC shim so modules using invoke() don't fail
(globalThis as any).window ??= globalThis;
(globalThis as any).window.__TAURI_IPC__ = () => {};

// Mock navigator for @tauri-apps/api helpers
(globalThis as any).navigator = { appVersion: 'test' };

// Configure Dexie to use the in-memory IndexedDB implementation
Dexie.dependencies.indexedDB = indexedDB as any;
Dexie.dependencies.IDBKeyRange = IDBKeyRange as any;

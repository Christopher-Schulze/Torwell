import { describe, it, beforeEach, expect, vi } from 'vitest';
import fs from 'fs';
import path from 'path';

vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

const logFile = path.join(process.cwd(), 'torwell.log');

vi.mock('$lib/database', () => {
  const settings = { put: vi.fn(), get: vi.fn().mockResolvedValue(undefined) };
  return { db: { settings } };
});

vi.mock('@tauri-apps/api/tauri', () => {
  let limit = 1000;
  return {
    invoke: vi.fn(async (cmd: string, args: any) => {
      if (cmd === 'set_log_limit') {
        limit = args.limit;
        const lines = fs
          .readFileSync(logFile, 'utf8')
          .split(/\n/)
          .filter(Boolean);
        if (lines.length > limit) {
          fs.writeFileSync(logFile, lines.slice(lines.length - limit).join('\n'));
        }
        return;
      }
      if (cmd === 'get_logs') {
        const lines = fs
          .readFileSync(logFile, 'utf8')
          .split(/\n/)
          .filter(Boolean)
          .slice(-limit);
        return lines.map((l) => JSON.parse(l));
      }
      if (cmd === 'get_log_file_path') {
        return logFile;
      }
      if (cmd === 'clear_logs') {
        fs.writeFileSync(logFile, '');
        return;
      }
      return;
    }),
  };
});

describe('log limit propagation', () => {
  beforeEach(() => {
    fs.writeFileSync(logFile, '');
    vi.clearAllMocks();
  });

  it('trims torwell.log when log limit is updated', async () => {
    const logs = [1, 2, 3, 4, 5].map((n) =>
      JSON.stringify({ level: 'INFO', timestamp: `t${n}`, message: `m${n}` })
    );
    fs.writeFileSync(logFile, logs.join('\n'));

    const { uiStore } = await import('../lib/stores/uiStore');
    await uiStore.actions.setLogLimit(3);

    const fileLines = fs.readFileSync(logFile, 'utf8').split(/\n/).filter(Boolean);
    expect(fileLines.length).toBe(3);
    expect(fileLines[0]).toContain('m3');

    const { invoke } = await import('@tauri-apps/api/tauri');
    const loaded = await invoke<any[]>('get_logs');
    expect(loaded.length).toBe(3);
    expect(loaded[0].message).toBe('m3');
  });
});

import { test, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import fs from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { performance } from 'node:perf_hooks';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, '../..');
const ARTIFACT_DIR = path.resolve(__dirname, '../artifacts');
const BASELINE_PATH = path.resolve(__dirname, '../baselines/playwright-baseline.json');
const PORT = Number.parseInt(process.env.PLAYWRIGHT_BENCH_PORT ?? '4173', 10);

let devServer: ReturnType<typeof spawn> | null = null;

async function waitForServer(url: string, timeoutMs: number) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url, { method: 'GET' });
      if (response.ok) {
        return;
      }
    } catch (err) {
      // swallow errors until timeout
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error(`Timed out waiting for dev server at ${url}`);
}

test.beforeAll(async () => {
  devServer = spawn('npm', ['run', 'dev', '--', '--host', '127.0.0.1', '--port', String(PORT)], {
    cwd: REPO_ROOT,
    env: {
      ...process.env,
      BROWSER: 'none',
      FORCE_COLOR: '1',
      PLAYWRIGHT_BENCH: '1',
    },
    stdio: 'pipe',
  });

  devServer.stderr?.on('data', (chunk) => {
    process.stderr.write(`[dev-server] ${chunk}`);
  });

  await waitForServer(`http://127.0.0.1:${PORT}`, 60_000);
});

test.afterAll(async () => {
  if (devServer) {
    devServer.kill('SIGINT');
    await new Promise((resolve) => {
      devServer?.once('exit', () => resolve(undefined));
      setTimeout(resolve, 5_000);
    });
  }
});

test('Dashboard first paint stays within baseline budget', async ({ page }) => {
  await fs.mkdir(ARTIFACT_DIR, { recursive: true });

  const start = performance.now();
  await page.goto('/', { waitUntil: 'networkidle' });
  const navMetrics = await page.evaluate(() => {
    const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming | undefined;
    const fcpEntry = performance.getEntriesByName('first-contentful-paint')[0] as PerformanceEntry | undefined;
    return {
      fcp: fcpEntry?.startTime ?? navigation?.domContentLoadedEventEnd ?? 0,
      domContentLoaded: navigation?.domContentLoadedEventEnd ?? 0,
      load: navigation?.loadEventEnd ?? 0,
    };
  });
  const totalElapsed = performance.now() - start;

  const metrics = {
    firstContentfulPaintMs: navMetrics.fcp,
    domContentLoadedMs: navMetrics.domContentLoaded,
    loadEventMs: navMetrics.load,
    totalElapsedMs: totalElapsed,
  };

  await fs.writeFile(
    path.join(ARTIFACT_DIR, 'playwright-latest.json'),
    JSON.stringify({ measuredAt: new Date().toISOString(), metrics }, null, 2),
    'utf8',
  );

  const baselineRaw = await fs.readFile(BASELINE_PATH, 'utf8');
  const baseline = JSON.parse(baselineRaw) as {
    fcp_ms: number;
    domContentLoaded_ms: number;
    load_ms: number;
    maxRegressionPct: number;
  };

  const allowed = (baselineValue: number) => baselineValue * (1 + baseline.maxRegressionPct / 100);

  expect(metrics.firstContentfulPaintMs).toBeLessThanOrEqual(allowed(baseline.fcp_ms));
  expect(metrics.domContentLoadedMs).toBeLessThanOrEqual(allowed(baseline.domContentLoaded_ms));
  expect(metrics.loadEventMs).toBeLessThanOrEqual(allowed(baseline.load_ms));
});

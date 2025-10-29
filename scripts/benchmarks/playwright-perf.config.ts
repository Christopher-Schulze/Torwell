import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './scripts/benchmarks/playwright',
  fullyParallel: false,
  retries: 0,
  timeout: 120_000,
  workers: 1,
  reporter: [['list'], ['json', { outputFile: 'scripts/benchmarks/artifacts/playwright-report.json' }]],
  use: {
    baseURL: 'http://127.0.0.1:4173',
    browserName: 'chromium',
    ...devices['Desktop Chrome'],
    headless: true,
    trace: 'off',
  },
});

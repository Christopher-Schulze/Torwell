import baseConfig from './vite.config.js';
import { defineConfig, mergeConfig } from 'vitest/config';

export default defineConfig(async ({ mode }) => {
  const base = await baseConfig({ mode: mode ?? 'test' });
  return mergeConfig(base, {
    test: {
      include: ['src/**/*.snapshot.test.ts'],
    },
  });
});

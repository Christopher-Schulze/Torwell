import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';
import Inspect from 'vite-plugin-inspect';

export default defineConfig(({ mode }) => {
  return {
    plugins: [sveltekit(), ...(mode === 'analyze' ? [Inspect({ build: true })] : [])],
    
    // Prevent Vite from obscuring Rust errors
    clearScreen: false,

    // Tauri expects a fixed port, fail if that port is not available
    server: {
      port: 1420,
      strictPort: true,
    },

    // to make use of `TAURI_DEBUG` and other env variables
    // https://tauri.app/v1/api/config#buildconfig.beforedevcommand
    envPrefix: ['VITE_', 'TAURI_'],
    build: {
      // Tauri supports es2021
      target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
      // don't minify for debug builds
      minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
      // produce sourcemaps for debug builds
      sourcemap: !!process.env.TAURI_DEBUG,
    },
    test: {
      environment: 'jsdom',
      globals: true,
      setupFiles: './src/setupTests.ts',
      include: ['**/*.{test,spec}.?(c|m)[jt]s?(x)', 'src/**/*.snapshot.test.ts'],
      exclude: [
        '**/node_modules/**',
        '**/dist/**',
        '**/cypress/**',
        '**/.{idea,git,cache,output,temp}/**',
        '**/{karma,rollup,webpack,vite,vitest,jest,ava,babel,nyc,cypress,tsup,build,eslint,prettier}.config.*',
        'src/__tests__/**',
        'scripts/__tests__/**',
        'scripts/benchmarks/**',
      ],
      coverage: {
        provider: 'v8',
        reporter: ['text', 'lcov', 'html'],
        reportsDirectory: './coverage/frontend',
        all: true,
        include: ['src/**/*.{ts,svelte}'],
        exclude: ['src/**/*.d.ts', 'src/**/*.stories.ts'],
      },
    },
  };
});

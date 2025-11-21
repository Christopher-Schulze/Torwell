// vite.config.js
import { sveltekit } from "file:///app/node_modules/@sveltejs/kit/src/exports/vite/index.js";
import { defineConfig } from "file:///app/node_modules/vitest/dist/config.js";
import Inspect from "file:///app/node_modules/vite-plugin-inspect/dist/index.mjs";
var vite_config_default = defineConfig(({ mode }) => {
  return {
    plugins: [sveltekit(), ...mode === "analyze" ? [Inspect({ build: true })] : []],
    // Prevent Vite from obscuring Rust errors
    clearScreen: false,
    // Tauri expects a fixed port, fail if that port is not available
    server: {
      port: 1420,
      strictPort: true
    },
    // to make use of `TAURI_DEBUG` and other env variables
    // https://tauri.app/v1/api/config#buildconfig.beforedevcommand
    envPrefix: ["VITE_", "TAURI_"],
    build: {
      // Tauri supports es2021
      target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
      // don't minify for debug builds
      minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
      // produce sourcemaps for debug builds
      sourcemap: !!process.env.TAURI_DEBUG
    },
    test: {
      environment: "jsdom",
      globals: true,
      setupFiles: "./src/setupTests.ts",
      include: ["**/*.{test,spec}.?(c|m)[jt]s?(x)", "src/**/*.snapshot.test.ts"],
      exclude: [
        "**/node_modules/**",
        "**/dist/**",
        "**/cypress/**",
        "**/.{idea,git,cache,output,temp}/**",
        "**/{karma,rollup,webpack,vite,vitest,jest,ava,babel,nyc,cypress,tsup,build,eslint,prettier}.config.*",
        "src/__tests__/**",
        "scripts/__tests__/**",
        "scripts/benchmarks/**"
      ],
      coverage: {
        provider: "v8",
        reporter: ["text", "lcov", "html"],
        reportsDirectory: "./coverage/frontend",
        all: true,
        include: ["src/**/*.{ts,svelte}"],
        exclude: ["src/**/*.d.ts", "src/**/*.stories.ts"]
      }
    }
  };
});
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcuanMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCIvYXBwXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ZpbGVuYW1lID0gXCIvYXBwL3ZpdGUuY29uZmlnLmpzXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ltcG9ydF9tZXRhX3VybCA9IFwiZmlsZTovLy9hcHAvdml0ZS5jb25maWcuanNcIjtpbXBvcnQgeyBzdmVsdGVraXQgfSBmcm9tICdAc3ZlbHRlanMva2l0L3ZpdGUnO1xuaW1wb3J0IHsgZGVmaW5lQ29uZmlnIH0gZnJvbSAndml0ZXN0L2NvbmZpZyc7XG5pbXBvcnQgSW5zcGVjdCBmcm9tICd2aXRlLXBsdWdpbi1pbnNwZWN0JztcblxuZXhwb3J0IGRlZmF1bHQgZGVmaW5lQ29uZmlnKCh7IG1vZGUgfSkgPT4ge1xuICByZXR1cm4ge1xuICAgIHBsdWdpbnM6IFtzdmVsdGVraXQoKSwgLi4uKG1vZGUgPT09ICdhbmFseXplJyA/IFtJbnNwZWN0KHsgYnVpbGQ6IHRydWUgfSldIDogW10pXSxcbiAgICBcbiAgICAvLyBQcmV2ZW50IFZpdGUgZnJvbSBvYnNjdXJpbmcgUnVzdCBlcnJvcnNcbiAgICBjbGVhclNjcmVlbjogZmFsc2UsXG5cbiAgICAvLyBUYXVyaSBleHBlY3RzIGEgZml4ZWQgcG9ydCwgZmFpbCBpZiB0aGF0IHBvcnQgaXMgbm90IGF2YWlsYWJsZVxuICAgIHNlcnZlcjoge1xuICAgICAgcG9ydDogMTQyMCxcbiAgICAgIHN0cmljdFBvcnQ6IHRydWUsXG4gICAgfSxcblxuICAgIC8vIHRvIG1ha2UgdXNlIG9mIGBUQVVSSV9ERUJVR2AgYW5kIG90aGVyIGVudiB2YXJpYWJsZXNcbiAgICAvLyBodHRwczovL3RhdXJpLmFwcC92MS9hcGkvY29uZmlnI2J1aWxkY29uZmlnLmJlZm9yZWRldmNvbW1hbmRcbiAgICBlbnZQcmVmaXg6IFsnVklURV8nLCAnVEFVUklfJ10sXG4gICAgYnVpbGQ6IHtcbiAgICAgIC8vIFRhdXJpIHN1cHBvcnRzIGVzMjAyMVxuICAgICAgdGFyZ2V0OiBwcm9jZXNzLmVudi5UQVVSSV9QTEFURk9STSA9PSAnd2luZG93cycgPyAnY2hyb21lMTA1JyA6ICdzYWZhcmkxMycsXG4gICAgICAvLyBkb24ndCBtaW5pZnkgZm9yIGRlYnVnIGJ1aWxkc1xuICAgICAgbWluaWZ5OiAhcHJvY2Vzcy5lbnYuVEFVUklfREVCVUcgPyAnZXNidWlsZCcgOiBmYWxzZSxcbiAgICAgIC8vIHByb2R1Y2Ugc291cmNlbWFwcyBmb3IgZGVidWcgYnVpbGRzXG4gICAgICBzb3VyY2VtYXA6ICEhcHJvY2Vzcy5lbnYuVEFVUklfREVCVUcsXG4gICAgfSxcbiAgICB0ZXN0OiB7XG4gICAgICBlbnZpcm9ubWVudDogJ2pzZG9tJyxcbiAgICAgIGdsb2JhbHM6IHRydWUsXG4gICAgICBzZXR1cEZpbGVzOiAnLi9zcmMvc2V0dXBUZXN0cy50cycsXG4gICAgICBpbmNsdWRlOiBbJyoqLyoue3Rlc3Qsc3BlY30uPyhjfG0pW2p0XXM/KHgpJywgJ3NyYy8qKi8qLnNuYXBzaG90LnRlc3QudHMnXSxcbiAgICAgIGV4Y2x1ZGU6IFtcbiAgICAgICAgJyoqL25vZGVfbW9kdWxlcy8qKicsXG4gICAgICAgICcqKi9kaXN0LyoqJyxcbiAgICAgICAgJyoqL2N5cHJlc3MvKionLFxuICAgICAgICAnKiovLntpZGVhLGdpdCxjYWNoZSxvdXRwdXQsdGVtcH0vKionLFxuICAgICAgICAnKiove2thcm1hLHJvbGx1cCx3ZWJwYWNrLHZpdGUsdml0ZXN0LGplc3QsYXZhLGJhYmVsLG55YyxjeXByZXNzLHRzdXAsYnVpbGQsZXNsaW50LHByZXR0aWVyfS5jb25maWcuKicsXG4gICAgICAgICdzcmMvX190ZXN0c19fLyoqJyxcbiAgICAgICAgJ3NjcmlwdHMvX190ZXN0c19fLyoqJyxcbiAgICAgICAgJ3NjcmlwdHMvYmVuY2htYXJrcy8qKicsXG4gICAgICBdLFxuICAgICAgY292ZXJhZ2U6IHtcbiAgICAgICAgcHJvdmlkZXI6ICd2OCcsXG4gICAgICAgIHJlcG9ydGVyOiBbJ3RleHQnLCAnbGNvdicsICdodG1sJ10sXG4gICAgICAgIHJlcG9ydHNEaXJlY3Rvcnk6ICcuL2NvdmVyYWdlL2Zyb250ZW5kJyxcbiAgICAgICAgYWxsOiB0cnVlLFxuICAgICAgICBpbmNsdWRlOiBbJ3NyYy8qKi8qLnt0cyxzdmVsdGV9J10sXG4gICAgICAgIGV4Y2x1ZGU6IFsnc3JjLyoqLyouZC50cycsICdzcmMvKiovKi5zdG9yaWVzLnRzJ10sXG4gICAgICB9LFxuICAgIH0sXG4gIH07XG59KTtcbiJdLAogICJtYXBwaW5ncyI6ICI7QUFBOEwsU0FBUyxpQkFBaUI7QUFDeE4sU0FBUyxvQkFBb0I7QUFDN0IsT0FBTyxhQUFhO0FBRXBCLElBQU8sc0JBQVEsYUFBYSxDQUFDLEVBQUUsS0FBSyxNQUFNO0FBQ3hDLFNBQU87QUFBQSxJQUNMLFNBQVMsQ0FBQyxVQUFVLEdBQUcsR0FBSSxTQUFTLFlBQVksQ0FBQyxRQUFRLEVBQUUsT0FBTyxLQUFLLENBQUMsQ0FBQyxJQUFJLENBQUMsQ0FBRTtBQUFBO0FBQUEsSUFHaEYsYUFBYTtBQUFBO0FBQUEsSUFHYixRQUFRO0FBQUEsTUFDTixNQUFNO0FBQUEsTUFDTixZQUFZO0FBQUEsSUFDZDtBQUFBO0FBQUE7QUFBQSxJQUlBLFdBQVcsQ0FBQyxTQUFTLFFBQVE7QUFBQSxJQUM3QixPQUFPO0FBQUE7QUFBQSxNQUVMLFFBQVEsUUFBUSxJQUFJLGtCQUFrQixZQUFZLGNBQWM7QUFBQTtBQUFBLE1BRWhFLFFBQVEsQ0FBQyxRQUFRLElBQUksY0FBYyxZQUFZO0FBQUE7QUFBQSxNQUUvQyxXQUFXLENBQUMsQ0FBQyxRQUFRLElBQUk7QUFBQSxJQUMzQjtBQUFBLElBQ0EsTUFBTTtBQUFBLE1BQ0osYUFBYTtBQUFBLE1BQ2IsU0FBUztBQUFBLE1BQ1QsWUFBWTtBQUFBLE1BQ1osU0FBUyxDQUFDLG9DQUFvQywyQkFBMkI7QUFBQSxNQUN6RSxTQUFTO0FBQUEsUUFDUDtBQUFBLFFBQ0E7QUFBQSxRQUNBO0FBQUEsUUFDQTtBQUFBLFFBQ0E7QUFBQSxRQUNBO0FBQUEsUUFDQTtBQUFBLFFBQ0E7QUFBQSxNQUNGO0FBQUEsTUFDQSxVQUFVO0FBQUEsUUFDUixVQUFVO0FBQUEsUUFDVixVQUFVLENBQUMsUUFBUSxRQUFRLE1BQU07QUFBQSxRQUNqQyxrQkFBa0I7QUFBQSxRQUNsQixLQUFLO0FBQUEsUUFDTCxTQUFTLENBQUMsc0JBQXNCO0FBQUEsUUFDaEMsU0FBUyxDQUFDLGlCQUFpQixxQkFBcUI7QUFBQSxNQUNsRDtBQUFBLElBQ0Y7QUFBQSxFQUNGO0FBQ0YsQ0FBQzsiLAogICJuYW1lcyI6IFtdCn0K

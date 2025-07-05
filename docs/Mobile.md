# Mobile Build

This directory contains the configuration for the Capacitor based mobile build.
The Svelte frontend is reused by pointing `webDir` to the compiled web assets.

## Build Steps

1. Run `task setup` once to install all dependencies.
2. Use the `Taskfile` targets to build the apps:

   ```bash
   task mobile:android  # Build Android APK
   task mobile:ios      # Build iOS project
   ```

   Each task performs the following:
   - `bun run build` generates the web assets in `build/`.
   - `cargo build --release --manifest-path src-tauri/Cargo.toml --features mobile` compiles the Rust backend so the HTTP bridge is included.
   - The Capacitor CLI then syncs and builds the native project.

## IPC Bridge

When compiled with the `mobile` feature, the Rust backend automatically launches
a small HTTP server listening on `http://127.0.0.1:1421`. The Capacitor shell
communicates with this server to control the Tor connection. Make sure requests
target this port when running the mobile app.

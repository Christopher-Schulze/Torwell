# Mobile Build

This directory contains the configuration for the Capacitor based mobile build.
The Svelte frontend is reused by pointing `webDir` to the compiled web assets.

## Build Scripts

Use the `Taskfile` targets to build the apps:

```bash
task mobile:android  # Build Android APK
task mobile:ios      # Build iOS project
```

The scripts under `mobile/scripts` run `bun run build` to generate the web
assets and then invoke Capacitor CLI to sync and build the native projects.

## IPC Bridge

When compiled with the `mobile` feature, the Rust backend starts a small HTTP
server on port `1421` to allow the mobile shell to communicate with the Tor
manager. The Capacitor app can perform requests to this server to control the
Tor connection.

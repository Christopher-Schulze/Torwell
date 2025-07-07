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
   - The Capacitor CLI then copies the assets and builds the native project.

3. To create the final application packages manually run:

   ```bash
   ./mobile/scripts/build_android.sh   # generates the APK
   ./mobile/scripts/build_ios.sh       # generates the IPA (macOS required)
   ```

### Reusing the frontend build

The build scripts check if the `build/` directory already exists. If present,
`bun run build` is skipped and the existing assets are reused. This shortens
subsequent mobile builds, for example:

```bash
bun run build              # create web assets once
./mobile/scripts/build_android.sh   # reuses build/
./mobile/scripts/build_ios.sh
```

## IPC Bridge

When compiled with the `mobile` feature, the Rust backend automatically launches
a small HTTP server listening on `http://127.0.0.1:1421`. The Capacitor shell
communicates with this server to control the Tor connection. Make sure requests
target this port when running the mobile app.

## CI Artifacts

A dedicated GitHub workflow (`mobile.yml`) builds the Android and iOS apps. The resulting packages are uploaded as artifacts on each run:

- `android-apk` contains the built `.apk` file.
- `ios-ipa` provides the `.ipa` bundle.

If no binary is produced, the workflow uploads a placeholder archive so the artifact list is always available. Download the desired artifact from the workflow run page to test the mobile build without setting up the full toolchain locally.

## CI Artifacts herunterladen

Die Artefakte kannst du direkt aus dem Mobile-Workflow herunterladen:

1. Rufe in GitHub den gewünschten Workflow-Lauf auf.
2. Scrolle zum Abschnitt **Artifacts**.
3. Lade das ZIP `android-apk` oder `ios-ipa` herunter.

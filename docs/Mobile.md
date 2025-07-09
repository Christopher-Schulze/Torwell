# Mobile Build

This directory contains the configuration for the Capacitor based mobile build.
The Svelte frontend is reused by pointing `webDir` to the compiled web assets.

## Build Steps

1. Run `task setup` once to install all dependencies.
2. Use the `Taskfile` targets to build the apps:

   ```bash
   task mobile:android  # Build Android APK
   task mobile:ios      # Build iOS project
   task mobile:release  # Build both and gather artifacts
   ```

   Each task performs the following:
   - `bun run build` generates the web assets in `build/`.
   - `cargo build --release --manifest-path src-tauri/Cargo.toml --features mobile` compiles the Rust backend so the HTTP bridge is included.
   - The Capacitor CLI then copies the assets and builds the native project.

3. To create the final application packages run `task mobile:release`. The command
   copies the resulting archives to `mobile/dist/`. You can still execute the
   scripts directly if desired:

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

## Quickstart für lokale Tests

Du möchtest die App selbst bauen? Mit dem `Taskfile` geht das recht schnell:

```bash
git clone https://github.com/Christopher-Schulze/Torwell.git
cd Torwell
task setup        # installiert alle Abhängigkeiten
task mobile:android  # oder `task mobile:ios`
task mobile:release  # erstellt beide Pakete
```

Die fertigen Dateien landen im Verzeichnis `mobile/dist`. Nach erfolgreichem
Lauf findest du das Android‑APK im Ordner `mobile/android/app/build/outputs/apk/`.
Für iOS wird ein Xcode-Projekt unter `mobile/ios` erzeugt.
Nach einem Build kannst du mit `./mobile/scripts/test_artifacts.sh` 
prüfen, ob die APK- bzw. IPA-Datei korrekt erstellt wurde.

## Testing the final builds

- **Android:** Starte einen Emulator in Android Studio und installiere das APK:

  ```bash
  adb install mobile/dist/*.apk
  ```

- **iOS:** Öffne `mobile/ios/App.xcworkspace` in Xcode und wähle einen
  Simulator aus. Du kannst das erzeugte `.ipa` aus `mobile/dist` auch über das
  Geräte-Fenster von Xcode auf ein verbundenes Gerät ziehen.

## Installation & Debugging

### Android

1. Stelle sicher, dass die Android-SDK-Plattform \(API 34\) installiert ist.
2. Installiere das APK auf einem Gerät oder Emulator und verfolge die Logs:

   ```bash
   adb install -r mobile/dist/*.apk
   adb logcat
   ```

### iOS

1. Öffne das Projekt mit Xcode:

   ```bash
   npx cap open ios
   ```

2. Wähle einen Simulator oder ein angeschlossenes Gerät und starte die App. Die
   Xcode-Konsole zeigt die Debug-Ausgabe an. Alternativ kannst du die `.ipa`
   mittels `xcrun simctl install booted mobile/dist/*.ipa` auf einen Simulator
   kopieren.

### Häufige Stolperfallen

- **Android SDK nicht gefunden:** Stelle sicher, dass `ANDROID_HOME` oder `ANDROID_SDK_ROOT` korrekt gesetzt ist und die Plattform API 34 installiert ist. Ohne diese Pfade schlagen die Build-Skripte sofort fehl.
- **Fehlende Java- oder Gradle-Version:** Das Android-Projekt benötigt eine funktionierende Java- und Gradle-Installation. Verwende die Versionen aus Android Studio oder setze `JAVA_HOME` passend.
- **iOS-Provisioning:** Xcode verlangt ein gültiges Entwicklerprofil. Prüfe, dass dein Apple-Account Provisioning-Profile und Zertifikate für das Projekt generiert hat, sonst lässt sich das IPA nicht auf Geräten testen.
- **Bun- und Node-Version:** Die Scripts setzen das `bun`-Tool voraus. Vergewissere dich, dass `bun install` erfolgreich gelaufen ist und eine aktuelle Node-Version verwendet wird.
- **Bridge-Port 1421 blockiert:** Der mobile Build kommuniziert über `http://127.0.0.1:1421`. Wenn dieser Port durch eine Firewall blockiert ist, lässt sich der Tor-Client nicht steuern.


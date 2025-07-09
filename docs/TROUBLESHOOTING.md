# Troubleshooting

This guide lists common problems encountered during development and how to analyse logs.

## Common Issues

- **Missing system libraries**: `cargo check` may fail if `glib-2.0` or other packages are not installed.
  To install the necessary headers on Ubuntu:

  1. `sudo apt-get update`
  2. `sudo apt-get install libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup2.4-dev pkg-config`
  3. Auf Systemen, die nur die `4.1` Varianten bereitstellen, fehlen die `*.pc`
     Dateien für `webkit2gtk-4.0` und `javascriptcoregtk-4.0`. Lege in diesem
     Fall symbolische Links an:

     ```bash
     sudo ln -s /usr/lib/x86_64-linux-gnu/pkgconfig/webkit2gtk-4.1.pc /usr/lib/x86_64-linux-gnu/pkgconfig/webkit2gtk-4.0.pc
     sudo ln -s /usr/lib/x86_64-linux-gnu/pkgconfig/javascriptcoregtk-4.1.pc /usr/lib/x86_64-linux-gnu/pkgconfig/javascriptcoregtk-4.0.pc
     ```

     Andernfalls meldet `cargo check` fehlende Pakete.

  Afterwards `cargo test` should run without missing-library errors.
- **Dependencies not installed**: If the frontend will not build, run `bun install` to fetch Node packages.
- **Build errors**: Ensure `bun run check` and `cargo check` succeed before opening a pull request.
  `cargo check` schlägt außerdem fehl, wenn die Umgebungsvariable
`TAURI_UPDATE_URL` nicht gesetzt ist. Für lokale Builds genügt ein Dummy-Wert,
z. B.:

  ```bash
export TAURI_UPDATE_URL="https://example.com"
```

- **Updater URL not substituted**: Wenn trotz gesetzter Variablen der Fehler
  `relative URL without a base: "${TAURI_UPDATE_URL}"` erscheint, wurde die
  Umgebungsvariable beim Kompilieren nicht korrekt übernommen. Stelle sicher,
  dass sie im selben Shell-Kontext exportiert wurde und verwende beim Export
  keine Anführungszeichen:

  ```bash
  export TAURI_UPDATE_URL=https://example.com
  cargo check --manifest-path src-tauri/Cargo.toml
  ```

## Node-Tools installieren

Das Projekt verwendet Bun als Paketmanager und SvelteKit für das Frontend. Führe einmalig

```bash
task setup    # oder: bun run setup
```

aus, um `bun install` und die Installation von `@sveltejs/kit` automatisch zu starten.

## Debugging & Log Analysis

- Start the app in development mode with `bun tauri dev` to view live output.
- The backend writes logs to a persistent file named `torwell.log` in the project directory. When the file exceeds the configured line limit it is rotated and the previous log is moved into an `archive` folder.
- Metrics are stored in `metrics.json`. If the file grows beyond the limits specified via `TORWELL_MAX_METRIC_LINES` or `TORWELL_MAX_METRIC_MB`, it is rotated as well and a warning is shown in the system tray.
- Each line of this file is a JSON object with `level`, `timestamp` and `message` fields.
- If the UI fails to load, open the browser developer tools (`Ctrl+Shift+I`) to inspect console logs and network activity.
- Failed connection attempts are recorded with `WARN` level. The retry counter resets when a new connection starts.
- If `Error::Timeout` occurs, the Tor bootstrap exceeded the allowed time. Check your network or increase the limit.
- The function `connect_with_backoff` enforces a maximum overall connection time and logs each retry.

## Rate Limits

- Connection attempts are limited to **5 per minute**. Exceeding this limit returns a `RateLimitExceeded` error.
- Retrieving logs via `get_logs` is limited to **20 requests per minute**.

## Zertifikatsupdate

Wenn das Herunterladen neuer Zertifikate fehlschlägt, erscheinen Meldungen wie
`certificate update failed` oder `failed to fetch new certificate` in
`torwell.log`. Prüfe, ob die in `cert_config.json` hinterlegte URL erreichbar
ist und dass `TORWELL_CERT_URL` korrekt gesetzt wurde. Der Befehl
`bun tauri dev` zeigt während des Starts ebenfalls eventuelle TLS-Fehler an.

## Schlüsselbundintegration

Auf macOS und Linux speichert Torwell84 sensible Daten im Betriebssystem-
Schlüsselbund. Tauchen dabei Fehler auf, liefert die Konsole Hinweise wie
`keyring access denied` oder `failed to unlock keychain`. Stelle sicher, dass
die Anwendung die erforderlichen Berechtigungen besitzt und teste die
Integration mit einem frischen Benutzerkonto, um Berechtigungsprobleme
auszuschließen.

## Stolperfallen

- Fehlende Systembibliotheken wie `glib-2.0` verhindern einen erfolgreichen `cargo check`.
- Vergessenes `bun install` führt zu nicht auflösbaren Frontend-Abhängigkeiten.
- Bei zu vielen Verbindungsversuchen oder Log-Abfragen tritt ein `RateLimitExceeded`-Fehler auf.
- Falsch konfigurierte Zertifikats-URLs melden `certificate update failed` im `torwell.log`.
- Scheitert die Schlüsselbundintegration, erscheint `keyring access denied` in der Konsole.

## Service Installation

Die Skripte `scripts/install_service.sh` und `scripts/test_service_install.sh`
helfen beim Einrichten des systemd-Dienstes. Die Testvariante verzichtet auf Root-Rechte
und simuliert `systemctl` Aufrufe. Eine erfolgreiche Ausführung zeigt z.B.:

```
Installing service file to /tmp/tmp.n82vj3ZQOZ
systemctl daemon-reload
Enabling and starting torwell84.service
systemctl enable --now torwell84.service
Service status:
systemctl status torwell84.service
● torwell84.service - Fake Service
   Loaded: loaded (/tmp/tmp.n82vj3ZQOZ/torwell84.service; enabled)
   Active: active (running)
Service file installed in /tmp/tmp.n82vj3ZQOZ
Test completed successfully
```

Wird `install_service.sh` direkt gestartet und `systemctl` durch ein Skript ersetzt,
sieht die Ausgabe beispielsweise so aus:

```
Installing service file to /tmp/tmp.1PBAKvEo7u
fake systemctl daemon-reload
Enabling and starting torwell84.service
fake systemctl enable --now torwell84.service
Service status:
fake systemctl --no-pager status torwell84.service
```


# Troubleshooting

This guide lists common problems encountered during development and how to analyse logs.

## Common Issues

- **Missing system libraries**: `cargo check` may fail if `glib-2.0` or other packages are not installed. Install the required development libraries or set `PKG_CONFIG_PATH` accordingly.
- **Dependencies not installed**: If the frontend will not build, run `bun install` to fetch Node packages.
- **Build errors**: Ensure `bun run check` and `cargo check` succeed before opening a pull request.

## Debugging & Log Analysis

- Start the app in development mode with `bun tauri dev` to view live output.
- The backend writes logs to a persistent file named `torwell.log` in the project directory. When the file exceeds the configured line limit it is rotated and the previous log is moved into an `archive` folder.
- Each line of this file is a JSON object with `level`, `timestamp` and `message` fields.
- If the UI fails to load, open the browser developer tools (`Ctrl+Shift+I`) to inspect console logs and network activity.
- Failed connection attempts are recorded with `WARN` level. The retry counter resets when a new connection starts.
- If `Error::Timeout` occurs, the Tor bootstrap exceeded the allowed time. Check your network or increase the limit.
- The function `connect_with_backoff` enforces a maximum overall connection time and logs each retry.

## Rate Limits

- Connection attempts are limited to **5 per minute**. Exceeding this limit returns a `RateLimited` error.
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
- Bei zu vielen Verbindungsversuchen oder Log-Abfragen tritt ein `RateLimited`-Fehler auf.
- Falsch konfigurierte Zertifikats-URLs melden `certificate update failed` im `torwell.log`.
- Scheitert die Schlüsselbundintegration, erscheint `keyring access denied` in der Konsole.


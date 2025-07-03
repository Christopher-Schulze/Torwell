# Certificate Management

Torwell84 uses certificate pinning to defend against man-in-the-middle attacks. The pinned certificates are stored in `src-tauri/certs`. A helper module (`secure_http.rs`) loads these certificates into a custom `RootCertStore` for `reqwest`.
All HTTPS connections enforce TLS&nbsp;1.2 or newer. `rustls` is configured to request OCSP stapling so revocation status is delivered with the server certificate when available.

## Hardened TLS Configuration

`SecureHttpClient` builds a `reqwest` client using `rustls`. OCSP stapling is
enabled so revocation information is provided with the certificate when the
server supports it. After each HTTPS request the client checks for the
`Strict-Transport-Security` header and logs a warning if it is missing.

## Rotation Procedure

1. The application fetches new certificates from a trusted endpoint using the existing pinned certificate.
2. Downloaded PEM files are saved to `src-tauri/certs/server.pem`.
3. `SecureHttpClient` reloads the file automatically so the new certificate is used without restarting the app.
4. If the download fails, the previous certificate remains in place and continues to be used.

This process ensures that the client always validates TLS connections against a known certificate while still allowing updates when required.

## Automatic Certificate Updates

`SecureHttpClient` checks for a new certificate each time the application
starts. The request uses the currently pinned certificate for validation.
Updated PEM files are saved to `src-tauri/certs/server.pem` and the HTTP client
reloads them immediately, so a restart is unnecessary. Periodic update checks
reuse the same mechanism.

## Configuration File

The certificate path and update URL are stored in a small JSON file. By default
`SecureHttpClient` reads `src-tauri/certs/cert_config.json`. When `init` is
called you may supply alternative values for the path and URL to override what
is specified inside the file.  This makes it possible to keep the configuration
file checked into version control while still testing different certificate
locations during development.

```json
{
  "cert_path": "src-tauri/certs/server.pem",
  "cert_url": "https://example.com/certs/server.pem"
}
```

`cert_path` is where the PEM file is written. `cert_url` specifies the HTTPS
endpoint used to retrieve updates.

When calling `SecureHttpClient::init` you can override these values without
modifying the file:

```rust
let client = SecureHttpClient::init(
    "src-tauri/certs/cert_config.json",
    Some("/tmp/dev.pem".into()),
    Some("https://localhost/dev_cert.pem".into()),
    None,
).await?;
```

### Update Workflow

1. On startup `SecureHttpClient::init` reads the configuration file and pins the
   certificate from `cert_path`. Optional parameters allow overriding these
   values without modifying the file.
2. The client downloads a new PEM from `cert_url` using the pinned certificate
   for validation.
3. The file at `cert_path` is replaced and the HTTP client reloads the
   certificate.
4. Periodic checks repeat the same process at the configured interval. The
   `schedule_updates` method spawns a background task that calls
   `update_certificates` on a timer.

## Konfiguration

Der Standardwert für `cert_url` verweist auf `https://example.com/certs/server.pem` und dient lediglich als Platzhalter.
Für produktive Einsätze muss dieser Wert auf den eigenen Update-Server zeigen.
Dazu öffnen Sie `src-tauri/certs/cert_config.json` und ersetzen die URL durch den gewünschten Endpunkt.
Alternativ können Sie beim Aufruf von `SecureHttpClient::init` einen abweichenden Wert übergeben, ohne die Datei zu verändern.
Ab Version 2.2.2 kann der Update-Endpunkt auch per Umgebungsvariable gesetzt werden.
Wird `TORWELL_CERT_URL` definiert, überschreibt dieser Wert die Einstellung aus
`cert_config.json`, sofern kein Parameter in `SecureHttpClient::init` gesetzt
wird.

## Geplante Zertifikatsrotation

Um eine durchgehende Vertrauenskette sicherzustellen, werden die
Serverzertifikate alle 90 Tage erneuert. `SecureHttpClient` ruft das
aktuelle PEM automatisch vom konfigurierten Endpunkt ab und ersetzt die
lokale Datei. So bleibt der Zertifikatspool stets aktuell, ohne dass ein
Neustart der Anwendung erforderlich ist.

### Rotation Workflow

1. Lege das neue Zertifikat auf dem Produktionsserver unter
   `https://certs.torwell.com/server.pem` ab.
2. Beim Start liest `SecureHttpClient` `cert_config.json` ein und
   verwendet `cert_url`, sofern keine Umgebungsvariable gesetzt ist.
   Wird `TORWELL_CERT_URL` definiert, hat dieser Wert Vorrang.
3. Der Client lädt das neue PEM herunter und ersetzt die Datei unter
   `cert_path`. Anschließend werden die Zertifikate im laufenden Prozess
   neu geladen.
4. Überprüfe die Logdatei auf Meldungen wie
   `certificate update failed` oder erfolgreiche Aktualisierungen, um
   sicherzustellen, dass der Wechsel stattgefunden hat.

# Certificate Management

Torwell84 uses certificate pinning to defend against man-in-the-middle attacks. The pinned certificates are stored in `src-tauri/certs`. A helper module (`secure_http.rs`) loads these certificates into a custom `RootCertStore` for `reqwest`.

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
`SecureHttpClient` looks for `src-tauri/certs/cert_config.json`.

```json
{
  "cert_path": "src-tauri/certs/server.pem",
  "cert_url": "https://example.com/certs/server.pem"
}
```

`cert_path` is where the PEM file is written. `cert_url` specifies the HTTPS
endpoint used to retrieve updates.

### Update Workflow

1. On startup `SecureHttpClient::init` reads the configuration file and pins the
   certificate from `cert_path`.
2. The client downloads a new PEM from `cert_url` using the pinned certificate
   for validation.
3. The file at `cert_path` is replaced and the HTTP client reloads the
   certificate.
4. Periodic checks repeat the same process at the configured interval.

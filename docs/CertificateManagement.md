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

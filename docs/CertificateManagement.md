# Certificate Management

Torwell84 uses certificate pinning to defend against man-in-the-middle attacks. The pinned certificates are stored in `src-tauri/certs`. A helper module (`secure_http.rs`) loads these certificates into a custom `RootCertStore` for `reqwest`.

## Rotation Procedure
1. The application fetches new certificates from a trusted endpoint using the existing pinned certificate.
2. The new certificate replaces `src-tauri/certs/server.pem` on success.
3. If the download fails, the previous certificate remains in place and continues to be used.

This process ensures that the client always validates TLS connections against a known certificate while still allowing updates when required.

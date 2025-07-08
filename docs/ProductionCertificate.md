# Production Certificate

This guide explains how to provide your own server certificate for Torwell84 deployments. The application pins a certificate and checks for updates at runtime. Example configuration files are located under `docs/examples`. Follow these steps to prepare a production setup.

## 1. Generate a Certificate

Create a PEM encoded certificate with your internal or public CA. For quick testing you can use OpenSSL:

```bash
openssl req -new -newkey rsa:4096 -days 90 -nodes -x509 \
    -keyout server.key -out server.pem \
    -subj "/CN=certs.torwell.com"
```

Place `server.pem` on your update server. Renew the file every 90 days.

## 2. Adjust `cert_config.json`

Use the example configuration in `docs/examples/cert_config.json` as a template. Copy it to `src-tauri/certs/cert_config.json` and set `cert_url` to the HTTPS endpoint where `server.pem` is hosted. The repository no longer ships a certificate file. Place your production PEM in `/etc/torwell/server.pem` or adjust `cert_path` accordingly. Windows and macOS paths can be configured via `cert_path_windows` and `cert_path_macos`.

```json
{
  "cert_path": "/etc/torwell/server.pem",
  "cert_url": "https://updates.example.com/certs/server.pem",
  "fallback_cert_url": null,
  "min_tls_version": "1.2"
}
```

## 3. Set Up Your Update Endpoint

Host `server.pem` on a web server reachable via HTTPS. The path must match the `cert_url` value from the configuration file, e.g. `https://updates.example.com/certs/server.pem`.  Ensure the file is replaced whenever a new certificate is issued.

A minimal Nginx setup might look like this:

```nginx
server {
    listen 443 ssl;
    server_name updates.example.com;

    ssl_certificate /etc/letsencrypt/live/updates.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/updates.example.com/privkey.pem;

    location /certs/ {
        alias /var/www/certs/;
        autoindex off;
    }
}
```

Copy `server.pem` into `/var/www/certs/` so it is served under `/certs/server.pem`.

Automate uploads with a cronjob that calls a small script after each renewal:

```bash
# /etc/cron.d/torwell-cert-renew
0 3 * * * root /usr/local/bin/push_cert.sh
```

`push_cert.sh` copies the fresh PEM to the update server:

```bash
#!/bin/bash
set -e
scp /pki/torwell/server.pem \
    user@updates.example.com:/var/www/certs/server.pem
```

Running this job ensures that clients can fetch the new certificate during the next update check.

## 4. Override via Environment Variables

Instead of editing the configuration file you can override the values at runtime:

```bash
export TORWELL_CERT_URL=https://updates.torwell.com/certs/server.pem
export TORWELL_CERT_PATH=/etc/torwell/server.pem
export TORWELL_FALLBACK_CERT_URL=https://backup.example.com/server.pem
```

`SecureHttpClient` prefers environment variables over `cert_config.json` when no parameters are passed to `init`.

## 5. Rotation Script Example

Automate certificate updates with a small script that copies the new PEM to the update server. Trigger it from a cronjob or CI pipeline.

```bash
#!/bin/bash
set -e
scp /pki/torwell/server.pem \
    user@certs.torwell.com:/var/www/certs/server.pem
```

Running this script after each renewal ensures that clients download the new certificate during the next update check.

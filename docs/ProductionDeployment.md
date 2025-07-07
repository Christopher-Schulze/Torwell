# Production Deployment

Torwell84 can run as a systemd service on Linux systems.
The repository provides a ready-to-use unit file at
`src-tauri/torwell84.service`.

## Installation

Copy the file to your systemd directory and enable the service:

```bash
sudo cp src-tauri/torwell84.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now torwell84.service
```

The service starts `/opt/torwell84/Torwell84` as the `torwell` user and group
and restarts automatically on failure. Logs are available with
`journalctl -u torwell84.service`.

## Service Installation

Instead of manually copying the unit file you can use the helper script:

```bash
./scripts/install_service.sh
```

It installs the service to `/etc/systemd/system/`, reloads systemd and enables
`torwell84.service` immediately.

## Certificate Configuration

Edit `src-tauri/certs/cert_config.json` to point to your production update server:

```json
{
  "cert_path": "src-tauri/certs/server.pem",
  "cert_url": "https://updates.example.com/certs/server.pem",
  "fallback_cert_url": null,
  "min_tls_version": "1.2",
  "update_interval": 86400,
  "note": "Production certificate update endpoint"
}
```

## Automatischer Update-Dienst

The application reads `update_interval` from the configuration file at startup.
If the value is greater than zero a background task periodically calls
`update_certificates_from` to refresh the pinned certificate. You can override
the interval at runtime using the `TORWELL_UPDATE_INTERVAL` environment
variable.

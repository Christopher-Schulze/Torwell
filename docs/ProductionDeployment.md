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

## Certificate Configuration

Edit `src-tauri/certs/cert_config.json` to point to your production update server:

```json
{
  "cert_path": "src-tauri/certs/server.pem",
  "cert_url": "https://updates.example.com/certs/server.pem",
  "fallback_cert_url": null,
  "min_tls_version": "1.2",
  "note": "Production certificate update endpoint"
}
```

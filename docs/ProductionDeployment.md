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

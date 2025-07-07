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

The service starts `/opt/torwell84/torwell84` as the `torwell` user and group
and restarts automatically on failure. Logs are available with
`journalctl -u torwell84.service`.

## Service Installation

Instead of running the commands manually you can use the helper script:

```bash
sudo ./scripts/install_service.sh
```

The script copies `src-tauri/torwell84.service` into `/etc/systemd/system/`,
reloads systemd and enables the service immediately.

## Certificate Configuration

Edit `src-tauri/certs/cert_config.json` to point to your production update server:

```json
{
  "cert_path": "/etc/torwell/server.pem",
  "cert_url": "https://updates.torwell.com/certs/server.pem",
  "fallback_cert_url": null,
  "min_tls_version": "1.2",
  "update_interval": 86400,
  "note": "Production certificate update endpoint"
}
```

Set the environment variables so `SecureHttpClient` can locate the
certificate and HSM library:

```bash
export TORWELL_CERT_PATH=/etc/torwell/server.pem
export TORWELL_HSM_LIB=/usr/local/lib/libyubihsm_pkcs11.so
```

## Automatischer Update-Dienst

The application reads `update_interval` from the configuration file at startup.
If the value is greater than zero a background task periodically calls
`update_certificates_from` to refresh the pinned certificate. You can override
the interval at runtime using the `TORWELL_UPDATE_INTERVAL` environment
variable.

## Updater Configuration

Set the URL for Tauri's auto-updater in `src-tauri/tauri.conf.json`. You may
hard-code the production endpoint or reference an environment variable:

```json
"endpoints": ["${TAURI_UPDATE_URL}"]
```

Export `TAURI_UPDATE_URL` before running `task release` to inject the desired
update server.

## Building Release Packages

Run the release task on the target platform to create the installer packages.

```bash
task release   # invokes scripts/build_release.sh
```

Depending on the operating system this produces:

- Windows: an `.msi` installer in `src-tauri/target/release/bundle/msi`
- Linux: `.deb` and `.AppImage` files under `src-tauri/target/release/bundle`
- macOS: a `.dmg` image in `src-tauri/target/release/bundle/dmg`

Copy the resulting package to the production machine and install it before
enabling the systemd service.

## Command Line Utility

The release also includes the `torwellctl` binary for headless
administration. It can import or export worker lists, trigger a manual
certificate update and print stored metrics.

```bash
torwellctl import-workers workers.txt
torwellctl export-workers
torwellctl update-cert
torwellctl show-metrics
```

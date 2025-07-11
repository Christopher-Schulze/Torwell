# Production Deployment

Torwell84 can run as a systemd service on Linux systems.
The repository provides a ready-to-use unit file at
`src-tauri/torwell84.service`. Example configuration files are available under
`docs/examples`.

## Installation

Copy the unit file to your systemd directory and enable the service:

```bash
sudo cp src-tauri/torwell84.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now torwell84.service
sudo systemctl status torwell84.service
```

The service starts `/opt/torwell84/torwell84` as the `torwell` user and group
and restarts automatically on failure. Logs are available with
`journalctl -u torwell84.service`.

**Root privileges are required** for installation because the script creates the `torwell` system user, copies the unit file to /etc/systemd/system and enables the service. If the application is installed in a different location adjust the `ExecStart`, `ExecStartPre` and `WorkingDirectory` paths in src-tauri/torwell84.service. Change the `User=` and `Group=` entries when using a different service account.

Ensure the binary under `/opt/torwell84/` exists and that the unit file's `ExecStart` and `WorkingDirectory` entries match the installation directory. When installing Torwell84 elsewhere update these directives accordingly and adapt `User=` and `Group=` to the owner of the files.

## Service Installation

Instead of running the commands manually you can use the helper script. It
validates the target directory and prints the service status after enabling it:

```bash
sudo ./scripts/install_service.sh
```

The script copies `src-tauri/torwell84.service` into `/etc/systemd/system/`,
reloads systemd and enables the service immediately. A lightweight test harness
is available to simulate the installation without touching real system files:

```bash
./scripts/test_service_install.sh
```
Running the script prints the commands that would be executed and a brief
status report:

```text
Installing Torwell84 systemd service...
Copying service file to /tmp/tmp.XYZ
Service file installed
systemctl daemon-reload
Enabling and starting torwell84.service
systemctl enable --now torwell84.service
Service status:
\u25cf torwell84.service - Fake Service
   Loaded: loaded (/tmp/tmp.XYZ/torwell84.service; enabled)
   Active: active (running)
Installation complete.
Service file installed in /tmp/tmp.XYZ
Test completed successfully
```

The GitHub Actions workflow executes this script on every pull request to
validate the service installation logic.

If the application resides in a custom directory or should run under a
different service account, edit `src-tauri/torwell84.service` before invoking
the installer. Adjust `ExecStart`, `ExecStartPre`, `WorkingDirectory`, `User`
and `Group` to match your setup. The helper script creates the `torwell` user
by default; when using another account create it manually first or modify the
`useradd` command in `scripts/install_service.sh`.

### Service-Test

The install script was executed on a test system. Starting the service failed
because the container lacked a running systemd instance, but the helper script
`test_service_install.sh` completed successfully and verified that the unit file
would be copied and enabled correctly on a real system.

### Log File

The backend writes persistent logs to `~/.local/share/torwell84/torwell.log` on
Linux systems (or `./torwell.log` when the XDG directories are unavailable).
Once the file grows beyond the limit defined by the `TORWELL_MAX_LOG_LINES`
environment variable (default `1000` lines), it is automatically rotated and the
previous log is moved into an `archive` subdirectory. Journald stores the
service output separately and handles rotation according to the system
configuration.

## Certificate Configuration

Copy `docs/examples/cert_config.json` to `src-tauri/certs/cert_config.json` and point it to your production update server:

```json
{
  "cert_path": "/etc/torwell/server.pem",
  "cert_url": "https://certs.torwell.com/server.pem",
  "fallback_cert_url": null,
  "min_tls_version": "1.2",
  "update_interval": 86400,
  "note": "Production certificate update endpoint"
}
```

Set the environment variables so `SecureHttpClient` can locate the
certificate and update endpoint:

```bash
export TORWELL_CERT_URL=https://certs.torwell.com/server.pem
export TORWELL_FALLBACK_CERT_URL=https://backup.torwell.com/server.pem
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

Before building on a fresh Linux machine execute the helper script to install
all required system libraries:

```bash
./scripts/setup_env.sh
```

On Windows install the Visual Studio Build Tools with the "Desktop development
with C++" workload. macOS users need the Xcode command-line tools. In either
case ensure `bun` and `cargo` are on your `PATH`.

The build script expects `TAURI_UPDATE_URL` to point to a valid update endpoint
or to a dummy URL when testing locally.

Depending on the operating system this produces:

- Windows: an `.msi` installer in `src-tauri/target/release/bundle/msi`
- Linux: `.deb` and `.AppImage` files under `src-tauri/target/release/bundle`
- macOS: a `.dmg` image in `src-tauri/target/release/bundle/dmg`

Copy the resulting package to the production machine and install it before
enabling the systemd service.

For official releases the GitHub workflow `release.yml` runs the same script on
Windows, macOS and Linux runners. The generated bundles are signed (when
secrets are available) and uploaded automatically to the GitHub Releases page.

## Obtaining Release Packages

The easiest way to install Torwell84 V2 on production machines is to reuse the
packages produced by CI. Navigate to the repository's **Actions** tab and open
the run for the `release` workflow matching the desired tag. Each job provides
an artifact named after the runner platform (for example `ubuntu-latest-bundle`
or `windows-latest-bundle`). Download the artifact for your operating system and
extract it locally. Inside you'll find the installer file along with a detached
signature ending in `.asc`.

Import the maintainer's public GPG key and verify the package before
installation:

```bash
gpg --verify torwell84_2.4.1_amd64.deb.asc torwell84_2.4.1_amd64.deb
```

On Windows you can additionally validate the MSI's code signing certificate:

```powershell
Get-AuthenticodeSignature Torwell84\torwell84.msi
```

After a successful verification copy the package to the target system.

## Signing Release Artifacts

The job defined in [`release.yml`](../.github/workflows/release.yml) imports a
GPG key and attaches detached signatures for every uploaded bundle. MSI files on
Windows are additionally signed with a code-signing certificate when the
secrets are configured. If you build packages manually, sign them using
`gpg --armor --detach-sign <file>` before distributing them.

## Verifying Signatures

Always verify the signatures on the production host before installation:

```bash
gpg --verify torwell84_2.4.1_amd64.deb.asc torwell84_2.4.1_amd64.deb
```

```powershell
Get-AuthenticodeSignature Torwell84\torwell84.msi
```

The commands report `Good signature` (and a valid certificate on Windows) if the
package has not been tampered with.

## Final Deployment Workflow

1. Build release packages with `task release` (ensure `TAURI_UPDATE_URL` is
   configured and dependencies are installed).
2. Sign the generated files using `gpg --armor --detach-sign` and apply a
   code-signing certificate on Windows.
3. Transfer the installer and signature to the production machine and verify
   them as shown above.
4. Install the package using the system installer (`dpkg -i`, double-click the
   MSI, etc.).
5. Enable the systemd service using `sudo ./scripts/install_service.sh`.
6. Check the status with `systemctl status torwell84.service` and view logs with `journalctl -u torwell84.service`.

## Tray Menu

When running with a system tray the application provides several actions:

- **Status** – shows whether Tor is currently connected.
- **Memory** and **Circuits** – display current resource usage.
- **Show** – opens the main window.
- **Connect** or **Disconnect** depending on the current state.
- **Reconnect** – attempts to reconnect when disconnected.
- **Show Dashboard** – opens the metrics dashboard.
- **Show Logs** – displays collected logs.
- **Open Log File** – reveals the log file on disk.
- **Settings** – opens the settings dialog.
- **Open Settings File** – opens the JSON configuration file.
- **Quit** – exits the application.

If a security warning occurs (for example high memory usage or repeated
certificate update failures) an additional disabled item is appended at the end
of the menu. On macOS this item uses the `NativeImage::Caution` icon.

## Custom Torrc Options

Advanced users can override Arti's defaults with a custom torrc snippet.
Open the settings dialog and edit the **Torrc Configuration** section. After
saving, the frontend sends the updated text to the backend using the
`set_torrc_config` command. The snippet is parsed as TOML and merged with the
generated configuration whenever a connection is established.

Example to disable IPv4 traffic:

```toml
[net]
ipv4 = false
```

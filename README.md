<div align="center">
  <img src="logo/image.png" alt="Torwell84 Logo" width="200" style="border-radius: 20px; box-shadow: 0 4px 8px rgba(0,0,0,0.1);">
  <h1>Torwell84</h1>
  <p>
    <strong>Privacy-First Tor Client</strong> · <em>Secure and Private Internet Access</em>
  </p>
  
  <!-- Badges -->
  <div style="margin: 1em 0;">
    <a href="https://github.com/Christopher-Schulze/Torwell84/releases">
      <img src="https://img.shields.io/badge/Status-Stable-brightgreen" alt="Status">
    </a>
    <a href="https://torproject.org">
      <img src="https://img.shields.io/badge/Tor-Enabled-7D4698?logo=tor" alt="Tor Network">
    </a>
    <a href="https://www.rust-lang.org/">
      <img src="https://img.shields.io/badge/Rust-1.77+-000000?logo=rust" alt="Rust">
    </a>
    <a href="https://tauri.app/">
      <img src="https://img.shields.io/badge/Tauri-1.6-FFC131?logo=tauri" alt="Tauri">
    </a>
    <a href="https://svelte.dev/">
      <img src="https://img.shields.io/badge/Svelte-4.0+-FF3E00?logo=svelte" alt="Svelte">
    </a>
  </div>

  <div style="margin: 1em 0;">
    <a href="https://privacy.community/">
      <img src="https://img.shields.io/badge/Privacy-First-2BB673" alt="Privacy First">
    </a>
    <a href="https://www.privacytools.io/">
      <img src="https://img.shields.io/badge/No%20Tracking-100%25-brightgreen" alt="No Tracking">
    </a>
    <a href="https://opensource.org/licenses/MIT">
      <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
    </a>
    <a href="https://github.com/Christopher-Schulze/Torwell84/actions">
      <img src="https://img.shields.io/github/actions/workflow/status/Christopher-Schulze/Torwell84/ci.yml?branch=main" alt="Build Status">
    </a>
  </div>
</div>

---

## 🔒 Secure and Private Internet Access

Torwell84 is a privacy-focused Tor client built with modern technologies to provide secure and private internet access. Built with Rust and Tauri, it offers a native desktop experience with strong privacy guarantees through the Tor network.

> 🚀 **Status**: Version 2.4.1 – actively developed with core Tor functionality implemented and working.

## ✨ Current Features

### 🛡️ Core Privacy Features
- **Tor Network Integration**: Secure and private internet access through the Tor network
- **Circuit Visualization**: Monitor your Tor circuit in real-time
- **No Logs Policy**: We don't track, store, or sell your browsing data
- **Bridge Support**: Configure custom Tor bridges from the settings modal
- **Isolated Circuits per Domain**: Multiple parallel circuits are maintained for the same domain
- **Circuit Management UI**: Build and inspect individual circuits
- **Guided torrc Generation**: Compose a hardened torrc fragment that reflects your pinned countries, fast-tier policy, and bridge configuration with a single click

### 🚀 Technical Highlights
- **Rust-Powered**: Built with Rust for performance and safety
- **Native UI**: Cross-platform desktop application using Tauri
- **Modern Stack**: Svelte-based frontend with TypeScript
- **Structured Logging**: JSON log entries with level and timestamp
- **Resource Monitoring**: Tray warnings for memory usage and circuit count; latency is measured for metrics
 - **Network Metrics**: `NetworkMonitor` and `NetworkTools` visualise CPU usage, traffic and traceroute results obtained from the backend.
   These widgets rely entirely on the metrics measured by the Rust backend.
- **Fast-tier Overrides**: Persist custom fast relay countries and surface them across the Rust backend and Svelte UI for deterministic route planning
- **HSM Support**: Optional PKCS#11 integration when built with the `hsm` feature
- **Mobile Workflow**: Capacitor-based build with HTTP bridge
- **Circuit Metrics**: Uses arti's experimental APIs when built with the
  `experimental-api` feature

### 📊 Status
- **Stable**: Core Tor functionality is working
- **Active Development**: Regular updates and improvements
- **Cross-Platform**: macOS, Linux and Windows supported

## 🛠️ Development Status

### Project Structure

The project has been refactored to a modern Tauri/Rust architecture, replacing the legacy Go backend.

```
Torwell84/
├── src/                        # SvelteKit frontend application
│   ├── lib/
│   │   ├── components/         # UI components
│   │   └── stores/             # Svelte stores for state management
│   └── routes/                 # App routes
├── src-tauri/                  # Rust backend (Tauri Core)
│   ├── src/
│   │   ├── commands.rs         # Tauri commands exposed to the frontend
│   │   ├── error.rs            # Custom error handling
│   │   ├── state.rs            # Shared application state
│   │   ├── tor_manager.rs      # Core Tor client logic
│   │   └── lib.rs              # Main library entry point
│   └── tauri.conf.json         # Tauri configuration
└── docs/                       # Project documentation
    ├── Changelog.md
    ├── DOCUMENTATION.md
    ├── CertificateManagement.md
    └── TODO123.md
```

### Development Setup

```bash
# Clone the repository
git clone https://github.com/Christopher-Schulze/Torwell.git
cd Torwell

# Install the Rust toolchain (provides `rustup` and `cargo`)
curl https://sh.rustup.rs -sSf | sh

# Install dependencies (using Bun as the package manager)
bun install

# Verify the frontend builds correctly
bun run check

# Start development server
bun tauri dev

# Build the application
bun tauri build
```

- Run backend tests:

```bash
cd src-tauri && cargo test
```

### UI Backup
Before experimenting with new designs you can back up the current UI
components by running:

```bash
scripts/backup_ui.sh
```
The script copies `src/lib/components` into a `src/lib/components_backup`
directory, creating it if necessary. This folder is not tracked in Git and can
be removed once you no longer need the backup.



### Updating Certificates
The pinned certificate location is configured in `src-tauri/certs/cert_config.json`.
By default this file uses the Torwell production endpoint `https://certs.torwell.com/server.pem`.
If you run your own update server, set the environment variables `TORWELL_CERT_URL`
or `TORWELL_CERT_PATH` to override the URL and local path at runtime. You can also
provide a backup with `TORWELL_FALLBACK_CERT_URL`.
The minimum TLS version can also be configured via the `min_tls_version` field
("1.2" or "1.3").

For a detailed description of the certificate rotation process, see
[docs/CertificateManagement.md#rotation-workflow](docs/CertificateManagement.md#rotation-workflow).
Instructions for creating your own production certificate are available in
[docs/ProductionCertificate.md](docs/ProductionCertificate.md).

Example for development:

```bash
TORWELL_CERT_URL=https://example.org/certs/server.pem \
TORWELL_CERT_PATH=src-tauri/certs/custom.pem bun tauri dev
```

### Runtime Configuration
You can influence certain backend parameters via environment variables:

- `TORWELL_CERT_URL` – HTTPS endpoint to download the pinned server certificate.
- `TORWELL_CERT_PATH` – Local path where the certificate is stored.
- `TORWELL_FALLBACK_CERT_URL` – Optional backup URL for certificate updates.
- `TORWELL_SESSION_TTL` – Lifetime of authentication tokens in seconds (default `3600`).
- `TORWELL_MAX_LOG_LINES` – Maximum number of log lines kept in `torwell.log` (default `1000`).
- `TORWELL_MAX_MEMORY_MB` – Memory usage threshold before warnings (default `1024`).
- `TORWELL_MAX_CIRCUITS` – Maximum allowed parallel circuits (default `20`).
- `TORWELL_HSM_LIB` – Path to the PKCS#11 module when compiled with the `hsm` feature.

> The first build will download many Rust crates and may take several minutes.

### Prerequisites
- Node.js 18+ and bun
- Rust and Cargo (via rustup)
- System dependencies for Tauri (see [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites))

### Systemvoraussetzungen
**Linux**
- `libglib2.0-dev`, `pkg-config` und GTK-Entwicklungsbibliotheken

**Frontend**
- [Svelte-CLI](https://github.com/sveltejs/cli) global installiert (`bun add -g svelte`)

Ohne diese Pakete schlagen `cargo test` und `bun run check` fehl. Ein Beispiel für einen fehlgeschlagenen Build findet sich in `/tmp/cargo_test.log`:

```text
error: failed to run custom build command for `glib-sys v0.15.10`
...
The system library `glib-2.0` required by crate `glib-sys` was not found.
```

### Windows Build
On Windows you also need the **Desktop development with C++** workload from the
Visual Studio Build Tools. After installing it, install Rust with the MSVC
toolchain and ensure Node.js and Bun are available.

To build locally:

```bash
bun install
bun run check
cd src-tauri && cargo test && cd ..
bun run tauri build
```

Bei jedem Push nach `main` erzeugt die GitHub Actions CI ebenfalls ein MSI.
Ist ein Codesigning-Zertifikat hinterlegt, wird der Installer signiert und als
Artefakt `windows-msi` im Workflow bereitgestellt.

## Deployment

Für automatisierte Builds steht die `Taskfile.yml` zur Verfügung. Der Befehl
`task build` ruft intern den Tauri Bundler über `bun tauri build` auf und setzt
standardmäßig das Cargo-Feature `experimental-api`. Dadurch werden
plattformabhängige Pakete mit erweiterten Circuit-Metriken erzeugt. Folgende
Umgebungsvariablen sind für den Release-Bau
relevant:

- `TORWELL_CERT_URL` – Serverpfad für das pinned Zertifikat
- `TORWELL_CERT_PATH` – Lokaler Speicherort der Zertifikatsdatei
- `TORWELL_FALLBACK_CERT_URL` – Optionale Ausweich-URL für Updates
- `TORWELL_SESSION_TTL` – Lebensdauer der Authentifizierungstokens

### Creating a Release

1. Versionsnummer in `package.json` und `src-tauri/Cargo.toml` aktualisieren.
2. `./scripts/update_changelog.sh` ausführen, um das Changelog zu aktualisieren.
3. Änderungen committen und einen Tag `vX.Y.Z` erstellen.
4. Tag und Branch pushen – der Release-Workflow baut und signiert die Pakete
   automatisch und lädt sie zu GitHub Releases hoch.
### Optional Cloudflare Worker

Im Ordner `cf worker` liegen zwei Beispielskripte für einen HTTPS-Proxy als Cloudflare Worker. Die Datei `Super-HTTPS-Proxy-CF-Worker-.txt` enthält eine Token-Prüfung, `Basic-HTTPS-Proxy-CF-Worker.txt` kommt ohne Authentifizierung aus. Diese Worker sind optional und für den Betrieb von Torwell84 nicht erforderlich.

**Deployment:**

1. `bun add -g wrangler`
2. `wrangler init` ausführen und den Beispielcode als `src/index.js` einbinden
3. `wrangler secret put SECRET_TOKEN` setzen
4. Mit `wrangler deploy` veröffentlichen

Eine ausführlichere Anleitung findet sich in
[docs/Todo-fuer-User.md](docs/Todo-fuer-User.md).

Der Worker erwartet die Zieladresse im Query-Parameter `url`. Torwell84 sendet
den Token im `X-Proxy-Token`‑Header und validiert ihn nach dem Speichern über
den internen Befehl `validate_worker_token`.


## Production Deployment

Stellen Sie für produktive Umgebungen einen erreichbaren Update-Endpunkt bereit,
damit die Zertifikate laut [CertificateManagement](docs/CertificateManagement.md)
regelmäßig erneuert werden können. Prüfen Sie `torwell.log` auf Meldungen wie
`certificate update failed`. Sobald das konfigurierbare Zeilenlimit erreicht
ist, rotiert die Anwendung die Datei und verschiebt ältere Logs in den Ordner
`archive`.

 Unter Linux empfiehlt sich der Betrieb als systemd‑Service. Die vollständige
 Datei findet sich unter `src-tauri/torwell84.service`. Eine minimale
 `torwell84.service`‑Datei könnte so aussehen:

```ini
[Unit]
Description=Torwell84 Service
After=network-online.target

[Service]
Type=simple
ExecStart=/opt/torwell84/torwell84
Restart=always
User=torwell
Group=torwell

[Install]
WantedBy=multi-user.target
```

Logs lassen sich anschließend mit `journalctl -u torwell84.service` abrufen.
Weitere Hinweise zur Einrichtung finden sich in
[docs/ProductionDeployment.md](docs/ProductionDeployment.md).

Zur schnellen Installation kopiert das Skript `scripts/install_service.sh` die
Unit-Datei nach `/etc/systemd/system/`, lädt systemd neu und startet den
Dienst:

```bash
sudo ./scripts/install_service.sh
```

## Installation

### Windows Installation
1. Lade den aktuellen `msi`‑Installer von der [Releases‑Seite](https://github.com/Christopher-Schulze/Torwell84/releases) herunter.
2. Doppelklicke die Datei `Torwell84_<version>.msi` und folge dem Installationsassistenten.
3. Bestätige den Herausgeber **Torwell84** und wähle das gewünschte Installationsverzeichnis.
4. Nach Abschluss findest du die Anwendung im Startmenü. Die Daten werden unter `%APPDATA%\Torwell84` gespeichert.
5. Sollte SmartScreen eine Warnung ausgeben, öffne die Eigenschaften der Datei, aktiviere "Zulassen" und starte den Installer erneut.

### Linux
For Debian-based distributions, install the `.deb` package:

```bash
sudo dpkg -i torwell84_<version>_amd64.deb
```

AppImage users can run the file directly:

```bash
chmod +x Torwell84-<version>.AppImage
./Torwell84-<version>.AppImage
```

### macOS
Open the `.dmg` file from the releases page and drag **Torwell84** to your Applications folder.

## 🛠️ Technical Details

### Built With
- **Backend**: Rust with arti (Tor implementation in Rust)
- **Frontend**: Svelte + TypeScript
- **Desktop**: Tauri 1.6+
- **Tor Version**: arti-client 0.31.0
- **UI Library**: Tailwind CSS with `tailwindcss-glassmorphism`

### Error States
The backend emits structured `Error` variants via the `tor-status-update` event. Common values are:

- `NotConnected` – command requires an active connection
- `AlreadyConnected` – connection attempt while already connected
- `ConnectionFailed` – connecting Tor failed with a `step` description
- `Identity` – changing circuits failed during a specific `step`
- `NetDir` – network directory lookup failed
- `Circuit` – circuit creation or inspection failed
- `RateLimitExceeded` – action exceeded its rate limit
- `Timeout` – operation aborted after the allowed time

## 📈 Roadmap

### Version 2.4.1
- Enhanced error handling and recovery
- Improved connection stability
- Better system tray integration
- Windows support
- Advanced circuit management
- Live resource monitoring with tray warnings
- Network monitoring tools

## 🤝 Contributing

See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines on code style, testing, and opening pull requests.

## 🔬 Test Coverage

An overview of existing backend tests and the CI workflow can be found in
[docs/ExistingTests.md](docs/ExistingTests.md).

## 🐞 Troubleshooting

Common issues and log analysis tips are documented in [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md).
The connection code already implements backoff with a maximum total time and logs each retry via `AppState.retry_counter`.
## ⚠ Known Limitations
See [docs/Limitations.md](docs/Limitations.md) for features that are currently impossible to implement, including per-circuit metrics.

## 📱 Mobile
Experimental Capacitor configuration is provided in [docs/Mobile.md](docs/Mobile.md). Use `task mobile:android` or `task mobile:ios` to build the mobile apps. The mobile build communicates with the Rust backend over a small HTTP bridge running on port 1421 when compiled with the `mobile` feature.
Guidance for using a Hardware Security Module is available in [docs/HSM.md](docs/HSM.md).

## 🔐 Security Findings
Aktuelle Erkenntnisse aus Audits sind im Dokument [docs/SecurityFindings.md](docs/SecurityFindings.md) zusammengefasst.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- The Tor Project for their incredible work on the Tor network
- The Arti team for their Rust implementation of Tor
- The Tauri team for the amazing desktop framework
- The open-source community for their invaluable contributions


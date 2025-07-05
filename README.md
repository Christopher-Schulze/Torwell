<div align="center">
  <img src="logo/image.png" alt="Torwell84 Logo" width="200" style="border-radius: 20px; box-shadow: 0 4px 8px rgba(0,0,0,0.1);">
  <h1>Torwell84</h1>
  <p>
    <strong>Privacy-First Tor Client</strong> · <em>Secure and Private Internet Access</em>
  </p>
  
  <!-- Badges -->
  <div style="margin: 1em 0;">
    <a href="https://github.com/Christopher-Schulze/Torwell84/releases">
      <img src="https://img.shields.io/badge/Status-Under%20Development-yellow" alt="Status">
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

> 🚀 **Status**: Actively developed with core Tor functionality implemented and working.

## ✨ Current Features

### 🛡️ Core Privacy Features
- **Tor Network Integration**: Secure and private internet access through the Tor network
- **Circuit Visualization**: Monitor your Tor circuit in real-time
- **No Logs Policy**: We don't track, store, or sell your browsing data
- **Bridge Support**: Configure custom Tor bridges from the settings modal
- **Isolated Circuits per Domain**: Multiple parallel circuits are maintained for the same domain

### 🚀 Technical Highlights
- **Rust-Powered**: Built with Rust for performance and safety
- **Native UI**: Cross-platform desktop application using Tauri
- **Modern Stack**: Svelte-based frontend with TypeScript
- **Structured Logging**: JSON log entries with level and timestamp

### 📊 Status
- **Stable**: Core Tor functionality is working
- **Active Development**: Regular updates and improvements
- **Cross-Platform**: macOS and Linux supported, Windows coming soon

## 🛠️ Development Status

### Current Focus
- Enhancing Tor connection stability
- Improving error handling and recovery
- Optimizing performance and resource usage

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
To back up the current Svelte components before experimenting with new designs,
run:

```bash
scripts/backup_ui.sh
```
The script copies `src/lib/components` to `src/lib/components_backup` so you can
easily restore the previous UI.



### Updating Certificates
The pinned certificate location is configured in `src-tauri/certs/cert_config.json`.
By default this file points `cert_url` to `https://internal.torwell.local/certs/server.pem` as a
placeholder. **Provide your own update endpoint for production.** Adjust the value
or set the environment variables `TORWELL_CERT_URL` or `TORWELL_CERT_PATH` to override the URL and local path at runtime.
The minimum TLS version can also be configured via the `min_tls_version` field
("1.2" or "1.3").

For a detailed description of the certificate rotation process, see
[docs/CertificateManagement.md#rotation-workflow](docs/CertificateManagement.md#rotation-workflow).

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

## Deployment

Für automatisierte Builds steht die `Taskfile.yml` zur Verfügung. Der Befehl
`task build` ruft intern den Tauri Bundler über `bun tauri build` auf und erzeugt
plattformabhängige Pakete. Folgende Umgebungsvariablen sind für den Release-Bau
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

## Production Deployment

Stellen Sie für produktive Umgebungen einen erreichbaren Update-Endpunkt bereit,
damit die Zertifikate laut [CertificateManagement](docs/CertificateManagement.md)
regelmäßig erneuert werden können. Prüfen Sie `torwell.log` auf Meldungen wie
`certificate update failed`. Sobald das konfigurierbare Zeilenlimit erreicht
ist, rotiert die Anwendung die Datei und verschiebt ältere Logs in den Ordner
`archive`.

Unter Linux empfiehlt sich der Betrieb als systemd‑Service. Eine minimale
`torwell84.service`‑Datei könnte so aussehen:

```ini
[Unit]
Description=Torwell84 Service
After=network-online.target

[Service]
Type=simple
ExecStart=/opt/torwell84/Torwell84
Restart=always
User=torwell
Group=torwell

[Install]
WantedBy=multi-user.target
```

Logs lassen sich anschließend mit `journalctl -u torwell84.service` abrufen.

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
The backend emits detailed error messages via the `tor-status-update` event. Possible values include `NotConnected`, `AlreadyConnected`, `Bootstrap`, `NetDir`, `Circuit`, and `Identity`.

## 📈 Roadmap

### In Progress
- [ ] Enhanced error handling and recovery
- [ ] Improved connection stability
- [ ] Better system tray integration

-### Upcoming Features
- [ ] Windows support
- [ ] Advanced circuit management
- [ ] Network monitoring tools

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

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- The Tor Project for their incredible work on the Tor network
- The Arti team for their Rust implementation of Tor
- The Tauri team for the amazing desktop framework
- The open-source community for their invaluable contributions


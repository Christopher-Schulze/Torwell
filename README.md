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
      <img src="https://img.shields.io/badge/Rust-1.70+-000000?logo=rust" alt="Rust">
    </a>
    <a href="https://tauri.app/">
      <img src="https://img.shields.io/badge/Tauri-1.5-FFC131?logo=tauri" alt="Tauri">
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
- **Cross-Platform**: Currently supports macOS, with Windows and Linux support planned

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

# Install dependencies
bun install  # Install Node.js dependencies

# Start development server
bun tauri dev

# Build the application
bun tauri build
```

### Prerequisites
- Node.js 18+ and bun
- Rust and Cargo (via rustup)
- System dependencies for Tauri (see [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites))

## 🛠️ Technical Details

### Built With
- **Backend**: Rust with arti (Tor implementation in Rust)
- **Frontend**: Svelte + TypeScript
- **Desktop**: Tauri 1.5+
- **Tor Version**: arti-client 0.31.0

### Error States
The backend emits detailed error messages via the `tor-status-update` event. Possible values include `NotConnected`, `AlreadyConnected`, `Bootstrap`, `NetDir`, `Circuit`, and `Identity`.

## 📈 Roadmap

### In Progress
- [ ] Enhanced error handling and recovery
- [ ] Improved connection stability
- [ ] Better system tray integration

### Upcoming Features
- [ ] Windows & Linux support
- [ ] Advanced circuit management
- [ ] Network monitoring tools

## 🤝 Contributing

See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines on code style, testing, and opening pull requests.

## 🐞 Troubleshooting

Common issues and log analysis tips are documented in [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md).
The connection code already implements backoff with a maximum total time and logs each retry via `AppState.retry_counter`.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- The Tor Project for their incredible work on the Tor network
- The Arti team for their Rust implementation of Tor
- The Tauri team for the amazing desktop framework
- The open-source community for their invaluable contributions

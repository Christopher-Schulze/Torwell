<div align="center">
  <img src="logo/image.png" alt="Torwell84 Logo" width="200" style="border-radius: 20px; box-shadow: 0 4px 8px rgba(0,0,0,0.1);">
  <h1>Torwell84</h1>
  <p>
    <strong>Next-Gen Privacy Suite</strong> · <em>Your Gateway to True Online Freedom</em>
  </p>
  
  <!-- Badges -->
  <div style="margin: 1em 0;">
    <a href="https://opensource.org/licenses/MIT">
      <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
    </a>
    <a href="https://github.com/Christopher-Schulze/Torwell84/releases">
      <img src="https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-blueviolet" alt="Platforms">
    </a>
    <a href="https://github.com/Christopher-Schulze/Torwell84/actions">
      <img src="https://img.shields.io/github/actions/workflow/status/Christopher-Schulze/Torwell84/ci.yml?branch=main" alt="Build Status">
    </a>
    <a href="https://torproject.org">
      <img src="https://img.shields.io/badge/Tor-Enabled-7D4698?logo=tor" alt="Tor Network">
    </a>
    <a href="https://gitlab.com/yawning/obfs4">
      <img src="https://img.shields.io/badge/OBFS4-Enabled-orange" alt="OBFS4">
    </a>
  </div>

  <div style="margin: 1em 0;">
    <a href="https://golang.org/">
      <img src="https://img.shields.io/badge/Go-1.21+-00ADD8?logo=go" alt="Go">
    </a>
    <a href="https://svelte.dev/">
      <img src="https://img.shields.io/badge/Svelte-4.0+-FF3E00?logo=svelte" alt="Svelte">
    </a>
    <a href="https://tauri.app/">
      <img src="https://img.shields.io/badge/Tauri-1.5-FFC131?logo=tauri" alt="Tauri">
    </a>
  </div>

  <!-- Navigation -->
  <p style="margin-top: 1.5em;">
    <a href="#features">Features</a> ·
    <a href="#getting-started">Getting Started</a> ·
    <a href="#development">Development</a> ·
    <a href="#roadmap">Roadmap</a>
  </p>
</div>

---

## 🔥 The Future of Private Browsing (Coming Soon)

Torwell84 is an upcoming privacy-focused application that aims to combine the power of Tor with military-grade obfuscation and cloud acceleration. Our goal is to create an all-in-one privacy suite that helps users bypass censorship and protect their online freedom.

> ⚠️ **Note**: Torwell84 is currently in active development. The macOS version is not yet ready for production use as we're still working on resolving some bugs and performance issues.

<div align="center">
  <img src="docs/TargetPicture.png" alt="Torwell84 in Action" width="80%">
</div>

## ✨ Planned Features

### 🛡️ Advanced Privacy Protection
- **Military-Grade Obfuscation**: Built-in OBFS4 bridges to defeat aggressive network censorship
- **Exit Node Selection**: Choose your virtual location from a global network of high-speed Tor nodes
- **Cloudflare Integration**: Optional Cloudflare Worker proxy for an additional layer of anonymity
- **No Logs Policy**: We don't track, store, or sell your data. Ever.

### ⚡ Performance Optimizations
- **Optimized Tor Network**: Custom-tuned Tor configuration for maximum speed
- **Smart Routing**: Automatically selects the fastest available nodes
- **Multipath Technology**: Distributes traffic for improved performance

### 🎮 User Experience
- **One-Click Connect**: Simple and intuitive interface
- **Real-Time Stats**: Monitor connection status and performance
- **Cross-Platform**: Native experience across all major platforms (in development)

## 🛠️ Development Status

### Current Focus
- Stabilizing the Rust backend and Tauri integration.
- Fixing critical bugs in the Tor connection logic.
- Ensuring the Svelte UI correctly communicates with the backend.

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
    └── TODO123.md
```

### Development Setup

```bash
# Clone the repository
git clone https://github.com/Christopher-Schulze/Torwell84.git
cd Torwell84

# Install dependencies
brew install go nodejs  # On macOS

# Setup development environment
task setup

# Start development servers
task dev
```

## 🛠️ Technical Details

### Built With
- **Backend**: Go 1.21+
- **Frontend**: Svelte + TypeScript
- **Desktop**: Tauri
- **Encryption**: AES-256, ChaCha20

## 📈 Roadmap

### Current Development
- [ ] Set up basic project tooling
- [ ] Implement missing backend packages
- [ ] Start integrating frontend with the backend

### Upcoming Features
- [ ] Windows & Linux support
- [ ] Cloudflare Worker integration
- [ ] Advanced routing options
- [ ] Mobile apps (iOS/Android)

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- The Tor Project for their incredible work on the Tor network
- The developers of OBFS4 for their obfuscation technology
- The open-source community for their invaluable contributions

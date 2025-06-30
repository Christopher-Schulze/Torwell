# Changelog

## [2.1.0] - 2025-06-29
### Added
- Implemented "New Identity" feature with Tauri backend support
- Added comprehensive logging system with in-memory storage
- Created task list for future improvements in TODO123.md

### Changed
- Improved error handling in Tor connection process
- Enhanced documentation for new features

## [2.1.1] - 2025-06-29
### Changed
- Removed Dark Mode references from TODO list
- Updated .gitignore to exclude build artifacts
- Deleted existing build artifacts to clean project

### Fixed
- None

## [2.2.0] - 2025-07-01
### Added
- Certificate pinning with `rustls` and hardened TLS configuration
- Documentation for certificate management

## [2.2.1] - 2025-07-03
### Added
- Bridge configuration through the `TorManager` with UI integration
- Parallel isolated circuits per domain via updated `get_isolated_circuit`
- Bridge selection list in the settings modal

## [2.0.0] - 2025-06-15
### Initial Release
- Rewritten architecture with Rust backend and Svelte frontend
- Tor integration via arti-client library
- Basic UI components for connection management

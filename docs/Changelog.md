## [2.3.1] - 2025-07-06
- Attempted to run cargo test, cargo clippy, bun run check and bun run lint:a11y; all failed due to missing toolchain components or commands.
- bun run tauri build could not run because Tauri CLI was not found.
- Service torwell84.service and journalctl logs unavailable in container.

## [2.3.0] - 2025-07-06
- Add circuit management commands and UI
- Add HSM initialization example and tests
- Add network diagnostics
- docs: add production certificate guide
- Update mobile build scripts and docs
- Improve error context for Tor connection
- Add platform-specific security notifications and badge
- Add ResourceDashboard component and route
- feat: add hsm feature flag
- Add Capacitor mobile skeleton and HTTP bridge
- feat(state): notify via system notification
- Add detailed error variants and update tor manager
- Add worker token settings and proxy support
- Add security tests and CI static analysis
- Improve error diagnostics with new variants
- Add detailed error logging and stack support
- docs: update status and deployment info
- feat: add bundle analysis plugin and code splitting
- Improve a11y labels and tests
- Add bridge preset support
- docs: add Cloudflare worker guide
- Log certificate update failures and warn on consecutive errors
- Remove npm lockfile and update docs for Bun
- Refactor error handling in TorManager
- Add external GeoIP database support
- chore: add a11y lint
- docs: link security findings
- Add log rotation test
- Add rate limit test for ping_host
- Handle isolation build errors and extend tests
- CI: add security audit steps
- docs: update cert url
- Add SettingsModal persistence tests
- Add systemd service file and documentation
- docs: describe session token usage
- docs: add Update-Server section
- Simplify Cloudflare worker examples
- Add circuit metrics tracking
- Add production deployment docs
- ci: build and sign windows msi
- Expand fuzz command test
- feat(tray): show status and warnings
- Add detailed error context
- Add certificate rotation test
- Add metrics chart and store buffering
- Add network error variant and update conversions
- Add tray menu actions and handlers
- docs: expand Windows installation
- Add tests for torStore and bridge saving
- docs: add UI design chapter
- ci: run svelte-check
- docs: add PenTest plan
- Add UI backup script and docs
- Remove obsolete NextSteps roadmap and update references
- feat(state): tray warnings for limits
- Enhance TLS and HSTS handling
- load bridge presets from json
- chore: sign release artifacts
- Add session cleanup and token checks
- Add fuzzing script for Tauri commands
- ci: add clippy step and document testing
- feat: add error boundary and focus trap
- Add automatic circuit shutdown on limits
- Update certificate URL to production
- docs: detail ci pipeline and cert rotation
- docs: update security audit checklist and findings
- Add logging hook and metrics annotations
- docs: add detailed rotation workflow
- Add accessibility attributes and modal focus handling
- Improve error context for TorManager
- Improve error context for TorManager
- Add TorManager metrics tests
- Fix apt step placement
- docs: add system requirements
- Update release checklist and certificate URL
- chore: add missing newline to layout
- docs: update ping section
- docs: add deployment details and release checklist
- Store AES key in OS keychain
- Harden TLS client and document cert rotation
- feat: emit security warnings
- Add detailed error handling
- feat: emit security warnings
- docs: list runtime env vars
- docs: add Windows build info
- docs: update todo to reflect geoip implementation
- docs: update storage encryption section
- ci: add macOS build
- Use surge-ping library for ICMP
- docs: document default cert url and env overrides
- Add TorChain tests and fix database spec for fake IndexedDB
- Add fallback certificate update mechanism
- Add fallback certificate update mechanism
- Implement security and UI improvements
- fix: address type checks and docs
- feat: encrypt sensitive settings
- Validate ping_host inputs
- Add session tokens and secure APIs
- Add TLS version config and HSTS warnings
- Add TLS version config and HSTS warnings
- Update cert url and document rotation
- chore: add cross-platform bundles
- Add tray menu and updater settings
- feat: add log rotation and metrics emitter
- Add security audit plan and findings
- docs: add architecture diagrams
- Move isolated circuit view to bottom
- Add unit and e2e tests
- Add session management and API rate limiting
- Implement TLS hardening and document cert rotation
- chore: enable cross-platform build
- Add exit country support in build_config
- feat: add exit country settings and log limit test
- docs: explain cert_url override
- Use ProjectDirs for log path
- Add note about cert_url configuration
- docs: clarify certificate pinning
- docs: standardize on bun
- Add basic ActionCard tests and update test script
- Add ARIA improvements
- docs: clarify certificate pinning
- Update cert_url to GitHub raw link
- Use bun for frontend
- Add basic component tests with vitest
- Add basic ARIA attributes
- docs: update version references
- chore: sync version to 2.2.2
- Add CI workflow
- docs: expand development setup
- Bump backend version
- Remove duplicate log lines setting and format file
- Add configurable log limit
- feat: add rate limiting
- feat: add rate limiting
- Add configurable log limit
- Improve accessibility
- Improve HSTS checks and document TLS
- Enable OCSP stapling and HSTS checks
- feat: improve accessibility
- Implement real ping using Tauri command
- Use Lazy GeoipDb and update tests
- Add periodic cleanup for isolation tokens
- docs: add config section for cert_url
- feat: display tor metrics
- docs: add overview of existing tests
- docs: note circuit metrics limitation
- docs: note existing retry and timeout features
- Add additional command tests
- Add test for init using default cert config
- Add metrics reporting
- Add tests and accessor methods
- Add retry logging and timeout handling
- Add secure HTTP client init and config
- Use Tor GeoIP DB for country lookup
- Use Tor GeoIP DB for country lookup
- docs: mention UI log filter
- test(cert): override config params
- docs: detail certificate config
- docs: mention structured JSON logs
- docs: note bridge and isolation features
- feat(cert): parameterize init
- docs: explain bootstrap progress
- feat(tor): emit bootstrap progress messages
- feat(tor): emit bootstrap progress messages
- Add Dexie config functions and settings UI
- Add configurable certificate path and update tests
- Refine LogsModal with level filter
- feat: add bridge support and improve isolation
- Add bootstrap progress callback
- test: add command tests
- track retry progress
- Expose traffic stats
- feat(cert): auto update on init
- feat: add exit country selector
- Expose retry status and backoff
- feat: add certificate rotation support
- remove old Torwell84 directory and clean gitignore
- Add mockable TorManager and tests
- feat(logs): add rotation config and ui controls
- Add GeoIP lookup for relay info
- Add backoff, isolation and exit policy support
- feat: persist logs to file
- Add rustls certificate pinning
- Expand error handling and docs
- Update CI for Rust and Svelte
- chore: adopt bun for frontend
- docs: add contributing and troubleshooting guides
- Update README: Remove navigation links and large image
- Initial commit: Core Tor management functionality with Rust/Tauri/Svelte
- docs: Add MIT License file
- Initial commit: Project setup with logo and documentation
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

## [2.3.0] - 2025-07-10
### Added
- Windows support is now available for builds and installation
- New resource monitoring with tray warnings for high memory usage, circuit count and latency
### Changed
- Status badge in README switched to Beta

## [2.2.2] - 2025-07-05
### Added
- Documented that connection retries and timeouts are already implemented.

## [2.0.0] - 2025-06-15
### Initial Release
- Rewritten architecture with Rust backend and Svelte frontend
- Tor integration via arti-client library
- Basic UI components for connection management

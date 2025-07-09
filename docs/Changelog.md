# Changelog

## [2.4.1] - 2025-07-08
- Fixed TypeScript errors and updated documentation
- Added support for custom `torrc` configuration
- Improved worker import and metrics handling

## [2.4.0] - 2025-07-08
- Worker queue now uses `VecDeque` with configurable metrics interval
- Enhanced service installation and mobile workflow
- Updated certificate configuration and tray menu refresh

## [2.3.1] - 2025-07-07
- Persisted metrics and added GeoIP directory option
- Worker import with token validation
- New circuit build time charts and diagnostics

## [2.3.0] - 2025-07-06
- Mobile build with HTTP bridge support
- Circuit management UI and HSM integration
- Added network diagnostics and resource dashboard

## [2.2.2] - 2025-07-05
- Documented connection retries and timeouts

## [2.2.1] - 2025-07-03
- Bridge configuration and isolated circuits per domain

## [2.2.0] - 2025-07-01
- Certificate pinning with `rustls` and hardened TLS

## [2.1.1] - 2025-06-29
- Updated `.gitignore` and removed Dark Mode references

## [2.1.0] - 2025-06-29
- Added "New Identity" feature and comprehensive logging

## [2.0.0] - 2025-06-15
- Initial release with Rust/Tauri architecture and basic Tor management

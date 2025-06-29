# Changelog - Torwell84 V2

## [2025-06-29] - Refactoring & Hardening
## [2025-06-29] - Critical Bug Fixes & UI Alignment

### Fixed
- **Compilation Error:** Corrected a critical bug in `tor_manager.rs` by changing `circuit.path()?.path().iter()` to the correct `circuit.path().unwrap().hops().iter()`.
- **UI Logic Error:** The "New Circuit" button in `ActionCard.svelte` now correctly calls the `new_identity` command instead of the wrong `get_active_circuit` command.

### Added
- **Log Commands:** Implemented placeholder commands `get_logs` and `clear_logs` in the Rust backend (`commands.rs` and `lib.rs`) to prevent UI crashes.

### Changed
- **README Update:** Replaced the outdated "Project Structure" section in `README.md` with an accurate description of the current Tauri/Rust architecture.

### Added
- Implemented a robust, fully-featured `TorManager` in `tor_manager.rs` to handle all Tor-related logic.
- Implemented previously missing backend functions: `get_active_circuit` and `new_identity`.
- Added specific, descriptive error types (`NotConnected`, `AlreadyConnected`, `NoCircuit`) to `error.rs` for robust error handling.
- Added periodic fetching of the Tor circuit to the Svelte UI for real-time updates.

### Changed
- **Refactored Backend Architecture:** Moved all business logic from `commands.rs` into `tor_manager.rs` to ensure a clean separation of concerns. `commands.rs` now only contains thin wrappers.
- **Hardened Backend:** Replaced all `.unwrap()` calls with proper error handling and logging.
- **Refactored Frontend Logic:** Removed mock data and placeholders from the UI. The inefficient reactive circuit-fetching logic was replaced with a robust `setInterval` that only runs when connected.
- **Updated Documentation:** Both `DOCUMENTATION.md` and `Changelog.md` have been updated to reflect the current, stable, and complete state of the application.

### Removed
- All `TODO` comments, stubs, and placeholder code have been removed from the entire codebase.
- Redundant connection logic was removed from `commands.rs`.
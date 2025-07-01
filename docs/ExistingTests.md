# Existing Tests and CI

This project already includes unit and integration tests for key functionality.
The most relevant files are:

- `src-tauri/tests/commands_tests.rs` – covers command handlers like
  `set_exit_country`, `set_bridges` and `clear_bridges`.
- `src-tauri/src/tor_manager.rs` – contains tests for the GeoIP cache logic.
- `src-tauri/tests/secure_http_tests.rs` – verifies certificate initialization
  and periodic updates.

Continuous integration runs these tests automatically. The workflow defined in
`.github/workflows/ci.yml` executes `cargo test` for the backend and
performs Svelte type checking.

No additional test implementation is required for these features. When running
`cargo test` locally, ensure that system libraries such as `glib-2.0` are
installed, otherwise the build will fail.

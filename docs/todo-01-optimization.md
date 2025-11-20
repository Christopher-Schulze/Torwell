# Optimization: Arti & Obfs4

## Goal
Optimize the Arti client for maximum reliability and obfuscation using `obfs4`.

## Tasks
1.  **Bridge Configuration**:
    -   Allow `TorManager` to accept bridge lines (obfs4).
    -   Configure `TorClientConfig` to use these bridges.
2.  **Circuit Pre-warming**:
    -   Ensure 3+ circuits are built proactively.
3.  **Caching & Performance**:
    -   Tune timeouts.
    -   Ensure directory caching is persistent and robust.

## Implementation Details
-   Modify `src-tauri/src/tor_manager.rs`.
-   Use `TorClientConfigBuilder`.
-   Ensure `Bridges` map correctly to Arti's config.

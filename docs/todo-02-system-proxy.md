# System Proxy Routing

## Goal
Route system traffic through the Tor SOCKS5 proxy (default port 9150) upon connection.

## Tasks
1.  **Proxy Manager Module**:
    -   Create `src-tauri/src/system_proxy.rs`.
    -   Implement `enable_global_proxy(port: u16)` and `disable_global_proxy()`.
    -   Support Linux (gsettings/env) and potentially Windows/macOS logic (via registry/networksetup).
2.  **Integration**:
    -   Call `enable` on `TorManager::connect`.
    -   Call `disable` on `TorManager::disconnect` and app exit.

## Implementation Details
-   On Linux (GNOME/Ubuntu): Use `gsettings set org.gnome.system.proxy ...`.
-   This fulfills the user's "VPN-like" requirement by routing app traffic.

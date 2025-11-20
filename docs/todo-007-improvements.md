# Technical Detail: Future Improvements (Todo-007)

## 1. Arti Persistence Isolation
Aktuell nutzt Arti Standard-XDG Pfade.
**Ziel:** `TorClientConfig` so konfigurieren, dass `storage.cache_dir` und `storage.state_dir` im Tauri App-Data Verzeichnis liegen (`app_handle.path_resolver().app_data_dir()`).
**Vorteil:** Saubere Deinstallation, Sandbox-Konformität.

## 2. Pluggable Transports (PT)
Aktuell wird nur direktes Tor genutzt.
**Ziel:** `arti-client` mit `pt-transport` Feature bauen und `BridgeConfig` ermöglichen (OBFS4, Snowflake).
**Status:** `Cargo.toml` enthält bereits `experimental-api`, aber PT Features müssen explizit aktiviert werden.

## 3. Circuit Visualization
Das Frontend zeigt eine Liste von Relays.
**Ziel:** GeoIP-Daten nutzen, um eine Weltkarte zu zeichnen (dazu wird `tor-geoip` bereits genutzt).

## 4. I18n
Lokalisierung für Deutsch/Englisch im Frontend.

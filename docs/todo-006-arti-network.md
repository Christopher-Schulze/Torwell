# Technical Detail: Arti Integration & Network Stack (Todo-006)

## Fragen
1. **Traffic Routing:** Nutzt `SecureHttpClient` (`secure_http.rs`) tatsächlich Arti/Tor oder geht er am Proxy vorbei?
2. **Robustheit:** Wie verhält sich Arti bei Verbindungsabbrüchen?
3. **Isolation:** Werden Circuits korrekt isoliert (per Domain)?

## Analyse-Ergebnisse
- **CRITICAL:** `SecureHttpClient` nutzt `reqwest` ohne Proxy-Konfiguration. Der Traffic geht **direkt** ins Internet, am Tor-Netzwerk vorbei.
- `TorManager` startet keinen SOCKS-Listener.
- `TorClient` wird nur für interne Checks (Prewarm, Ping) genutzt, nicht für den App-Traffic via `reqwest`.

## Lösungskonzept
1. **SOCKS Listener:** `TorManager` muss einen lokalen SOCKS5 Listener starten (z.B. Port 9150 oder ephemeral).
2. **Proxy Config:** `SecureHttpClient` muss dynamisch auf diesen Proxy konfiguriert werden, sobald Tor verbunden ist.
3. **Rebuild:** Da `reqwest::Client` immutable ist, muss `SecureHttpClient` den Client neu bauen, wenn der Proxy verfügbar ist.

## Maßnahmen
1. `tor_manager.rs`: `launch_socks_listener` hinzufügen.
2. `secure_http.rs`: `set_proxy` Methode hinzufügen, die den Client neu baut.
3. `lib.rs` / `state.rs`: Verbindung zwischen `TorManager` (Proxy Port) und `SecureHttpClient` herstellen.

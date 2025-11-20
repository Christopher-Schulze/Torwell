# Technical Detail: Dependency Management & Arti Update (Todo-002)

## Problembeschreibung
Die `Cargo.toml` verwendet Wildcards (`*`) für `arti-client` und `tor-*` Abhängigkeiten. Dies führt zu instabilen Builds, da ungetestete Major-Updates automatisch gezogen werden. Zudem forderte der Nutzer explizit die "allerneueste" Version von Arti.

## Aktueller Zustand (`Cargo.toml`)
```toml
arti-client = { version = "*", features = ["tokio", "rpc", "full", "experimental-api", "geoip"] }
tor-rtcompat = { version = "*" }
tor-circmgr = "*"
# ... weitere *
```

## Zielzustand
Alle `tor-*` und `arti-*` Pakete sollen auf eine feste, aktuelle Version gepinnt werden (Semantic Versioning).

## Recherche (Latest Versions)
*Stand: Heute*
- `arti-client`: v0.23.0 (oder neuer, zu prüfen via crates.io)
- `tor-rtcompat`: passend zu arti
- `tor-geoip`: separat zu prüfen.

## Maßnahmen
1. **Versionen ermitteln:** `cargo search arti-client` ausführen.
2. **Update `Cargo.toml`:**
   - `arti-client` -> `x.y.z`
   - `tor-rtcompat` -> `x.y.z`
   - Alle anderen `*` durch konkrete Versionen ersetzen.
3. **Rust Version:**
   - `rust-version` in `Cargo.toml` prüfen. Arti benötigt oft sehr neues Rust. Ggf. auf `1.77` oder höher setzen.

## Risiken
- Breaking Changes in der Arti API (sehr wahrscheinlich bei Sprung von `*` auf Latest, falls der alte Code alt war).
- `experimental-api` Features könnten sich geändert haben.

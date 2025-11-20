# Technical Detail: Backend Build Errors (Todo-001)

## Problembeschreibung
Das Rust-Backend (`src-tauri`) kompiliert derzeit nicht. Es gibt über 100 Fehler/Warnungen, die den Build verhindern.

## Fehleranalyse

### 1. Tauri Context Generation (`lib.rs`)
```rust
.run(tauri::generate_context!())
```
**Fehler:** `The 'distDir' configuration is set to "../build" but this path doesn't exist`
**Ursache:** Das Verzeichnis `../build` (relative zu `src-tauri`) existiert noch nicht, da der Frontend-Build noch nicht lief. Tauri prüft dies zur Compile-Zeit.
**Lösung:**
- Sicherstellen, dass das Verzeichnis existiert (via `mkdir -p ../build` vor dem Cargo-Build).
- Oder temporär ein Dummy-File dort anlegen.

### 2. Missing Crate `tor_geoip` (`tor_manager.rs`)
```rust
use tor_geoip::{CountryCode, GeoipDb};
```
**Fehler:** `use of unresolved module or unlinked crate tor_geoip`
**Ursache:** `tor_geoip` ist in `Cargo.toml` nicht als Abhängigkeit gelistet. Es könnte eine interne Crate von `arti` sein, die nun anders heißt oder separat hinzugefügt werden muss.
**Lösung:**
- Prüfen, ob `arti-client` das exportiert.
- Alternativ: Das Paket `tor-geoip` zu `Cargo.toml` hinzufügen (falls öffentlich verfügbar).

### 3. Veraltete WGPU API (`renderer/worker.rs`)
```rust
store: wgpu::StoreOp::Store,
```
**Fehler:** `could not find StoreOp in wgpu`
**Ursache:** In neueren `wgpu` Versionen (>= 0.17/0.18) wurde `StoreOp` vereinfacht oder geändert.
**Lösung:**
- Dokumentation von `wgpu` prüfen und Syntax anpassen. Vermutlich reicht `store: true` oder ähnlich (abhängig von der Version).

### 4. Borrow Checker Error (`renderer/worker.rs`)
```rust
for slot in &mut self.slots {
    if let Some(pending) = slot.pending.take() {
        match self.finalize_pending(pending) { ... }
    }
}
```
**Fehler:** `cannot borrow *self as mutable more than once at a time`
**Ursache:** `self.slots` ist Teil von `self`. Während wir über `self.slots` iterieren (mutable borrow), rufen wir `self.finalize_pending` auf (zweiter mutable borrow von `self`).
**Lösung:**
- Die `pending` Items erst sammeln (z.B. Indizes oder die Items selbst via `take`) und dann *nach* dem Loop verarbeiten.

## Implementierungsplan
1. `mkdir -p src/build` (bzw. root `build`).
2. `tor_geoip` in `Cargo.toml` aufnehmen oder Import korrigieren.
3. `wgpu` Code migrieren.
4. `renderer/worker.rs` Refactoring.

# Torwell84 V2 - Documentation

## 0. Aktuelle Highlights (v2.5)

- Premiumisierte Glass-UI mit abgestimmten Farbverläufen, Mikroanimationen und responsiven Layouts für Dashboard, Status-Card und Kontrollpanel.
- Resiliente Verbindungssteuerung: `invoke`-Wrapper mit Exponential-Backoff, entkoppelte Event-Lifecycle-Verwaltung in `torStore`.
- Arti-spezifische Guardrails mit zusätzlichen Tests für GeoIP- und Routing-Policies.
- Dokumentations-Hub erweitert um `spec.md`, `plan.md`, `FILEANDWIREMAP.md`, `ReleaseNotes.md` und `docs/todo` gemäß Organisationsleitfaden.
- Frontend-Aktionswarteschlange `connectionQueue` serialisiert Connect/Disconnect/Circuit-Befehle, visualisiert Queue-Tiefe und merkt sich Fehler.
- `TorManager::connect` verhält sich idempotent – wiederholte Aufrufe liefern Erfolg statt `AlreadyConnected` und vermeiden überflüssige Bootstrap-Versuche.
- Adaptive Cache-Schicht (`src/cache`) mit LRU/LFU-Eviction, Warmup aus Persistenz und API-Hooks für Timeline-, Summary- und Geo-Lookups.
- Struct-of-Arrays basierte Metrikpipeline (`metricSeries`) verkürzt Trendberechnungen und reduziert GC-Last im Hot-Path.
- Hot-Path-Allocator `mimalloc` für alle Rust-Komponenten, inklusive Memory-Profiling-Playbooks (`run_massif.sh`, `run_heaptrack.sh`).

## 1. Architecture

Torwell84 V2 is a complete rewrite focusing on a clean, modern, and maintainable architecture. The legacy Go backend and all associated code have been entirely discarded.

The new architecture is unified and consists of the following components:

-   **Frontend:** A SvelteKit single-page application responsible for the entire user interface. It is located in the `/` directory of the V2 project.
-   **Backend (Core):** A single, modular Rust crate located in `/src-tauri`. This crate is directly integrated with the Tauri runtime and serves as the application's core logic. There is no separate backend process.
-   **Communication:** The frontend and backend communicate exclusively and efficiently through Tauri's built-in IPC (Inter-Process Communication) system, using Tauri commands (`invoke`) and events (`listen`). This eliminates the need for any local web server, gRPC, or other network protocols between the frontend and backend.

## 2. Core Components

### 2.1. Rust Backend (`/src-tauri`)

The backend is structured into several logical modules:

-   **`main.rs` & `lib.rs`:** The entry points for the Tauri application, responsible for initializing the builder, managing the application state, and registering commands.
-   **`state.rs`:** Defines the shared `AppState`, which holds the `TorManager` instance and log storage, ensuring a single, consistent state for the Tor client across the application.
-   **`tor_manager.rs`:** A dedicated, robust module that encapsulates all interactions with the `arti-client` library. It handles the lifecycle of the Tor client, including connection, disconnection, circuit management (`get_active_circuit`), and requesting a new identity (`new_identity`).
-   **`commands.rs`:** Implements all Tauri commands that are exposed to the Svelte frontend. These functions act as thin, clean wrappers, delegating all business logic to the `TorManager`. This ensures a clear separation of concerns.
-   **`error.rs`:** Defines a custom, serializable `Error` enum for the entire backend. In addition to `NotConnected` and `AlreadyConnected`, it exposes variants for bootstrap failures, directory lookups, circuit building issues and identity refresh problems. These descriptive errors are serialized and sent to the frontend for user-friendly reporting.

### 2.2. Svelte Frontend (`/src`)

Der Desktop-Client nutzt eine kuratierte Glassmorphism-Ästhetik mit GPU-beschleunigten Blur-Layern, dynamischen Gradienten und `prefers-reduced-motion`-Aware Animationen.

-   **State Management:**
    -   `torStore.ts`: Registriert Event-Listener lazy und cleaned-up, begrenzt Metrik-Historien und normalisiert Payloads.
    -   `uiStore.ts`: Verwalten der Modals, persistente Einstellungen via Dexie.
-   **Analytics:**
    -   `src/cache/metricSeries.ts` stellt Struct-of-Arrays Puffer für Trendberechnungen bereit.
    -   `src/lib/utils/metrics.ts` nutzt Typed-Arrays zur latenzarmen Trend-, Statistik- und Health-Auswertung.
-   **Komponenten:**
    -   `StatusCard.svelte`: Enthält Circuit-Routing-Intelligenz, Ping-Historie, Statusbadges und automatische Layout-Anpassung.
    -   `IdlePanel.svelte`: Tweened Bootstrap-Balken, ARIA-Live Statusmeldungen, Retry-Ausgaben.
    -   `ActionCard.svelte`: Primärer Connect/Disconnect-Controller mit sequentieller Aktionswarteschlange, Queue-Indikatoren, haptischen Hover-States, Logging & Error-Toasts.
    -   `NetworkTools`, `TorChain`, `ConnectionDiagnostics`: unverändert, werden in Folgeaufträgen auf das neue Design gehoben.

Alle Komponenten nutzen geteilte Design-Tokens aus `src/app.css` und halten sich an die Barrierefreiheitsanforderungen (mind. 4.5:1 Kontrast).

### 2.3 Cache Layer (`/src/cache`)

-   `adaptiveCache.ts`: Generischer Cache mit konfigurierbarer Größe, TTL, LRU/LFU/FIFO-Eviction, Warmup-Plan und Statistik-API.
-   `metricSeries.ts`: Typed-Array Stacks (Struct-of-Arrays) für numerische Metriken, reduzieren GC-Druck in Trendberechnungen.
-   `index.ts`: Stellt `connectionTimelineCache`, `connectionSummaryCache` und `countryLookupCache` bereit, persistiert Snapshots in `localStorage`, bietet Warmup- und Invalidation-Hooks.
-   Cache-Snapshots werden zyklisch aktualisiert (`cacheConnectionTimeline`, …) und können via `warmupCaches()` hydratisiert werden – wichtig für Offline-Start.

## 3. New Features in V2.1

### 3.1 New Identity Functionality
- Added ability to request new Tor circuits via `new_identity` command
- Full integration with frontend UI using dedicated button
- Uses arti-client's `reconfigure` and `retire_all_circs` for identity refresh

### 3.2 Logging System
- Centralized log storage in AppState with thread-safe access
- Automatic log rotation (max 1000 entries)
- Commands for log retrieval and clearing
- Logs are stored as JSON lines containing the log level, timestamp and message
- The Logs modal in the Svelte UI lets users filter by level and highlights
  warnings and errors with dedicated colours

### 3.3 Documentation Updates
- Comprehensive changelog tracking
- Task list for future improvements

### 3.4 Hardware Security Module
Support for PKCS#11 modules is available when compiling with the `hsm` feature.
`SecureHttpClient` loads the library specified by the `TORWELL_HSM_LIB`
environment variable and can access keys on a hardware token.

### 3.5 Circuit Management
The command `build_new_circuit` allows creating additional circuits on demand.
`circuit_metrics` returns the number of currently open circuits and the age of
the oldest one when compiled with the `experimental-api` feature (pass
`--features experimental-api` or use `task build`).

### 3.6 Mobile Workflow
Running `task mobile:android` or `task mobile:ios` builds a Capacitor-based
mobile shell. The backend runs a small HTTP bridge on port 1421 when compiled
with the `mobile` feature so that the web app can control the Tor client.

### 3.7 Metrics Retrieval Limit
The `load_metrics` command now accepts an optional `limit` parameter to
restrict the number of entries returned. If no limit is provided, the backend
returns the most recent 100 metric points. Frontend components pass their
desired limit to fetch only the data required for their charts. A valid
session token is required for this command, aligning it with the
authentication model used by other API endpoints.

## 4. Build Process

The application is built as a standard Tauri project:

1.  The SvelteKit frontend is built into a set of static assets (HTML, CSS, JS).
2.  The Rust backend is compiled into a binary.
3.  The Tauri bundler packages the frontend assets and the Rust binary into a single, native executable für das Zielsystem (z. B. `.app` für macOS, `.exe` für Windows).
4.  Optional: `scripts/benchmarks/connection_startup.sh` führt nach erfolgreichem Build einen Bootstrap-Benchmark aus und speichert Rohdaten unter `.benchmarks/`.

## 5. Error States

Errors from the backend are emitted through the `tor-status-update` event. The main variants are:

- `NotConnected` – a command requiring an active Tor connection was invoked while disconnected.
- `AlreadyConnected` – historischer Fehlerfall; `TorManager::connect` fängt erneute Aufrufe nun ab und meldet Erfolg.
- `Bootstrap` – the Tor client failed to bootstrap.
- `NetDir` – the network directory could not be retrieved.
- `Circuit` – building or retrieving a circuit failed.
- `Identity` – refreshing the Tor identity was unsuccessful.

The serialized error message is provided in the event's `errorMessage` field so the frontend can display user-friendly feedback.

## 6. Traffic Statistics

Version 2.2 introduces live traffic counters. `TorManager` exposes the total bytes
sent and received via `traffic_stats()`, and the `get_traffic_stats` Tauri command
passes these values to the frontend. The main status card now periodically
displays the aggregate traffic in megabytes.

## 7. Bootstrap Progress

`TorManager::connect_once` streams bootstrap events from `arti-client` and
invokes a progress callback with the current percentage. The `connect` command
forwards these values through `tor-status-update` events so the frontend can
render a live progress bar. `torStore.ts` listens for the events to keep its
`bootstrapProgress` state updated, and `IdlePanel.svelte` displays this value to
give users visual feedback during connection.

## 8. Bridge Configuration and Circuit Isolation

- `TorManager` accepts a list of bridge lines which are applied to the `TorClientConfig` when connecting.
- The command `set_bridges` stores user-provided bridges so that censored networks can reach the Tor network.
- `get_isolated_circuit` manages multiple isolation tokens per domain, allowing several parallel circuits to the same hostname.
- The settings modal in the Svelte UI offers a simple list of bridges the user can enable or disable.

## 9. Accessibility Strategy

Das Skript führt sequentiell aus:
1. `bun run check` — Typen- & A11y-Checks.
2. `bun run test` — Vitest-Unit- und Integrationstests.
3. `cargo test --locked` im `src-tauri`-Crate.

Fehlschläge werden sofort gemeldet (Exit-Code ≠ 0). Für Umgebungen ohne die benötigten Systembibliotheken dokumentiert `docs/plan.md` entsprechende CI-Hooks.

## 4. Benchmarks
`scripts/benchmarks/run_frontend_benchmarks.sh` nutzt `bun x vitest bench` für UI-Mikrobenchmarks (z. B. Store-Reducer, Layout-Berechnungen). Weitere Benchmarks können in diesem Ordner ergänzt werden.

```bash
scripts/benchmarks/run_frontend_benchmarks.sh -- --runInBand
```

Ergebnisse sollten p50/p95/p99-Latenzen im Terminal anzeigen. Integration mit `hyperfine` oder `cargo bench` folgt nach Definition konkreter Metriken (siehe Spec).

## 5. Architektur-Überblick
```mermaid
graph TD
    User((Benutzer)) -->|UI-Aktionen| Frontend
    Frontend -->|invoke| Backend
    Backend -->|Commands| TorManager
    TorManager -->|arti-client| TorNetz
    Backend -->|Events| Frontend
```

### Komponenten
- **Frontend (`/src`):** SvelteKit SPA mit State Stores (`torStore.ts`, `uiStore.ts`), Komponenten wie `StatusCard.svelte`, `ActionCard.svelte`, `IdlePanel.svelte`. Nutzt Dexie für persistente Einstellungen.
- **Backend (`/src-tauri`):** Rust-Crate mit `TorManager` (arti-Integration), `state.rs` (AppState, Log-Puffer), Tauri Commands (`commands.rs`), Fehlerenum (`error.rs`).
- **IPC Layer:** Tauri `invoke/listen` Events (`tor-status-update`, `metrics-update`, `log-update`).
- **Tooling:** Taskfile, Scripts in `/scripts`, Tests im Frontend (Vitest) & Backend (Cargo).

### Datenfluss (Connect)
```mermaid
sequenceDiagram
    participant UI
    participant Store
    participant Backend
    participant Tor

    UI->>Backend: connect()
    Backend->>Tor: Bootstrap starten
    Tor-->>Backend: Status-Events
    Backend-->>Store: tor-status-update
    Store-->>UI: Reaktive Anzeige
```

## 13. Environment Variables

Das Backend akzeptiert verschiedene Umgebungsvariablen zur Laufzeitkonfiguration.

- `TORWELL_CERT_URL` – HTTPS-Endpunkt zum Abrufen des Serverzertifikats.
- `TORWELL_CERT_PATH` – Lokaler Pfad zum abgelegten Zertifikat.
- `TORWELL_FALLBACK_CERT_URL` – Optionale Ausweich-URL für Zertifikatsupdates.
- `TORWELL_SESSION_TTL` – Lebensdauer eines Session-Tokens in Sekunden (Standard `3600`).
- `TORWELL_MAX_LOG_LINES` – Maximale Anzahl von Logzeilen, die in `torwell.log` aufbewahrt werden (Standard `1000`).
- `TORWELL_MAX_MEMORY_MB` – Schwellenwert für Speichernutzung, ab dem Warnungen ausgegeben werden (Standard `1024`).
- `TORWELL_MAX_CIRCUITS` – Maximale Anzahl erlaubter paralleler Tor-Circuits (Standard `20`).
- `TORWELL_METRICS_FILE` – Pfad für aufgezeichnete Metrikpunkte (Standard `metrics.json`).
- `TORWELL_MAX_METRIC_LINES` – Maximale Zeilenanzahl der Metrikdatei (Standard `10000`).
- `TORWELL_MAX_METRIC_MB` – Maximale Dateigröße in MB (Standard `5`).
- `TORWELL_METRIC_INTERVAL` – Intervall der Metrikerfassung in Sekunden (Standard `30`).
- `TORWELL_LOG_ENDPOINT` – Optionaler HTTP-Endpunkt zum Weiterleiten von Logeinträgen.
- Bei Überschreitung dieser Limits erscheint ein Warnhinweis im Systemtray-Menü.


## 14. Session Tokens

Bestimmte Befehle wie `get_logs` oder `ping_host` benötigen ein gültiges Session-Token. Dieses lässt sich mit dem Kommando `request_token` abrufen und muss danach bei jedem Aufruf als `token`-Argument übergeben werden.

```ts
import { invoke } from '@tauri-apps/api/tauri';

async function fetchLogs() {
  const tok = await invoke<string>('request_token');
  return invoke('get_logs', { token: tok });
}
```

Die Tokens verfallen nach der in `TORWELL_SESSION_TTL` definierten Zeitspanne. Erhält der Client einen `401`-Fehler oder eine Meldung "Invalid session token", sollte umgehend ein neues Token angefordert und der Befehl erneut ausgeführt werden.

## 15. Memory Profiling & Allocator

-   **Allocator:** Das Rust-Backend nutzt `mimalloc` als globalen Allocator (`src-tauri/src/lib.rs`), um Fragmentierung zu reduzieren und Hot-Path-Latenzen zu glätten.
-   **Cache Limits:** `connectionTimelineCache` (max 16 Einträge / maxCost 256) und `connectionSummaryCache` (max 4) invalidieren sich automatisch bei Statuswechsel (`torStore` → `invalidateConnectionCaches`).
-   **Profiling-Skripte:**
    -   `scripts/benchmarks/run_massif.sh [testname]` – startet Valgrind Massif auf dem ausgewählten Integrationstest (`parallel_metrics_benchmark` als Default). Ergebnisse landen unter `src-tauri/target/memory-profiles/massif-*.out`.
    -   `scripts/benchmarks/run_heaptrack.sh [testname]` – erzeugt Heaptrack-Traces (`*.gz`) im selben Verzeichnis.
-   **Analyse:** `massif-visualizer` bzw. `heaptrack_gui` öffnen die Artefakte. Empfehlungen: Peaks < 200 MB für `parallel_metrics_benchmark`, Differenz zwischen Start/Ende < 5 MB.
-   **Operational Limits:** Profiling ist optional; CI kann über Feature-Toggles (Follow-up C7/C8) entscheiden. Skripte prüfen Tool-Verfügbarkeit und brechen mit Exit-Code ≠0 ab, falls Tools fehlen.

## 16. UI Backup

Vor Experimenten mit neuen Layouts kann die aktuelle Benutzeroberfläche
gesichert werden. Das Skript `scripts/backup_ui.sh` kopiert dazu den Inhalt von
`src/lib/components` in das Verzeichnis `src/lib/components_backup` und legt es
bei Bedarf an. Dieser Ordner ist in `.gitignore` eingetragen und wird nicht ins
Repository übernommen.

## 16. Diagnostics & Benchmark Status

Die im ursprünglichen Change Request erfassten Nacharbeiten wurden final
strukturiert und dokumentiert:

- **ConnectionDiagnostics & NetworkTools**: Der Modernisierungsbedarf ist im
  Roadmap-Milestone "Diagnostics UX" (siehe `docs/plan.md`) fest eingeplant und
  beschreibt die Anpassung an die neue Motion-/Glass-Sprache.
- **Trace/Timeline-Komponenten**: Das Spezifikationskapitel "Qualitätsziele"
  wurde um Animationsanforderungen für Diagnosen ergänzt, sodass neue Timeline
  Overlays `prefers-reduced-motion` respektieren müssen.
- **Benchmarking**: `scripts/benchmarks/connection_startup.sh` ist als offizieller
  Messpunkt dokumentiert. Das Skript misst Bootstrapping-Latenzen, protokolliert
  p50/p95/p99-Werte und wird im Kapitel "Build Process" referenziert.
- **Testmatrix**: Die Dokumentation verweist auf die erweiterte Testmatrix im
  Roadmap-Dokument; ältere Intel-GPUs werden ausdrücklich berücksichtigt.

Alle offenen Punkte aus `docs/todo/CR-0001.md` wurden damit in die zentrale
Dokumentation überführt; das CR-Blatt liegt zur Nachvollziehbarkeit im Archiv.

## 17. Release Notes

Ausführliche Release Notes für Version 2.5 befinden sich in `docs/ReleaseNotes.md`.
Das Dokument listet Highlights, Fixes, bekannte Probleme und Upgrade-Hinweise.


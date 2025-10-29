# Plan / Roadmap

Dieses Dokument strukturiert laufende und geplante Arbeitspakete. Pakete sind so geschnitten, dass sie parallelisierbar bleiben und minimale Konfliktflächen besitzen.

## Arbeitsprinzip
1. Doku-Sync erfolgt nach Abschluss aller technischen Pakete (ein Paket besitzt Schreibrecht auf `docs/DOCUMENTATION.md`).
2. Tests & Lint laufen über `scripts/tests/run_all.sh`; CI integriert Linux Desktop Dependencies (`pkg-config`, `libgtk-3-dev`, `webkit2gtk`).
3. Benchmarks liefern reproduzierbare p50/p95-Werte; Ergebnisse werden perspektivisch in Artefakten gespeichert.

## Work Breakdown Structure (WBS)

| ID | Paket | Beschreibung | Impact | Konfliktrisiko |
|----|-------|--------------|--------|----------------|
| P1 | UI Token Harmonisierung | Konsolidierung von Glassmorphism-Tokens in `src/app.css`, responsive Grid für `src/routes/+page.svelte`. | Hoch | Mittel |
| P2 | Motion & Feedback | Mikroanimationen (`IdlePanel`, `StatusCard`), `prefers-reduced-motion` Utility, Tests für Motion-Reducer. | Mittel | Niedrig |
| P3 | Connection Resilience | Verbesserungen am `invokeWithRetry`, Listener-Lifecycle im `torStore`, Tests für Backoff-Strategie. | Hoch | Niedrig |
| P4 | Arti Guardrails | Erweiterte Tests für GeoIP, Routing-Policies, Logging & Error-Surface in `src-tauri`. | Mittel | Niedrig |
| P5 | Diagnostics UX | Überarbeitung `ConnectionDiagnostics` & `NetworkTools`, neue Visualisierungen. | Mittel | Hoch |
| P6 | Docs & Security Sync | Pflege `docs/DOCUMENTATION.md`, Threat-Model (STRIDE), Architektur-Updates, Konsolidierung der CR-Notizen. | Mittel | Mittel |
| P7 | Automation & CI Hooks | Integration von `scripts/tests/run_all.sh` in GitHub Actions, pre-commit Setup (lint, fmt, clippy). | Hoch | Mittel |
| P8 | Benchmark Pipeline | Aufbau weiterer Skripte (`hyperfine`, `cargo bench`), Vergleichsmetriken dokumentieren. | Mittel | Niedrig |
| P9 | Observability Enhancements | Ausbau Telemetrie (p95 Alerts, Logging-Exports), Update der SLAs. | Mittel | Mittel |
| P10| Security Hardening | Fuzz-Targets für kritische Commands, Secrets-Scanning in CI. | Hoch | Mittel |

## Priorisierte Auswahl
Mangels externer Vorgaben sind aktuell P1–P4 im aktiven Sprint. Dieses Paket (P6 + P7 + P8 Teil 1) wurde umgesetzt, um Dokumentations- und Tooling-Basis zu schließen. Folgeaufträge priorisieren P5, P8 (Restarbeiten) und P9.

## CI-/Lint-Hooks (P7)
1. **GitHub Actions**: Node/Bun-Setup, Cache für `~/.cargo`, Installation der Linux Abhängigkeiten (`sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev`). Danach Aufruf `scripts/tests/run_all.sh`.
2. **Pre-Commit**: Hooks für `bun run check`, `bun run test --run tests/unit`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`. Optional `scripts/benchmarks/run_frontend_benchmarks.sh -- --summary` als non-blocking Step.
3. **Nightly**: Geplanter Workflow zum Ausführen von Benchmarks & Security-Scans (OSS Review Toolkit, License-Checks).

## Risiken & Mitigation
- **GPU/Blur-Inkompatibilität**: Fallback-Styles via `@supports not (backdrop-filter)` implementieren.
- **Rate-Limit bei Tauri-Commands**: Exponentielles Backoff + Jitter, Logging bei Überschreitung.
- **Test-Laufzeit**: Bun/Vitest parallelisierbar, `cargo test` kann mit `-- --test-threads=1` laufen, falls UI/IPC-Mocks nötig.
- **CI-Dependencies**: Fehlende GTK/WebKit Libs führen zu Build-Brüchen – Setup-Skripte dokumentiert (siehe oben).

## Nächste Schritte
- P7: CI-Pipeline mit oben genannten Hooks provisionieren.
- P8: Zusätzliche Benchmarks (UI Render, TorManager Metrics) definieren und Skripte ergänzen.
- P9: Observability-Plan (Alerting, Dashboards) detaillieren und in Spec aufnehmen.
- P10: Security-Fuzzing-Roadmap erstellen, Abhängigkeiten (cargo-fuzz) evaluieren.

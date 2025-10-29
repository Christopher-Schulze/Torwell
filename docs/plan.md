# Plan / Roadmap

## Arbeitsprinzip
Dieses Dokument bündelt die aktuellen Arbeitspakete für das UI-/Resilienz-Upgrade sowie das neue Quality-Gate-Programm. Pakete sind so geschnitten, dass sie parallelisiert werden können und minimale Konfliktflächen besitzen.

## Work Breakdown Structure (WBS)

| ID | Paket | Beschreibung | Impact | Konfliktrisiko |
|----|-------|--------------|--------|----------------|
| P1 | Visual Identity Refresh | Überarbeitung von `src/app.css`, Harmonisierung der Glas-Surface-Token, responsives Grid in `src/routes/+page.svelte`. | Hoch | Mittel |
| P2 | Motion & Micro-Interactions | Tweened Fortschrittsbalken, Status-Transitions (`IdlePanel`, `StatusCard`), Utility für Reduced-Motion. | Mittel | Niedrig |
| P3 | Status Intelligence | Aufwertung `StatusCard` inkl. Route-Badges, Ping-Historie, adaptiver Kopplung an Policy-Report. | Hoch | Mittel |
| P4 | Connection Resilience | Verbesserte `invoke`-Retry-Strategie, Guarding in `torStore`, robustes Listener-Lifecycle-Management. | Hoch | Niedrig |
| P5 | Arti Integration Guardrails | Tests für Routing-Policy & GeoIP, Verifikation von `TorManager::ensure_unique_route`, Logging-Verbesserungen. | Mittel | Niedrig |
| P6 | Documentation Hub Sync | Aktualisierung `docs/DOCUMENTATION.md`, Pflege `docs/spec.md`, `docs/todo.md`, Threat-Model. | Mittel | Mittel |
| P7 | Diagnostics UX | Verbesserte Darstellung in `ConnectionDiagnostics` & `NetworkTools` (Follow-up). | Mittel | Hoch |
| P8 | Automation & Tooling | Ergänzung von `/scripts/tests/` Runnern, CI-Hinweise (Baseline erledigt, Erweiterungen laufend). | Mittel | Niedrig |
| P9 | Coverage & Reporting | Einrichtung `scripts/tests/coverage.sh`, Integration `cargo tarpaulin` & `vitest --coverage`, Dokumentation in `docs/todo/CR-0002.md`. | Hoch | Niedrig |
| P10 | Benchmarks & Performance Baselines | Criterion-Bench für `TorManager`, Playwright-Perf-Suite inkl. Baseline-Daten & Reporter. | Hoch | Mittel |
| P11 | Fuzzing & Hardening | `cargo-fuzz` Targets für Parser-/Policy-Funktionen (`secure_http`, `tor_manager`), Orchestrierung über `scripts/tests/fuzz.sh`. | Hoch | Niedrig |

## Priorisierte Auswahl
Mangels weiterer Vorgaben werden P1–P6 als abgeschlossen betrachtet. Aktueller Fokus liegt auf P8–P11, um das Quality-Gate-Programm produktiv zu machen. P7 bleibt als dokumentierter Next Step für zukünftige UI-Aufwertungen.

## Meilensteine
1. **Milestone A – UI & Motion**: Abschluss P1–P3.
2. **Milestone B – Resilienz & Backend Guards**: Abschluss P4–P5.
3. **Milestone C – Docs & Enablement**: Abschluss P6, Übergabe an QA.
4. **Milestone D – Quality Gates**: Abschluss P8–P11, automatisierte Regressionserkennung aktiv.

## Risiken & Mitigation
- **GPU/Blur-Inkompatibilität**: Fallback-Styles definiert (`@supports not (backdrop-filter)`).
- **Rate-Limit bei Tauri-Commands**: Exponentielle Retry-Strategie mit Jitter, Logging wenn Limit überschritten.
- **Test-Laufzeit**: Rust- und Frontend-Checks parallelisierbar, orchestriert über `scripts/tests/run_all.sh`.
- **Playwright-Stabilität**: Dev-Server-Startup überwacht, Timeout/Retry-Logik im Benchmark-Runner.
- **Tarpaulin-Toolchain**: Installation via `cargo install` dokumentiert, Script prüft lokale Verfügbarkeit.

## Nächste Schritte
- P7 in eigenem Auftrag adressieren.
- Benchmarks & Fuzzing in CI-Jobs integrieren, Artefakte hochladen.
- Coverage-Grenzwerte definieren (z. B. Frontend 80 %, Backend 70 %) und als Fail-Gate einführen.

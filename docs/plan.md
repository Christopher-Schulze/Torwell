# Plan / Roadmap

Dieses Dokument strukturiert laufende und geplante Arbeitspakete. Pakete sind so geschnitten, dass sie parallelisierbar bleiben und minimale Konfliktflächen besitzen.

## Arbeitsprinzip
1. Doku-Sync erfolgt nach Abschluss aller technischen Pakete (ein Paket besitzt Schreibrecht auf `docs/DOCUMENTATION.md`).
2. Tests & Lint laufen über `scripts/tests/run_all.sh`; CI integriert Linux Desktop Dependencies (`pkg-config`, `libgtk-3-dev`, `webkit2gtk`).
3. Benchmarks liefern reproduzierbare p50/p95-Werte; Ergebnisse werden perspektivisch in Artefakten gespeichert.

## Work Breakdown Structure (WBS)

| ID | Paket | Beschreibung | Impact | Konfliktrisiko | Status |
|----|-------|--------------|--------|----------------|--------|
| P1 | Visual Identity Refresh | Überarbeitung von `src/app.css`, Harmonisierung der Glas-Surface-Token, responsives Grid in `src/routes/+page.svelte`. | Hoch | Mittel | ✅ Abgeschlossen |
| P2 | Motion & Micro-Interactions | Tweened Fortschrittsbalken, Status-Transitions (`IdlePanel`, `StatusCard`), Utility für Reduced-Motion. | Mittel | Niedrig | ✅ Abgeschlossen |
| P3 | Status Intelligence | Aufwertung `StatusCard` inkl. Route-Badges, Ping-Historie, adaptiver Kopplung an Policy-Report. | Hoch | Mittel | ✅ Abgeschlossen |
| P4 | Connection Resilience | Verbesserte `invoke`-Retry-Strategie, Guarding in `torStore`, robustes Listener-Lifecycle-Management. | Hoch | Niedrig | ✅ Abgeschlossen |
| P5 | Arti Integration Guardrails | Tests für Routing-Policy & GeoIP, Verifikation von `TorManager::ensure_unique_route`, Logging-Verbesserungen. | Mittel | Niedrig | ✅ Abgeschlossen |
| P6 | Documentation Hub Sync | Aktualisierung `docs/DOCUMENTATION.md`, Anlegen von Spec/Backlog-Struktur, Pflege `docs/todo`. | Mittel | Mittel | ✅ Abgeschlossen |
| P7 | Diagnostics UX | Modernisierung `ConnectionDiagnostics` & `NetworkTools`, Timeline-Overlay, Motion-Token-Sharing. | Mittel | Mittel | 🔄 Geplant (Milestone D) |
| P8 | Automation & Tooling | Ergänzung von `/scripts/tests/` Runnern, CI-Hinweise. | Niedrig | Niedrig | 🔄 Geplant |
| P9 | Benchmark Automation | `scripts/benchmarks/connection_startup.sh`, Integration in Release-CI, Latenz-Reporting. | Mittel | Niedrig | ✅ Abgeschlossen |

## Priorisierte Auswahl
Milestones A–C sind produktiv gesetzt. Milestone D bündelt die verbliebenen Diagnostics-UX-Anpassungen (P7) und zusätzliche CI-Hooks (P8).

## Meilensteine
1. **Milestone A – UI & Motion**: Abschluss P1–P3. ✅ Delivered in v2.5.
2. **Milestone B – Resilienz & Backend Guards**: Abschluss P4–P5. ✅ Delivered in v2.5.
3. **Milestone C – Docs & Enablement**: Abschluss P6 & P9, QA-Begleitung inklusive Benchmark-Dashboards. ✅ Delivered in v2.5.
4. **Milestone D – Diagnostics Experience**: Umsetzung P7 & P8 mit Fokus auf Timeline-Komponenten und automatisierte Checks. ⏳ Offen.

## Risiken & Mitigation
- **GPU/Blur-Inkompatibilität**: Fallback-Styles via `@supports not (backdrop-filter)` implementieren.
- **Rate-Limit bei Tauri-Commands**: Exponentielles Backoff + Jitter, Logging bei Überschreitung.
- **Test-Laufzeit**: Bun/Vitest parallelisierbar, `cargo test` kann mit `-- --test-threads=1` laufen, falls UI/IPC-Mocks nötig.
- **CI-Dependencies**: Fehlende GTK/WebKit Libs führen zu Build-Brüchen – Setup-Skripte dokumentiert (siehe oben).

## Testmatrix
- **Desktop macOS 13+ (Apple Silicon, Intel GPU)**: UI & Bootstrap-Benchmark.
- **Windows 11 (Intel iGPU, AMD dGPU)**: Resilienztests, Motion-Reduced Validation.
- **Ubuntu 22.04 (Wayland/X11, Intel iGPU)**: Fokus auf Blur-Fallbacks und IdlePanel.

## Nächste Schritte
- Milestone D planen (Design-Vorlauf, UX-Research für Diagnostics).
- Benchmarking der Animationen auf älteren Intel-Macs (Follow-up erforderlich).

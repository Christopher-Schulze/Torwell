# Plan / Roadmap

## Arbeitsprinzip
Dieses Dokument bündelt die aktuellen Arbeitspakete für das UI-/Resilienz-Upgrade. Pakete sind so geschnitten, dass sie parallelisiert werden können und minimale Konfliktflächen besitzen.

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
- **GPU/Blur-Inkompatibilität**: Fallback-Styles definiert (`@supports not (backdrop-filter)`).
- **Rate-Limit bei Tauri-Commands**: Exponentielle Retry-Strategie mit jitter, Logging wenn Limit überschritten.
- **Test-Laufzeit**: Rust- und Frontend-Checks parallelisierbar, können über `scripts/tests/run_all.sh` orchestriert werden.

## Testmatrix
- **Desktop macOS 13+ (Apple Silicon, Intel GPU)**: UI & Bootstrap-Benchmark.
- **Windows 11 (Intel iGPU, AMD dGPU)**: Resilienztests, Motion-Reduced Validation.
- **Ubuntu 22.04 (Wayland/X11, Intel iGPU)**: Fokus auf Blur-Fallbacks und IdlePanel.

## Nächste Schritte
- Milestone D planen (Design-Vorlauf, UX-Research für Diagnostics).
- Benchmarking der Animationen auf älteren Intel-Macs (Follow-up erforderlich).

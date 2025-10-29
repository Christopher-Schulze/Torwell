# Plan / Roadmap

## Arbeitsprinzip
Dieses Dokument bündelt die aktuellen Arbeitspakete für das UI-/Resilienz-Upgrade. Pakete sind so geschnitten, dass sie parallelisiert werden können und minimale Konfliktflächen besitzen.

## Work Breakdown Structure (WBS)

| ID | Paket | Beschreibung | Impact | Konfliktrisiko |
|----|-------|--------------|--------|----------------|
| P1 | Visual Identity Refresh | Überarbeitung von `src/app.css`, Harmonisierung der Glas-Surface-Token, responsives Grid in `src/routes/+page.svelte`. | Hoch | Mittel |
| P2 | Motion & Micro-Interactions | Tweened Fortschrittsbalken, Status-Transitions (`IdlePanel`, `StatusCard`), Utility für Reduced-Motion. | Mittel | Niedrig |
| P3 | Status Intelligence | Aufwertung `StatusCard` inkl. Route-Badges, Ping-Historie, adaptiver Kopplung an Policy-Report. | Hoch | Mittel |
| P4 | Connection Resilience | Verbesserte `invoke`-Retry-Strategie, Guarding in `torStore`, robustes Listener-Lifecycle-Management. | Hoch | Niedrig |
| P5 | Arti Integration Guardrails | Tests für Routing-Policy & GeoIP, Verifikation von `TorManager::ensure_unique_route`, Logging-Verbesserungen. | Mittel | Niedrig |
| P6 | Documentation Hub Sync | Aktualisierung `docs/DOCUMENTATION.md`, Anlegen von Spec/Backlog-Struktur, Pflege `docs/todo`. | Mittel | Mittel |
| P7 | Diagnostics UX | Verbesserte Darstellung in `ConnectionDiagnostics` & `NetworkTools` (Future Work). | Mittel | Hoch |
| P8 | Automation & Tooling | Ergänzung von `/scripts/tests/` Runnern, CI-Hinweise (Future Work). | Niedrig | Niedrig |

## Priorisierte Auswahl
Mangels weiterer Vorgaben werden P1–P6 sofort umgesetzt. P7–P8 bleiben als dokumentierte Next Steps.

## Meilensteine
1. **Milestone A – UI & Motion**: Abschluss P1–P3.
2. **Milestone B – Resilienz & Backend Guards**: Abschluss P4–P5.
3. **Milestone C – Docs & Enablement**: Abschluss P6, Übergabe an QA.

## Risiken & Mitigation
- **GPU/Blur-Inkompatibilität**: Fallback-Styles definiert (`@supports not (backdrop-filter)`).
- **Rate-Limit bei Tauri-Commands**: Exponentielle Retry-Strategie mit jitter, Logging wenn Limit überschritten.
- **Test-Laufzeit**: Rust- und Frontend-Checks parallelisierbar, können über `scripts/tests/run_all.sh` orchestriert werden.

## Nächste Schritte
- P7 & P8 in eigenem Auftrag adressieren.
- Benchmarking der Animationen auf älteren Intel-Macs (Follow-up erforderlich).

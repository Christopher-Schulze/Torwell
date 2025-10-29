# Plan / Roadmap

## Arbeitsprinzip
Dieses Dokument bündelt die aktuellen Arbeitspakete für das UI-/Resilienz-Upgrade. Pakete sind so geschnitten, dass sie parallelisiert werden können und minimale Konfliktflächen besitzen.

## Work Breakdown Structure (WBS)

| ID | Paket | Beschreibung | Impact | Konfliktrisiko |
|----|-------|--------------|--------|----------------|
| R1 | GPU-Spec & Docs Sync | Spezifikation/Plan aktualisieren, Threat-Model-Aktualisierungen und Annahmen zu Shader-Cache & Renderloop dokumentieren. | Hoch | Niedrig |
| R2 | Renderer-Core (wgpu) | Neues Modul `renderer` mit Adapter-Discovery, Worker-Thread, Renderloop-Steuerung und Triple-Buffering. | Hoch | Mittel |
| R3 | Shader-Cache & Warmup | Hash-basierter Cache unter `~/Library/Application Support/Torwell/shader_cache`, Warmup im Worker inkl. Override per Env. | Hoch | Niedrig |
| R4 | CPU↔GPU Sync & Metrics | Fence-Handling, `FrameMetrics` + Percentiles, Events + `get_frame_metrics` Command, Integration in `AppState`. | Hoch | Mittel |
| R5 | Screenshot/Headless Tests | CLI `renderer_capture`, Integrationstests (`renderer_tests.rs`), Skripte unter `/scripts/tests/` inkl. Hash-Validierung. | Hoch | Mittel |
| R6 | Docs Hub Update | `docs/DOCUMENTATION.md` GPU-Abschnitt, TODO-Backlog ergänzen, SUMMARY.md aktualisieren. | Mittel | Mittel |
| R7 | Follow-up Backlog | Erweiterte Shader (PostFX), Benchmark-Szenarien, GPU-Fallback-UX vorbereiten (Dokumentation in `docs/todo`). | Mittel | Niedrig |
| R8 | CI/Automation Hooks | GitHub Actions/Taskfile-Erweiterung für GPU-Checks (auf spätere Aufträge verschoben). | Mittel | Hoch |

## Priorisierte Auswahl
Für diesen Auftrag werden R1–R6 umgesetzt, um GPU-Backend, Cache, Telemetrie und Tests vollständig bereitzustellen. R7–R8 bleiben als nachgelagerte Maßnahmen dokumentiert.

## Meilensteine
1. **Milestone Γ – Renderer Core**: Abschluss R1–R3 (Initialisierung, Cache, Threading).
2. **Milestone Δ – Telemetrie & Sync**: Abschluss R4 (Frame-Metriken, Commands, Events).
3. **Milestone Ε – Tests & Enablement**: Abschluss R5–R6, Erstellung der Headless-/Screenshot-Pipeline.

## Risiken & Mitigation
- **Kein kompatibler GPU-Adapter vorhanden**: Renderer degradiert und liefert Telemetrie-Flag `available = false`; Tests erkennen und skippen kontrolliert.
- **Shader-Cache-Korruption**: Hash-Index wird atomar ersetzt; Warmup überschreibt nur bei Hash-Mismatch.
- **Test-Laufzeit**: GPU-Headless-Tests werden parallel zu bestehenden Rust-Tests im Runner ausgeführt, Caching minimiert Setup-Zeit.
- **Screenshot-Drift**: Referenz wird aus deterministischem Shader abgeleitet und per CPU-Rechnung validiert, kein statisches Binary-Baseline notwendig.

## Nächste Schritte
- R7 vorbereiten: Erweiterte Shader, Postprocessing und GPU-Fallback-UI im Backlog priorisieren.
- R8: Automatisierte GPU-Prüfungen in CI evaluieren (Benötigt GPU-fähige Runner oder Software-Fallback-Builds).

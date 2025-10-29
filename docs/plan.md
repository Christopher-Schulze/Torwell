# Plan / Roadmap

## Arbeitsprinzip
Dieses Dokument bündelt die aktuellen Arbeitspakete für das UI-/Resilienz-Upgrade. Pakete sind so geschnitten, dass sie parallelisiert werden können und minimale Konfliktflächen besitzen.

## Work Breakdown Structure (WBS)

| ID | Paket | Beschreibung | Impact | Konfliktrisiko |
|----|-------|--------------|--------|----------------|
| P1 | Devcontainer Provisioning | Bereitstellung von `.devcontainer` inkl. Features (Node, Rust, Bun), VSCode/Neovim Defaults, Post-Create Bootstrap. | Hoch | Niedrig |
| P2 | Task Automation Suite | Ausbau des `Taskfile.yml` um Lint/Test/Bench-Ketten, Helper-Skripte für Task-Runner. | Hoch | Mittel |
| P3 | Git Hooks & QA Guardrails | `.githooks/` pre-commit mit lint/test, standardisierte bootstrap-Integration. | Mittel | Niedrig |
| P4 | Repository Standards | `.editorconfig`, Issue/PR-Templates, `.env.example` Pflege. | Mittel | Niedrig |
| P5 | Onboarding Automation | `scripts/utils/bootstrap.sh` + Dokumentation der Workflows, Ankopplung an Devcontainer. | Hoch | Mittel |
| P6 | Benchmark Harness | Skripte unter `scripts/benchmarks/` für Build- und Compile-Duration, Task-Verknüpfung. | Mittel | Niedrig |
| P7 | Future CI Enhancements | Containerisierte CI-Pipeline, Remote Cache Setup (Folgeauftrag). | Mittel | Hoch |
| P8 | Advanced Telemetry Tooling | Observability CLI & metrics dashboards (Folgeauftrag). | Mittel | Hoch |

## Priorisierte Auswahl
Mangels weiterer Vorgaben werden P1–P6 sofort umgesetzt. P7–P8 bleiben als dokumentierte Next Steps.

## Meilensteine
1. **Milestone A – Dev Enablement Core**: Abschluss P1–P3.
2. **Milestone B – Standards & Automation**: Abschluss P4–P6.
3. **Milestone C – Expansion**: Vorbereitung P7–P8.

## Risiken & Mitigation
- **Container Build Zeit**: Post-Create Skripte minimieren Setup, parallelisierbare apt-Installationen.
- **Task Runner Verfügbarkeit**: `scripts/utils/run_task.sh` nutzt fallback auf `bunx`/`npx` falls `task` fehlt.
- **Hook-Akzeptanz**: Bootstrap setzt `core.hooksPath`, optionale Flags für CI-Umgebungen.

## Nächste Schritte
- P7 & P8 in eigenem Auftrag adressieren.
- CI-Umgebungen auf Container-Image umstellen (Follow-up erforderlich).

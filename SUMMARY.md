## Änderungen
- Aktualisierte Projektspezifikation (Scope, Assumptions, SLAs) und Dokumentations-Hub inklusive Architektur-Übersicht & STRIDE-Threat-Model.
- Roadmap um CI-/Lint-Hooks, Benchmark- und Security-Pakete erweitert; zentrale Doku-Links konsolidiert.
- Neue Skripte: `scripts/tests/run_all.sh`, `scripts/utils/dev_run.sh`, Benchmark-Skeleton (`scripts/benchmarks/`).

## Kommandos
- Tests: `scripts/tests/run_all.sh`
- Benchmarks: `scripts/benchmarks/run_frontend_benchmarks.sh`
- Dev: `scripts/utils/dev_run.sh`

## Nächste Schritte
- CI-Workflows mit Bun/Rust-Setup erstellen und `scripts/tests/run_all.sh` integrieren.
- Zusätzliche Benchmarks (Rust, Hyperfine) hinzufügen und Ergebnisse versionieren.
- Observability-Plan (Alerts, Dashboards) detaillieren und in Spec überführen.

## Annahmen
- Bun ≥1.1, Rust ≥1.77, sowie `pkg-config`, `glib-2.0`, `openssl` stehen auf Dev-/CI-Systemen bereit.
- Vitest Benchmarks existieren (oder liefern `0` Tests) und können via `bun x vitest bench` ausgeführt werden.

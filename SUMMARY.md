## Änderungen
- Qualitätstor-Skripte ergänzt: `scripts/tests/run_all.sh` orchestriert ESLint, Clippy, Unit-, Integrations- und UI-Snapshot-Tests inklusive `svelte-kit sync` Bootstrap.
- Coverage-, Benchmark- und Fuzzing-Infrastruktur aufgebaut (`scripts/tests/coverage.sh`, `scripts/benchmarks/run_all.sh`, `scripts/tests/fuzz.sh`) inkl. Baseline-/Artefakt-Verzeichnisse, neuen npm-Skripten sowie Criterion-/Playwright-/cargo-fuzz-Setups.
- Frontend-Tooling erweitert (ESLint-Konfiguration, Snapshot-Test, Vitest-Coverage, Playwright-Perf-Konfiguration, neue Dev-Dependencies) und Rust-Crate um Criterion-Bench + fuzzing Hooks ergänzt.
- Dokumentation aktualisiert (`docs/spec.md`, `docs/plan.md`, `docs/todo.md`, `docs/todo/CR-0002.md`) für Quality Gates & Backlog.

## Kommandos
- Tests gesamt: `scripts/tests/run_all.sh`
- Coverage: `scripts/tests/coverage.sh`
- Benchmarks: `scripts/benchmarks/run_all.sh`
- Fuzzing: `scripts/tests/fuzz.sh`

## Nächste Schritte
- System-Paket `libglib2.0-dev` in CI/Devcontainer installieren, um Tarpaulin & Criterion lauffähig zu machen.
- Playwright-Browser einmalig installieren (`npx playwright install --with-deps`), Baselines nach erfolgreichem Lauf aktualisieren.
- Fuzzing-Läufe in CI (nightly) integrieren, Frontend-Testabdeckung ausbauen.

## Annahmen
- Nightly Rust-Toolchain verfügbar für cargo-fuzz (`rustup toolchain install nightly`).
- Playwright darf lokalen Dev-Server auf Port 4173 starten; Port ist frei.
- `glib-2.0` wird extern bereitgestellt (siehe TODO in CR-0002).

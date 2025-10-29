# Spec (Zielzustand)

## Scope
- Zentralisiere Projektwissen gemäß Organisationsleitfaden: `docs/DOCUMENTATION.md` als Hub, vollständige Pflege von `spec.md`, `plan.md`, `todo.md`.
- Liefere reproduzierbare Qualitäts-Gates über `scripts/tests/run_all.sh` und Dev-Workflow via `scripts/utils/dev_run.sh`.
- Stelle Benchmark-Grundgerüst unter `scripts/benchmarks/` bereit (Vitest Bench für UI, Hookpunkte für Rust/Cargo-Benchmarks).
- Erweitere Architektur- und Sicherheitsdokumentation um eine präzise Übersicht sowie Threat-Model (STRIDE) und verweise auf relevante Artefakte.

## Nicht-Ziele
- Keine grundlegende Änderung des Build-Tooling (Tauri, SvelteKit, Cargo) oder der Paketmanager (Bun, Cargo).
- Keine Erweiterung der bestehenden Feature-Sets der UI oder Backend-Kommandos über Dokumentation/Tooling hinaus.
- Keine Auslieferung produktiver Benchmark-Suites – Fokus auf lauffähige Skeletons & Integrationspunkte.

## Annahmen
- Entwicklungsumgebungen verfügen über Bun ≥1.1, Node.js ≥20, Rust ≥1.77 und Cargo.
- Systemweite Abhängigkeiten für `cargo test` (insbesondere `pkg-config`, `glib-2.0`, `openssl`) sind installiert.
- `bun` ist bevorzugter Runner für Frontend-Builds/Tests; `npm`/`pnpm` werden nicht offiziell unterstützt.
- Benchmarks nutzen `vitest bench`; Rust-Benchmarks folgen später via `cargo bench` oder Criterion.

## Constraints
- Shell-Skripte folgen `set -euo pipefail`, vermeiden Inline-Abhängigkeiten und akzeptieren Forwarded-Args.
- Dokumentation verbleibt in `/docs/` mit Markdown (UTF-8, 120 Zeichen Zeilenlimit empfohlen) und Mermaid für Diagramme.
- Threat-Model muss STRIDE-Analyse beinhalten und in `docs/DOCUMENTATION.md` referenziert werden.

## Schnittstellen & Workflows
- **Tests:** `scripts/tests/run_all.sh` orchestriert Bun Checks, Vitest Suites und `cargo test` (mit Option `-- <args>` zur Testfilterung).
- **Dev-Run:** `scripts/utils/dev_run.sh` startet Tauri-Entwicklungssession (`bun run tauri:dev`), respektiert zusätzliche CLI-Argumente.
- **Benchmarks:** `scripts/benchmarks/run_frontend_benchmarks.sh` ruft `bun x vitest bench` auf; zusätzliche Skripte können in diesem Ordner abgelegt werden und werden aus `DOCUMENTATION.md` verlinkt.
- **Documentation Hub:** `docs/DOCUMENTATION.md` referenziert Spec, Plan, TODOs, File/Wire Map, Security-Dokumente und externe Ressourcen.

## Qualitätsziele & SLAs
- **Performance:** UI-Animationen <16 ms Framebudget; Benchmark-Skripte liefern p50/p95-Ausgabe sobald Tests vorliegen.
- **Resilienz:** Test-Skript bricht beim ersten Fehler ab und signalisiert fehlende Abhängigkeiten klar.
- **DX:** Ein Kommando für Tests (`scripts/tests/run_all.sh`), eines für Dev (`scripts/utils/dev_run.sh`), strukturierte Dokumentationslinks, konsistenter Markdown-Stil.
- **Sicherheit:** Threat-Model gepflegt, Skripte vermeiden unsichere Temp-Verzeichnisse, alle externen Eingaben validiert (z. B. Argumentweitergabe mit Quotes).

## Telemetrie & Logging
- Test-/Bench-Skripte geben klare Statusmeldungen (Start/Erfolg/Fehler) aus.
- Dokumentation verweist auf bestehende Telemetrie (Tor Bootstrap, Metrics-Burst) und aktualisiert SLAs (Bootstrap-Update ≥ alle 250 ms, Fehleranzeige ≤ 500 ms).

## Migration / Rollout
- Bestehende Workflows (`bun run check`, `bun run test`, `cargo test`) bleiben unverändert nutzbar.
- Neue Skripte werden in CI-Pipeline integriert, sobald Hooks in `docs/plan.md` umgesetzt sind (Follow-up mit konkreten CI-Konfigurationen).

## Offene Fragen
- Benötigte Zusatzmetriken für Benchmarks (FPS, Memory) werden nach ersten UI-Benchmarks definiert.
- CI-Umgebungen ohne grafischen Stack: Evaluierung von Headless-/Mock-Layern für Tauri Tests steht aus.
- Definition konkreter Erfolgskriterien für Security-Benchmarks (z. B. Fuzz-Targets) in späteren Iterationen.

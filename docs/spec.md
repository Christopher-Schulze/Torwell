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
- Desktop-Zielplattformen: macOS 13+, Windows 11, Ubuntu 22.04+ (Wayland/X11).
- GPU-beschleunigte Blur-Filter sind verfügbar; bei `prefers-reduced-motion` wird Animation reduziert.
- Rust 1.77+, Node.js 20+/Bun 1.1+ sowie `cargo-tarpaulin`, `cargo-fuzz` und `@playwright/test` stehen in der Toolchain bereit.
- Tor-Netzwerkzugriff ist möglich, Firewalls erlauben ausgehende Verbindungen auf Standard-Tor-Ports.
- Diagnose-Ansichten (`ConnectionDiagnostics`, `NetworkTools`) teilen sich Motion-Tokens und Timeline-Komponenten mit dem Dashboard.
- Benchmark-Läufe dürfen die Tor-Bootstrap-Infrastruktur maximal 3 parallele Sessions starten (koordiniert via `task desktop:bootstrap`).

## Schnittstellen
- Frontend ↔ Backend via Tauri `invoke` / `listen` Events (`tor-status-update`, `metrics-update`).
- Dokumentations-Hub `docs/DOCUMENTATION.md` verlinkt auf Spezifikation, Roadmap und Todos.
- Tests laufen via `npm run lint`, `npm run test:unit`, `npm run test:ui-snapshots` und `cargo test` im Ordner `src-tauri`.
- Coverage wird über `scripts/tests/coverage.sh` generiert und in `docs/todo/CR-0002.md` gespiegelt.
- Benchmarks laufen via `scripts/benchmarks/run_all.sh`; Fuzzing via `scripts/tests/fuzz.sh`.

- **Performance:** UI-Animationen <16ms Framebudget, keine Layout-Jumps >8px; Bootstrap-Benchmark p95 < 45s laut `connection_startup.sh`.
- **Resilienz:** Verbindungs-UI führt max. 1 parallelen Connect/Disconnect-Workflow; API-Retries mit exponentiellem Backoff bis 3 Versuche.
- **DX:** Saubere Typen, modulare Komponenten, `torStore` ohne Event-Leaks, dedizierte Utilitys für Motion/Theme.
- **Sicherheit:** Konsistente Fehlerpfade, strukturierte Logs, keine unvalidierten Eingaben Richtung TorManager.
- **Barrierefreiheit:** WCAG 2.1 AA Farbkontrast, ARIA-Live Regionen für Statuswechsel, respektiert Reduced-Motion; Timeline-Overlays bieten Tastatur- und Screenreader-Zugriff.

## SLAs & Telemetrie
- Bootstrap-Fortschritt aktualisiert mindestens alle 250ms während Connect.
- Metriken-Burst begrenzt auf 720 Punkte (~2h Historie bei 10s Intervall).
- Fehler werden innerhalb von 500ms als Toast und im Statuspanel angezeigt.
- Benchmark-Ausführung protokolliert p50/p95/p99 in `.benchmarks/bootstrap_summary.txt` und archiviert Rohdaten.

## Migration / Rollout
- Bestehende Workflows (`bun run check`, `bun run test`, `cargo test`) bleiben unverändert nutzbar.
- Neue Skripte werden in CI-Pipeline integriert, sobald Hooks in `docs/plan.md` umgesetzt sind (Follow-up mit konkreten CI-Konfigurationen).

## Offene Fragen
- 3D-Hardwarebeschleunigung auf Low-End-Geräten: Monitoring via Telemetrie TBD.
- Evaluieren, ob zusätzliche Mobile-spezifische Benchmarks erforderlich sind (abhängig von Capacitor-Follow-up).

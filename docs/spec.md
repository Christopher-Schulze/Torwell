# Spec (Zielzustand)

## Scope
- Modernisiere die Torwell84-Desktopoberfläche mit einer hochwertigen Glassmorphism-Ästhetik, fein abgestimmten Animationen und responsiven Layouts.
- Stärke die Resilienz der Verbindungssteuerung (Frontend + API-Layer), damit wiederholte Verbindungsversuche, Token-Invalidierungen und temporäre Netzwerkfehler ohne Nutzerinteraktion abgefangen werden.
- Sicherstelle, dass das Arti-basierte Backend (TorManager) deterministisch konfiguriert wird, Brücken-/Länderkonfigurationen respektiert und mit aussagekräftigen Fehlern reagiert.
- Dokumentiere Architektur, Annahmen, Workstreams und Backlog zentral in `/docs` gemäss Projektleitfaden.
- Statte das Projekt mit integrierten Quality-Gates aus: Lint (ESLint, Clippy), Unit-/Integrations-Tests, UI-Snapshots, Coverage-Reports, Benchmarks (Criterion, Playwright) und Fuzzing für alle sicherheitskritischen/Low-Level-Module.

## Nicht-Ziele
- Keine grundlegende Änderung des Tauri-Build- oder Deployment-Prozesses.
- Kein Austausch des arti-Clients oder Migration auf eine andere Tor-Implementierung.
- Keine tiefgreifenden Änderungen an Mobile-/Capacitor-spezifischen Workflows.

## Annahmen
- Desktop-Zielplattformen: macOS 13+, Windows 11, Ubuntu 22.04+ (Wayland/X11).
- GPU-beschleunigte Blur-Filter sind verfügbar; bei `prefers-reduced-motion` wird Animation reduziert.
- Rust 1.77+, Node.js 20+/Bun 1.1+ sowie `cargo-tarpaulin`, `cargo-fuzz` und `@playwright/test` stehen in der Toolchain bereit.
- Tor-Netzwerkzugriff ist möglich, Firewalls erlauben ausgehende Verbindungen auf Standard-Tor-Ports.
- Playwright-Benchmarks dürfen lokal einen Dev-Server (Vite) starten; Ports >=4173 sind frei.

## Schnittstellen
- Frontend ↔ Backend via Tauri `invoke` / `listen` Events (`tor-status-update`, `metrics-update`).
- Dokumentations-Hub `docs/DOCUMENTATION.md` verlinkt auf Spezifikation, Roadmap und Todos.
- Tests laufen via `npm run lint`, `npm run test:unit`, `npm run test:ui-snapshots` und `cargo test` im Ordner `src-tauri`.
- Coverage wird über `scripts/tests/coverage.sh` generiert und in `docs/todo/CR-0002.md` gespiegelt.
- Benchmarks laufen via `scripts/benchmarks/run_all.sh`; Fuzzing via `scripts/tests/fuzz.sh`.

## Qualitätsziele
- **Performance:** UI-Animationen <16ms Framebudget, keine Layout-Jumps >8px. Benchmarks liefern Baselines für Bootstrap (Rust) und First Contentful Paint (UI); Regressionen >10 % schlagen fehl.
- **Resilienz:** Verbindungs-UI führt max. 1 parallelen Connect/Disconnect-Workflow; API-Retries mit exponentiellem Backoff bis 3 Versuche. Playwright-Perf-Suite überwacht Fehlerfreiheit beim automatischen Bootstrapping.
- **DX:** Saubere Typen, modulare Komponenten, `torStore` ohne Event-Leaks, dedizierte Utilitys für Motion/Theme. Tests/Skripte laufen über `scripts/tests/run_all.sh` und sind CI-fähig.
- **Sicherheit:** Konsistente Fehlerpfade, strukturierte Logs, keine unvalidierten Eingaben Richtung TorManager. Fuzz-Targets decken Parser-/Policy-Funktionen ab und laufen nightly.
- **Barrierefreiheit:** WCAG 2.1 AA Farbkontrast, ARIA-Live Regionen für Statuswechsel, respektiert Reduced-Motion.

## SLAs & Telemetrie
- Bootstrap-Fortschritt aktualisiert mindestens alle 250ms während Connect.
- Metriken-Burst begrenzt auf 720 Punkte (~2h Historie bei 10s Intervall).
- Fehler werden innerhalb von 500ms als Toast und im Statuspanel angezeigt.

## Migration / Rollout
- Feature-Flag `experimental-api` bleibt optional; UI degradert ohne zusätzliche Datenquellen.
- Keine Datenbankmigrationen notwendig. Clientseitige Einstellungen bleiben kompatibel.

## Offene Fragen
- 3D-Hardwarebeschleunigung auf Low-End-Geräten: Monitoring via Telemetrie TBD.
- Integration der neuen Coverage-/Benchmark-Skripte in CI (GitHub Actions vs. selbstgehostet) muss abgestimmt werden.

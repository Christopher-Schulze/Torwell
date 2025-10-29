# Spec (Zielzustand)

## Scope
- Modernisiere die Torwell84-Desktopoberfläche mit einer hochwertigen Glassmorphism-Ästhetik, fein abgestimmten Animationen und responsiven Layouts.
- Stärke die Resilienz der Verbindungssteuerung (Frontend + API-Layer), damit wiederholte Verbindungsversuche, Token-Invalidierungen und temporäre Netzwerkfehler ohne Nutzerinteraktion abgefangen werden.
- Sicherstelle, dass das Arti-basierte Backend (TorManager) deterministisch konfiguriert wird, Brücken-/Länderkonfigurationen respektiert und mit aussagekräftigen Fehlern reagiert.
- Dokumentiere Architektur, Annahmen, Workstreams und Backlog zentral in `/docs` gemäss Projektleitfaden.
- Liefere reproduzierbare Developer-Workflows inklusive Devcontainer, Task-Runners und Git-Hooks.

## Nicht-Ziele
- Keine grundlegende Änderung des Tauri-Build- oder Deployment-Prozesses.
- Kein Austausch des arti-Clients oder Migration auf eine andere Tor-Implementierung.
- Keine tiefgreifenden Änderungen an Mobile-/Capacitor-spezifischen Workflows.

## Annahmen
- Desktop-Zielplattformen: macOS 13+, Windows 11, Ubuntu 22.04+ (Wayland/X11).
- GPU-Beschleunigte Blur-Filter sind verfügbar; bei `prefers-reduced-motion` wird Animation reduziert.
- Rust 1.77+, Node.js 20+/Bun 1.1+ sowie `go-task` (oder `@go-task/cli`) stehen bereit.
- Tor-Netzwerkzugriff ist möglich, Firewalls erlauben ausgehende Verbindungen auf Standard-Tor-Ports.

## Schnittstellen
- Frontend ↔ Backend via Tauri `invoke` / `listen` Events (`tor-status-update`, `metrics-update`).
- Dokumentations-Hub `docs/DOCUMENTATION.md` verlinkt auf Spezifikation, Roadmap und Todos.
- Tests laufen via `task lint`, `task test`, `task bench` (Wraps `bun`/`cargo`).
- Devcontainer initialisiert Toolchain (Node, Rust, Bun, Neovim) und ruft `scripts/utils/bootstrap.sh` nach Provisionierung auf.
- Git-Hooks verwenden `scripts/utils/run_task.sh`, um lokale Task-Runner zu abstrahieren.

## Qualitätsziele
- **Performance:** UI-Animationen <16ms Framebudget, keine Layout-Jumps >8px.
- **Resilienz:** Verbindungs-UI führt max. 1 parallelen Connect/Disconnect-Workflow; API-Retries mit exponentiellem Backoff bis 3 Versuche.
- **DX:** Saubere Typen, modulare Komponenten, `torStore` ohne Event-Leaks, dedizierte Utilitys für Motion/Theme.
- **DX Tooling:** One-command Bootstrap (`task setup`), vorgefertigte `.env.example`, reproduzierbare Pre-Commit Hooks.
- **Sicherheit:** Konsistente Fehlerpfade, strukturierte Logs, keine unvalidierten Eingaben Richtung TorManager.
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
- CI Container sollen Devcontainer-Setup spiegeln; Evaluierung eines veröffentlichten Base-Images steht aus.

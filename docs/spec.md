# Spec (Zielzustand)

## Scope
- Modernisiere die Torwell84-Desktopoberfläche mit einer hochwertigen Glassmorphism-Ästhetik, fein abgestimmten Animationen und responsiven Layouts.
- Stärke die Resilienz der Verbindungssteuerung (Frontend + API-Layer), damit wiederholte Verbindungsversuche, Token-Invalidierungen und temporäre Netzwerkfehler ohne Nutzerinteraktion abgefangen werden.
- Sicherstelle, dass das Arti-basierte Backend (TorManager) deterministisch konfiguriert wird, Brücken-/Länderkonfigurationen respektiert und mit aussagekräftigen Fehlern reagiert.
- Etabliere einen zentralen Work-Stealing-Scheduler für CPU-/I/O-lastige Backendantasks (Traceroute, Zertifikats-Rotation, Batch-Analysen) inklusive Telemetrie-Schnittstellen.
- Dokumentiere Architektur, Annahmen, Workstreams und Backlog zentral in `/docs` gemäss Projektleitfaden.

## Nicht-Ziele
- Keine grundlegende Änderung des Tauri-Build- oder Deployment-Prozesses.
- Kein Austausch des arti-Clients oder Migration auf eine andere Tor-Implementierung.
- Keine tiefgreifenden Änderungen an Mobile-/Capacitor-spezifischen Workflows.

## Annahmen
- Desktop-Zielplattformen: macOS 13+, Windows 11, Ubuntu 22.04+ (Wayland/X11).
- GPU-Beschleunigte Blur-Filter sind verfügbar; bei `prefers-reduced-motion` wird Animation reduziert.
- Rust 1.77+, Node.js 20+/Bun 1.1+ sind installiert.
- Tor-Netzwerkzugriff ist möglich, Firewalls erlauben ausgehende Verbindungen auf Standard-Tor-Ports.

## Schnittstellen
- Frontend ↔ Backend via Tauri `invoke` / `listen` Events (`tor-status-update`, `metrics-update`).
- Dokumentations-Hub `docs/DOCUMENTATION.md` verlinkt auf Spezifikation, Roadmap und Todos.
- Tests laufen via `bun run lint`, `bun run check`, `cargo test` im Ordner `src-tauri`.

## Qualitätsziele
- **Performance:** UI-Animationen <16ms Framebudget, keine Layout-Jumps >8px; Scheduler-Latenzen p99 < 25 ms und Queue-Backlog < 64 Tasks im Regelbetrieb.
- **Resilienz:** Verbindungs-UI führt max. 1 parallelen Connect/Disconnect-Workflow; API-Retries mit exponentiellem Backoff bis 3 Versuche.
- **DX:** Saubere Typen, modulare Komponenten, `torStore` ohne Event-Leaks, dedizierte Utilitys für Motion/Theme.
- **Sicherheit:** Konsistente Fehlerpfade, strukturierte Logs, keine unvalidierten Eingaben Richtung TorManager; Scheduler-Tasks laufen isoliert und berichten Panics/Miri-Funde sofort.
- **Barrierefreiheit:** WCAG 2.1 AA Farbkontrast, ARIA-Live Regionen für Statuswechsel, respektiert Reduced-Motion.

## SLAs & Telemetrie
- Bootstrap-Fortschritt aktualisiert mindestens alle 250ms während Connect.
- Metriken-Burst begrenzt auf 720 Punkte (~2h Historie bei 10s Intervall).
- Scheduler-Metriken (p50/p95/p99, Queue-Depth) werden im gleichen Intervall persistiert und via UI abrufbar gemacht.
- Fehler werden innerhalb von 500ms als Toast und im Statuspanel angezeigt.

## Migration / Rollout
- Feature-Flag `experimental-api` bleibt optional; UI degradert ohne zusätzliche Datenquellen.
- Keine Datenbankmigrationen notwendig. Clientseitige Einstellungen bleiben kompatibel; bestehende Metrikdateien werden dank `serde(default)` mit neuen Scheduler-Feldern automatisch erweitert.

## Offene Fragen
- 3D-Hardwarebeschleunigung auf Low-End-Geräten: Monitoring via Telemetrie TBD.
- Tuning der Worker-Anzahl (auto vs. fixed) sowie zusätzliche SIMD-Pfade für Bulk-Analyse TBD.

# Spec (Zielzustand)

## Scope
- Modernisiere die Torwell84-Desktopoberfläche mit einer hochwertigen Glassmorphism-Ästhetik, fein abgestimmten Animationen und responsiven Layouts.
- Stärke die Resilienz der Verbindungssteuerung (Frontend + API-Layer), damit wiederholte Verbindungsversuche, Token-Invalidierungen und temporäre Netzwerkfehler ohne Nutzerinteraktion abgefangen werden.
- Sicherstelle, dass das Arti-basierte Backend (TorManager) deterministisch konfiguriert wird, Brücken-/Länderkonfigurationen respektiert und mit aussagekräftigen Fehlern reagiert.
- Dokumentiere Architektur, Annahmen, Workstreams und Backlog zentral in `/docs` gemäss Projektleitfaden.
- Beschleunige Hot-Path-Pixel- und Metrics-Transformationen via modulare SIMD-Pipeline (`src-tauri/src/lib/simd`) inklusive Laufzeit-Feature-Erkennung, Fallback-Pfade und Benchmarks.

## Nicht-Ziele
- Keine grundlegende Änderung des Tauri-Build- oder Deployment-Prozesses.
- Kein Austausch des arti-Clients oder Migration auf eine andere Tor-Implementierung.
- Keine tiefgreifenden Änderungen an Mobile-/Capacitor-spezifischen Workflows.

## Annahmen
- Desktop-Zielplattformen: macOS 13+, Windows 11, Ubuntu 22.04+ (Wayland/X11).
- GPU-Beschleunigte Blur-Filter sind verfügbar; bei `prefers-reduced-motion` wird Animation reduziert.
- Rust 1.77+, Node.js 20+/Bun 1.1+ sind installiert.
- Tor-Netzwerkzugriff ist möglich, Firewalls erlauben ausgehende Verbindungen auf Standard-Tor-Ports.
- Bilddaten für SIMD-Routinen werden als RGBA32F (interleaved, normiert auf `[0.0, 1.0]`) bereitgestellt; Konvertierungsroutinen folgen.

## Schnittstellen
- Frontend ↔ Backend via Tauri `invoke` / `listen` Events (`tor-status-update`, `metrics-update`).
- Dokumentations-Hub `docs/DOCUMENTATION.md` verlinkt auf Spezifikation, Roadmap und Todos.
- Tests laufen via `bun run lint`, `bun run check`, `cargo test` im Ordner `src-tauri`.

- **Performance:** UI-Animationen <16ms Framebudget, keine Layout-Jumps >8px, SIMD-Filter erzielen ≥3x Speedup gegenüber Scalar-Baseline (gemessen via Criterion).
- **Resilienz:** Verbindungs-UI führt max. 1 parallelen Connect/Disconnect-Workflow; API-Retries mit exponentiellem Backoff bis 3 Versuche.
- **DX:** Saubere Typen, modulare Komponenten, `torStore` ohne Event-Leaks, dedizierte Utilitys für Motion/Theme, dokumentierte SIMD-Dispatch-Layer.
- **Sicherheit:** Konsistente Fehlerpfade, strukturierte Logs, keine unvalidierten Eingaben Richtung TorManager.
- **Barrierefreiheit:** WCAG 2.1 AA Farbkontrast, ARIA-Live Regionen für Statuswechsel, respektiert Reduced-Motion.

## SLAs & Telemetrie
- Bootstrap-Fortschritt aktualisiert mindestens alle 250ms während Connect.
- Metriken-Burst begrenzt auf 720 Punkte (~2h Historie bei 10s Intervall).
- Fehler werden innerhalb von 500ms als Toast und im Statuspanel angezeigt.
- SIMD-Pipeline liefert Backend-Telemetrie (`backend`, Speedup-Faktor) an Benchmark-Reports in `/docs/todo/`.

## Migration / Rollout
- Feature-Flag `experimental-api` bleibt optional; UI degradert ohne zusätzliche Datenquellen.
- Keine Datenbankmigrationen notwendig. Clientseitige Einstellungen bleiben kompatibel.

## Offene Fragen
- 3D-Hardwarebeschleunigung auf Low-End-Geräten: Monitoring via Telemetrie TBD.
- Ob dedizierte Benchmark-Skripte nötig sind, hängt von weiteren Performance-Arbeitsaufträgen ab.

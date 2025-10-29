# Spec (Zielzustand)

## Scope
- Modernisiere die Torwell84-Desktopoberfläche mit einer hochwertigen Glassmorphism-Ästhetik, fein abgestimmten Animationen und responsiven Layouts.
- Stärke die Resilienz der Verbindungssteuerung (Frontend + API-Layer), damit wiederholte Verbindungsversuche, Token-Invalidierungen und temporäre Netzwerkfehler ohne Nutzerinteraktion abgefangen werden.
- Sicherstelle, dass das Arti-basierte Backend (TorManager) deterministisch konfiguriert wird, Brücken-/Länderkonfigurationen respektiert und mit aussagekräftigen Fehlern reagiert.
- Führe ein GPU-basiertes Render-Backend auf Basis von `wgpu` ein, das Metal (macOS) explizit aktiviert, auf dedizierten Worker-Threads läuft, Triple-Buffering inkl. Fence-Handling nutzt und fortlaufende Frame-Metriken erzeugt.
- Implementiere einen Hash-basierten Shader-Cache unterhalb des System-App-Verzeichnisses (z. B. `~/Library/Application Support/Torwell/shader_cache`) inklusive Warmup bei App-Start.
- Ergänze headless Integrations- sowie Screenshot-Tests, die den Renderpfad prüfen und im CI automatisiert über `/scripts/tests/` ausgeführt werden.
- Dokumentiere Architektur, Annahmen, Workstreams und Backlog zentral in `/docs` gemäss Projektleitfaden.

## Nicht-Ziele
- Keine grundlegende Änderung des Tauri-Build- oder Deployment-Prozesses.
- Kein Austausch des arti-Clients oder Migration auf eine andere Tor-Implementierung.
- Keine tiefgreifenden Änderungen an Mobile-/Capacitor-spezifischen Workflows.

## Annahmen
- Desktop-Zielplattformen: macOS 13+, Windows 11, Ubuntu 22.04+ (Wayland/X11).
- GPU-Beschleunigte Blur-Filter sind verfügbar; bei `prefers-reduced-motion` wird Animation reduziert.
- Für den Renderer steht mindestens ein durch `wgpu` adressierbares Backend (Metal/Vulkan/DX12, notfalls Fallback-Adapter) zur Verfügung. Falls kein Adapter gefunden wird, degradiert der Render-Service sauber und meldet dies via Metriken.
- `wgpu` wird mit nativen Backend-Features (Metal/Vulkan/DX12) über das Standard-Feature-Set gebaut; macOS-Ziele nutzen automatisch Metal.
- Shader-Cache-Pfad kann über `TORWELL_SHADER_CACHE_DIR` überschrieben werden (Tests, Sandbox).
- Rust 1.77+, Node.js 20+/Bun 1.1+ sind installiert.
- Tor-Netzwerkzugriff ist möglich, Firewalls erlauben ausgehende Verbindungen auf Standard-Tor-Ports.

## Schnittstellen
- Frontend ↔ Backend via Tauri `invoke` / `listen` Events (`tor-status-update`, `metrics-update`, `frame-metrics`).
- Neuer Tauri-Command `get_frame_metrics` liefert Momentaufnahme + Percentiles der Renderloop-Kennzahlen.
- Headless-Screenshot-Tool `renderer_capture` (Cargo-Binary) erzeugt Referenzframes für Skripte in `/scripts/tests/`.
- Dokumentations-Hub `docs/DOCUMENTATION.md` verlinkt auf Spezifikation, Roadmap und Todos.
- Tests laufen via `bun run lint`, `bun run check`, `cargo test` im Ordner `src-tauri` sowie den GPU-Skripten in `/scripts/tests/`.

- **Performance:** UI-Animationen <16 ms Framebudget, keine Layout-Jumps >8 px. GPU-Renderloop mit <2 ms CPU-Encode-P50, <4 ms GPU-P50 und Frame-Interval-P95 ≤18 ms.
- **Resilienz:** Verbindungs-UI führt max. 1 parallelen Connect/Disconnect-Workflow; API-Retries mit exponentiellem Backoff bis 3 Versuche. Render-Service überlebt Adapter-Verlust durch automatisches Degradieren.
- **DX:** Saubere Typen, modulare Komponenten, `torStore` ohne Event-Leaks, dedizierte Utilitys für Motion/Theme, Skripte für GPU/Screenshot-Tests.
- **Sicherheit:** Konsistente Fehlerpfade, strukturierte Logs, keine unvalidierten Eingaben Richtung TorManager. GPU-spezifische Fehler laufen über `Error::Gpu`.
- **Barrierefreiheit:** WCAG 2.1 AA Farbkontrast, ARIA-Live Regionen für Statuswechsel, respektiert Reduced-Motion.

## SLAs & Telemetrie
- Bootstrap-Fortschritt aktualisiert mindestens alle 250 ms während Connect.
- Metriken-Burst begrenzt auf 720 Punkte (~2 h Historie bei 10 s Intervall).
- Fehler werden innerhalb von 500 ms als Toast und im Statuspanel angezeigt.
- Frame-Metriken werden mindestens alle 30 s aggregiert und via `metrics-update` + `frame-metrics` Event bereitgestellt; `get_frame_metrics` liefert History (max. 120 Frames) und Percentiles.

## Migration / Rollout
- Feature-Flag `experimental-api` bleibt optional; UI degradert ohne zusätzliche Datenquellen.
- Keine Datenbankmigrationen notwendig. Clientseitige Einstellungen bleiben kompatibel.

## Offene Fragen
- 3D-Hardwarebeschleunigung auf Low-End-Geräten: Monitoring via Telemetrie TBD.
- GPU-Benchmark-Skripte (synthetische Last, Frame-Histogramm) folgen nach Evaluierung der Produktionsdaten.
- Validierung zusätzlicher Shader (Postprocessing) sobald neue Visual-Features spezifiziert sind.

# Spec (Zielzustand)

## Scope
- Modernisiere die Torwell84-Desktopoberfläche mit einer hochwertigen Glassmorphism-Ästhetik, fein abgestimmten Animationen und responsiven Layouts.
- Stärke die Resilienz der Verbindungssteuerung (Frontend + API-Layer), damit wiederholte Verbindungsversuche, Token-Invalidierungen und temporäre Netzwerkfehler ohne Nutzerinteraktion abgefangen werden.
- Sicherstelle, dass das Arti-basierte Backend (TorManager) deterministisch konfiguriert wird, Brücken-/Länderkonfigurationen respektiert und mit aussagekräftigen Fehlern reagiert.
- Dokumentiere Architektur, Annahmen, Workstreams und Backlog zentral in `/docs` gemäss Projektleitfaden.
- Ergänze eine konfigurierbare Cache-Schicht (`src/cache`) mit LRU/LFU-Eviction, Warmup aus Persistenz sowie API-Integration für Timeline-, Summary- und Geo-Lookups.
- Härte die Speicherverwaltung durch einen Hot-Path-Allocator (`mimalloc`) im Rust-Backend aus und stelle eine SoA-basierte Metrikaufbereitung für Trendanalysen bereit.
- Liefere reproduzierbare Memory-Profiling-Workflows (Valgrind Massif & Heaptrack) inklusive Skripten, Dokumentation und Auswertungsrichtlinien.

## Nicht-Ziele
- Keine grundlegende Änderung des Tauri-Build- oder Deployment-Prozesses.
- Kein Austausch des arti-Clients oder Migration auf eine andere Tor-Implementierung.
- Keine tiefgreifenden Änderungen an Mobile-/Capacitor-spezifischen Workflows.

## Annahmen
- Desktop-Zielplattformen: macOS 13+, Windows 11, Ubuntu 22.04+ (Wayland/X11).
- GPU-Beschleunigte Blur-Filter sind verfügbar; bei `prefers-reduced-motion` wird Animation reduziert.
- Rust 1.77+, Node.js 20+/Bun 1.1+ sind installiert.
- Tor-Netzwerkzugriff ist möglich, Firewalls erlauben ausgehende Verbindungen auf Standard-Tor-Ports.
- Browser-Laufzeit stellt `localStorage` für Cache-Hydration bereit; im Headless-Testbetrieb werden Polyfills eingesetzt.
- Profiling-Tools (`valgrind`, `heaptrack`) sind optional installiert und dürfen in CI via Feature-Flag aktiviert werden.

## Schnittstellen
- Frontend ↔ Backend via Tauri `invoke` / `listen` Events (`tor-status-update`, `metrics-update`).
- Dokumentations-Hub `docs/DOCUMENTATION.md` verlinkt auf Spezifikation, Roadmap und Todos.
- Tests laufen via `bun run lint`, `bun run check`, `cargo test` im Ordner `src-tauri`.
- Cache-Layer exponiert `connectionTimelineCache`, `connectionSummaryCache`, `countryLookupCache` sowie Warmup/Invalidierungs-Hooks (`warmupCaches`, `invalidateConnectionCaches`).
- Memory-Profiling via `scripts/benchmarks/run_massif.sh` und `scripts/benchmarks/run_heaptrack.sh`, Ergebnisse landen unter `src-tauri/target/memory-profiles`.

## Qualitätsziele
- **Performance:** UI-Animationen <16ms Framebudget, keine Layout-Jumps >8px.
- **Resilienz:** Verbindungs-UI führt max. 1 parallelen Connect/Disconnect-Workflow; API-Retries mit exponentiellem Backoff bis 3 Versuche; Cache-Layer garantiert <5ms Hit-Latenz und <35ms Miss-Latenz (95th Percentile) bei 100 gleichzeitigen Requests.
- **DX:** Saubere Typen, modulare Komponenten, `torStore` ohne Event-Leaks, dedizierte Utilitys für Motion/Theme.
- **Sicherheit:** Konsistente Fehlerpfade, strukturierte Logs, keine unvalidierten Eingaben Richtung TorManager; Cache-Snapshots sind signalfrei und enthalten keine Secrets.
- **Barrierefreiheit:** WCAG 2.1 AA Farbkontrast, ARIA-Live Regionen für Statuswechsel, respektiert Reduced-Motion.
- **Speichereffizienz:** Backend-Allektionen laufen über `mimalloc`; Metrik-Trendberechnungen verwenden SoA-Datenpfad mit <5% Overhead ggü. nativem Array-of-Structs.

## SLAs & Telemetrie
- Bootstrap-Fortschritt aktualisiert mindestens alle 250ms während Connect.
- Metriken-Burst begrenzt auf 720 Punkte (~2h Historie bei 10s Intervall).
- Fehler werden innerhalb von 500ms als Toast und im Statuspanel angezeigt.
- Cache-Hitrate >80% für Timeline/Summary bei stabiler Verbindung; Persistenz-Snapshots werden alle 30s aktualisiert.
- Memory-Profiling-Durchlauf <12min auf CI-Runner (8 vCPU, 16GB RAM); Berichte bleiben <50MB pro Lauf.

## Migration / Rollout
- Feature-Flag `experimental-api` bleibt optional; UI degradert ohne zusätzliche Datenquellen.
- Keine Datenbankmigrationen notwendig. Clientseitige Einstellungen bleiben kompatibel.

## Offene Fragen
- 3D-Hardwarebeschleunigung auf Low-End-Geräten: Monitoring via Telemetrie TBD.
- Ob dedizierte Benchmark-Skripte nötig sind, hängt von weiteren Performance-Arbeitsaufträgen ab.
- Automatisierte Auswertung der Memory-Profile (Massif/Heaptrack) in CI – folgt nach MVP-Bewertung.

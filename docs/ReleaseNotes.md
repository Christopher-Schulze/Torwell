# Release Notes – Torwell84 v2.5

## Highlights
- Glas-UI mit responsivem Dashboard, mikroanimierten Statuskarten und reduzierter Motion-Unterstützung.
- Resiliente Connect/Disconnect-Pipeline mit Queueing, Exponential-Backoff und Listener-Lifecycle-Guards.
- Bootstrap-Benchmark `scripts/benchmarks/connection_startup.sh` inklusive p50/p95/p99-Auswertung.
- Dokumentations-Hub konsolidiert (`DOCUMENTATION.md`, `plan.md`, `spec.md`, aktualisierte Backlog-Übersicht).

## Fixes & Verbesserungen
- Idempotente `TorManager::connect`-Aufrufe verhindern wiederholte Bootstrap-Versuche.
- Einheitliche Fehlerbehandlung im Frontend: Toasts + Statuspanel binnen 500ms.
- Motion-Tokens zentralisiert und für Diagnostics-Views vorbereitet.

## Breaking Changes
- Keine.

## Upgrade Notes
- `task desktop:bootstrap` wird für Benchmarks vorausgesetzt; Tor-Netzwerkzugang wird benötigt.
- Für Rust-Tests ist `glib-2.0` weiterhin Systemvoraussetzung.

## Known Issues
- Diagnostics-Views folgen im Milestone D und nutzen vorübergehend das alte Styling.
- GPU-Blur auf älteren Intel-GPUs kann deaktiviert werden (`prefers-reduced-motion`).

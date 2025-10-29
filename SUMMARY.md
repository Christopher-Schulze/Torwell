## Änderungen
- Premiumisierte Dashboard-Komponenten (`StatusCard`, `ActionCard`, `IdlePanel`) mit neuen Glasgradients, Mikroanimationen und responsiven Layouts.
- Erweiterte Resilienz im Frontend (`torStore`, `api`-Wrapper) inkl. Retry-Backoff, listener cleanup und reduzierter Motion-Utilities.
- Aktualisierte Dokumentation gemäss Organisationsrichtlinie (Spec, Plan, File & Wire Map, TODO-Backlog).
- Neue Motion-Utilities und angepasste Metrik-Auswertungen (Rolling Latency, Trendberechnung, Tests für TorManager).

## Kommandos
- Tests (Frontend): `bun run check`
- Tests (Rust): `cargo test` (scheitert ohne systemweite glib-2.0 Bibliotheken)

## Nächste Schritte
- Follow-up CR-0001 zur Modernisierung der Diagnostics- und Network-Ansichten umsetzen.
- glib-2.0 Bereitstellung in CI/Build-Umgebung sicherstellen, damit `cargo test` überall läuft.

## Annahmen
- Reduced-Motion Nutzer*innen sollen Animationen deaktivieren; neue Motion-Store respektiert dies.
- Latency-Metriken können temporär fehlen und werden konservativ mit 0 in Trends berücksichtigt.

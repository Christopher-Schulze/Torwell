## Änderungen
- Change-Request-Blätter konsolidiert und in `DOCUMENTATION.md`, `plan.md`, `spec.md` sowie `ReleaseNotes.md` final verankert.
- `docs/FILEANDWIREMAP.md` aktualisiert (Module + Benchmark-Skripte) und `docs/archive/CR-0001.md` angelegt.
- Bootstrap-Benchmarkskript `scripts/benchmarks/connection_startup.sh` erstellt (p50/p95/p99-Auswertung via Python).
- Release Notes für v2.5 ergänzt; Plan- und Spec-Dokumente um Benchmarks, Testmatrix und Motion-Anforderungen erweitert.

## Kommandos
- Tests (Rust): `cargo test` *(scheitert ohne systemweite glib-2.0 Bibliothek)*
- Concurrency-Harness: `scripts/tests/run_concurrency.sh`
- Tests (Frontend): `bun run check`
- Tests (Rust): `cargo test` (erfordert systemweite `glib-2.0` Bibliotheken)
- Benchmarks: `scripts/benchmarks/connection_startup.sh`

## Nächste Schritte
- Milestone D vorbereiten (Diagnostics UX Refresh, Timeline-Komponenten, CI-Hooks).
- glib-2.0 Bereitstellung in CI/Build-Umgebung sicherstellen, damit `cargo test` überall läuft.
- CR-0002 adressieren (Postprocessing-Shader, Renderer-Fallbacks, GPU-Benchmarks).

## Annahmen
- Reduced-Motion Nutzer*innen sollen Animationen deaktivieren; neue Motion-Store respektiert dies.
- Latency-Metriken können temporär fehlen und werden konservativ mit 0 in Trends berücksichtigt.
- Benchmark-Ausführungen nutzen `task desktop:bootstrap` als verbindliche Bootstrap-Sequenz und halten <3 parallele Sessions.

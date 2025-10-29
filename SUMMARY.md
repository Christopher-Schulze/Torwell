## Änderungen
- Zentralen Work-Stealing-Scheduler (`core::executor::TaskScheduler`) eingeführt, lokale Batching-Strategie für Latenz-Histogramme (p50/p95/p99) implementiert und `AppState`/`MetricPoint` um Scheduler-Telemetrie erweitert.
- CPU-/I/O-intensive Pfade (`traceroute_host`, Zertifikatsrotation im `SecureHttpClient`) an den Scheduler angebunden und Tests/Dokumentation entsprechend aktualisiert.
- Neues Concurrency-Harness (`scripts/tests/run_concurrency.sh`) mit Loom-Model-Check & optionalem Miri-Lauf ergänzt; Spezifikation/Dokumentation um Scheduler-Ziele erweitert.

## Kommandos
- Tests (Rust): `cargo test` *(scheitert ohne systemweite glib-2.0 Bibliothek)*
- Concurrency-Harness: `scripts/tests/run_concurrency.sh`
- Tests (Frontend): `bun run check`

## Nächste Schritte
- Worker-Anzahl und weitere CPU-Pfade (z. B. GeoIP-Analysen) sukzessive an den Scheduler anbinden.
- glib-2.0 Bereitstellung in CI/Build-Umgebung sicherstellen, damit `cargo test` überall läuft.

## Annahmen
- Scheduler skaliert initial mit `num_cpus::get().max(2)` Threads; Feintuning erfolgt nach Telemetrieauswertung.
- Historische Metrikdateien werden via `serde(default)` auf neue Scheduler-Felder erweitert; keine Migration nötig.

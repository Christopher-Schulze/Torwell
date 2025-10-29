# Benchmarks

Dieser Ordner bündelt alle Performance-relevanten Skripte.

## Frontend Benchmarks
`scripts/benchmarks/run_frontend_benchmarks.sh` führt `bun x vitest bench` aus. Verwende zusätzliche Argumente, um Filter oder Reporter zu setzen:

```bash
scripts/benchmarks/run_frontend_benchmarks.sh -- --runInBand --reporter=default
```

Empfohlene Ausgaben: p50/p95/p99 pro Test. Ergebnisse können via `--outputFile` in JSON gespeichert und von CI als Artefakt gesichert werden.

## Erweiterungspunkte
- Ergänze `cargo bench` Wrapper für Rust-spezifische Hotpaths (TorManager, Circuit-Metriken).
- Integriere `hyperfine` für End-to-End Workflows (z. B. Startzeit der Tauri-App) und exportiere `benchmarks/*.json`.
- Dokumentiere neue Skripte in `docs/DOCUMENTATION.md` und verlinke dortige SLAs/Metriken.

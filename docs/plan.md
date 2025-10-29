# Plan / Roadmap

## Arbeitsprinzip
Dieses Dokument bündelt die aktuellen Arbeitspakete für das UI-/Resilienz-Upgrade. Pakete sind so geschnitten, dass sie parallelisiert werden können und minimale Konfliktflächen besitzen.

## Work Breakdown Structure (WBS)

| ID | Paket | Beschreibung | Impact | Konfliktrisiko |
|----|-------|--------------|--------|----------------|
| C1 | Cache-Fundament | Implementiert `AdaptiveCache`, Eviction-Strategien, Warmup-Konfiguration & Persistenz in `src/cache`. | Hoch | Mittel |
| C2 | API-Integration & Invalidierung | Verdrahtet Cache in `src/lib/api.ts`, invalidiert über `torStore`, stellt Warmup-Hooks bereit. | Hoch | Mittel |
| C3 | Metrik-SoA & Trendpfad | Stellt `metricSeries` bereit, optimiert `metrics.ts` auf Struct-of-Arrays, verbessert Trend-Berechnung. | Mittel | Niedrig |
| C4 | Hot-Path-Allocator | Bindet `mimalloc` als globalen Allocator ein, kontrolliert Memory-Footprint der Rust-Komponenten. | Mittel | Niedrig |
| C5 | Profiling Toolchain | Skripte für Massif/Heaptrack unter `scripts/benchmarks`, Artefakt-Pipeline & Dokumentation. | Mittel | Niedrig |
| C6 | Tests & Leak-Checks | Vitest-Suite für Cache-Hits/Misses, Persistenz-Warmup sowie API-Memoisierung; verifiziert Begrenzungen. | Hoch | Mittel |
| C7 | Observability & Limits (Follow-up) | Automatisierte Auswertung der Profiling-Berichte, Dashboards, Alerting. | Mittel | Hoch |
| C8 | CI Memory Gates (Follow-up) | Integration der Profiling-Skripte in CI, Threshold-basierte Abbrüche. | Mittel | Mittel |

## Priorisierte Auswahl
C1–C6 sind umgesetzt und dienen als Basis für Memory-Härtung. C7–C8 werden als zukünftige Erweiterungen dokumentiert.

## Meilensteine
1. **Milestone Ω – Cache & Analytics**: Abschluss C1–C3 (bereitgestellt).
2. **Milestone Σ – Runtime Hardening**: Abschluss C4–C6 inkl. Tests und Skripte.
3. **Milestone Φ – Observability Scale-Up**: Umsetzung der Follow-ups C7–C8.

## Risiken & Mitigation
- **Persistenz-Korruption**: Snapshot-Schreiboperationen sind guardiert und fehlertolerant; Warmup-Errors werden geloggt.
- **Profiler-Laufzeit**: Skripte erzwingen Vorbuild (`cargo test --no-run`) und laufen optional, um CI nicht zu blockieren.
- **Allocator-Kompatibilität**: `mimalloc` wird mit Standard-Konfiguration eingebunden; Smoke-Tests prüfen Startpfad.

## Nächste Schritte
- Follow-up-Auftrag für C7/C8 planen (automatisierte Profil-Analyse, CI-Einbindung).
- Optional: weitere Caches (Bridge-Liste, Zertifikate) evaluieren.

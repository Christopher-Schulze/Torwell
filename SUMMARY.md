## Änderungen
- Cache-Layer (`src/cache`) mit `AdaptiveCache`, Persistenz-Warmup und Invalidierungs-Hooks für Timeline-, Summary- und Geo-Daten.
- API-Integration: `src/lib/api.ts` nutzt Cache-Hits, persistiert Snapshots und startet Warmup automatisch; `torStore` invalidiert bei Statuswechsel.
- Metrikpipeline um Struct-of-Arrays (`metricSeries`) erweitert; Trendberechnung in `metrics.ts` arbeitet auf Typed-Arrays.
- Rust-Backend bindet `mimalloc` als globalen Allocator ein; neue Memory-Profiling-Skripte unter `scripts/benchmarks/` dokumentiert.
- Vitest-Suite ergänzt (`cacheLayer.spec.ts`, `apiCache.spec.ts`) für Cache-Hits/Misses, Warmup und Token-Reuse.
- Dokumentation (spec/plan/context/DOCUMENTATION.md/FILEANDWIREMAP.md) um Cache-, Allocator- und Profiling-Inhalte aktualisiert.

## Kommandos
- Tests (zielgerichtet): `npx vitest run src/__tests__/cacheLayer.spec.ts src/__tests__/apiCache.spec.ts`
- Memory-Profiling: `scripts/benchmarks/run_massif.sh`, `scripts/benchmarks/run_heaptrack.sh`
- Rust-Tests: `cargo test` (benötigt systemweites `glib-2.0` → in Container derzeit nicht verfügbar)

## Nächste Schritte
- Follow-up: Automatisierte Auswertung der Profiling-Artefakte (Plan C7/C8) und CI-Integration.
- Optionale Caches für weitere IPC-Endpunkte evaluieren (Bridge-Liste, Zertifikate).

## Annahmen
- Browser stellt `localStorage` bereit; Tests mocken `window.localStorage`.
- Profiling-Tools (Valgrind/Heaptrack) werden bei Bedarf lokal oder in CI installiert.

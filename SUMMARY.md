## Änderungen
- Premiumisierte Dashboard-Komponenten (`StatusCard`, `ActionCard`, `IdlePanel`) mit neuen Glasgradients, Mikroanimationen und responsiven Layouts.
- Erweiterte Resilienz im Frontend (`torStore`, `api`-Wrapper) inkl. Retry-Backoff, listener cleanup und reduzierter Motion-Utilities.
- Aktualisierte Dokumentation gemäss Organisationsrichtlinie (Spec, Plan, File & Wire Map, TODO-Backlog).
- Neue Motion-Utilities und angepasste Metrik-Auswertungen (Rolling Latency, Trendberechnung, Tests für TorManager).
- GPU-Render-Backend (wgpu) mit Worker-basiertem Renderloop, Triple-Buffering, Metal/Vulkan/DX12-Backends, Shader-Cache, Frame-Metriken sowie Headless-Screenshot-Tests & Capture-CLI-Warmup.

## Kommandos
- Tests (Frontend): `bun run check`
- Tests (Rust): `cargo test` (scheitert ohne systemweite glib-2.0 Bibliotheken)
- Headless-GPU-Test: `scripts/tests/headless_renderer.sh`

## Nächste Schritte
- Follow-up CR-0001 zur Modernisierung der Diagnostics- und Network-Ansichten umsetzen.
- glib-2.0 Bereitstellung in CI/Build-Umgebung sicherstellen, damit `cargo test` überall läuft.
- CR-0002 adressieren (Postprocessing-Shader, Renderer-Fallbacks, GPU-Benchmarks).

## Annahmen
- Reduced-Motion Nutzer*innen sollen Animationen deaktivieren; neue Motion-Store respektiert dies.
- Latency-Metriken können temporär fehlen und werden konservativ mit 0 in Trends berücksichtigt.
- Shader-Cache kann via `TORWELL_SHADER_CACHE_DIR` überschrieben werden; ohne kompatiblen Adapter liefern Tests einen Skip.

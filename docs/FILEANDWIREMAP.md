# File & Wire Map

```text
src/
  app.css                  # Globale Themes, Glas-Token, Motion-Keyframes
  lib/
    api.ts                 # Invoke-Wrapper mit Token/Retries
    stores/
      torStore.ts          # Status/Metriken, Event-Lifecycle
    components/
      StatusCard.svelte    # Hauptstatus mit Circuit/Route Visualisierung
      IdlePanel.svelte     # Bootstrap-Progress & Retry-State
      ActionCard.svelte    # Connect/Disconnect Controls
  routes/
    +page.svelte           # Dashboard Layout (Sections, Modals)

src-tauri/
  src/
    tor_manager.rs         # Arti Integration, Circuit Policies, Backoff
    commands.rs            # Tauri Commands & Rate-Limits
    renderer/
      mod.rs               # Re-exports für Cache/Metrics/Worker
      cache.rs             # Hash-basierter Shader-Cache (BLAKE3, Warmup)
      metrics.rs           # Frame-Metriken + Percentiles, Event-Emitter
      worker.rs            # Worker-Thread, Triple-Buffering, Fence-Handling
    bin/
      renderer_capture.rs  # Headless CLI für Frame-Captures & Hash-Check

scripts/
  tests/
    run_all.sh             # Orchestriert Rust/UI/GPU-Tests
    headless_renderer.sh   # Führt Renderer-Capture & Screenshot-Prüfung aus

docs/
  DOCUMENTATION.md         # Hub, Überblick
  spec.md                  # Zielzustand & SLAs
  plan.md                  # WBS & Priorisierung
  todo.md                  # Offene Arbeiten
  todo/CR-0001.md          # Detailnotizen (UI Diagnostics Follow-up)
  todo/CR-0002.md          # GPU Renderer Follow-ups (PostFX, Benchmarks)
```

```mermaid
flowchart LR
    subgraph Frontend [Svelte Frontend]
        A[Dashboard +page.svelte]
        B[StatusCard]
        C[IdlePanel]
        D[ActionCard]
    end
    subgraph Stores
        S1[torStore]
        S2[uiStore]
    end
    subgraph Backend [Tauri Backend]
        T1[commands.rs]
        T2[tor_manager.rs]
        T3[renderer::RendererService]
    end

    A --> B
    A --> C
    A --> D
    B --> S1
    C --> S1
    D --> S1
    S1 -- invoke/listen --> T1
    T1 --> T2
    T1 -->|get_frame_metrics| T3
    T3 -->|frame-metrics event| Frontend
    T2 -->|arti-client| Tor[(Tor Network)]
```

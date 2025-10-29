# File & Wire Map

```text
src/
  app.css                  # Globale Themes, Glas-Token, Motion-Keyframes
  cache/
    adaptiveCache.ts       # Konfigurierbarer Cache (LRU/LFU/FIFO, TTL, Warmup)
    index.ts               # Cache-Instanzen (Timeline, Summary, Geo), Persistenz
    metricSeries.ts        # Struct-of-Arrays für Metrikberechnungen
  lib/
    api.ts                 # Invoke-Wrapper mit Token/Retries + Cache-Hits
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
    lib.rs                 # Tauri-Setup, globaler mimalloc Allocator

scripts/
  benchmarks/
    run_massif.sh          # Valgrind Massif Runner (Memory Profiling)
    run_heaptrack.sh       # Heaptrack Runner (Allocation Tracing)

docs/
  DOCUMENTATION.md         # Hub, Überblick
  spec.md                  # Zielzustand & SLAs
  plan.md                  # WBS & Priorisierung
  todo.md                  # Offene Arbeiten & Backlog
  ReleaseNotes.md          # Versionshinweise v2.5
  archive/CR-0001.md       # Historisches CR-Blatt (Diagnostics Follow-up)

scripts/
  benchmarks/
    connection_startup.sh  # Bootstrap-Benchmark (p50/p95/p99)
  backup_ui.sh             # Sicherung der UI-Komponenten
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
    subgraph Cache
        C1[AdaptiveCache]
        C2[metricSeries]
    end
    subgraph Backend [Tauri Backend]
        T1[commands.rs]
        T2[tor_manager.rs]
    end

    A --> B
    A --> C
    A --> D
    B --> S1
    C --> S1
    D --> S1
    S1 --> C1
    C1 --> S1
    S1 --> C2
    C2 --> S1
    S1 -- invoke/listen --> T1
    A -->|cache-aware calls| C1
    C1 -->|persist/warmup| A
    T1 --> T2
    T2 -->|arti-client| Tor[(Tor Network)]
```

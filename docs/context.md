# Kontext

Torwell84 befindet sich in der Phase „UI/Resilience Refresh + GPU“. Die bisherige Implementierung verfügte über eine Glas-Optik, allerdings mit inkonsistenten Animationen, fehlendem Lifecycle-Management bei den Tauri-Events und ohne dedizierten GPU-Renderpfad. Dieser Auftrag liefert:

- Ein modernisiertes UI mit detaillierten Glasflächen, Mikroanimationen und responsiven Layouts.
- Verbesserte Resilienzschichten im Frontend (`invoke`-Wrapper, Stores) und zusätzliche Tests für den Arti-TorManager.
- Einen wgpu-basierten Renderer (Metal/Vulkan/DX12) mit Worker-Threads, Triple-Buffering, Shader-Cache und Frame-Metriken plus Headless-/Screenshot-Tests (`scripts/tests/headless_renderer.sh`).
- Einen aktualisierten Dokumentations-Hub mit Spezifikation, Roadmap und Backlog (`docs/todo`).

Stakeholder: Privacy Engineering (Lead), Desktop Team, QA Automation.

# Technical Detail: WebGPU Architecture & ZeroCopy (Todo-005)

## Fragen
1. **Zweck:** Wofür wird der WebGPU Renderer in Rust genutzt? Ist er für die UI (Anzeige im Fenster) oder nur für Hintergrund-Berechnungen/Tests?
2. **Integration:** Wenn für UI, wie kommen die Pixel ins Frontend?
3. **Performance:** Wird ZeroCopy genutzt oder werden Daten unnötig kopiert (GPU -> CPU -> JSON -> JS)?

## Analyse-Ergebnisse (Vorläufig)
- `renderer/worker.rs` nutzt `wgpu` headless (kein Surface).
- Es gibt `RenderTarget::Capture`, welches Pixel in einen Buffer kopiert und dann in einen `Vec<u8>` (`FrameCapture`).
- Es scheint keine direkte Anbindung an das Tauri-Fenster zu geben (kein `create_surface` mit `app_handle`).
- **Verdacht:** Das Rendering ist derzeit isoliert und nicht sichtbar, oder es fehlt der Transport-Layer zum Frontend.

## Ziele
- Klären, ob das so gewollt ist.
- Wenn Daten zum Frontend sollen: Binary Transport nutzen (nicht JSON Overhead).
- Wenn möglich: Shared Context oder direktes Rendering in Tauri Surface.

## Maßnahmen
1. Code in `secure_http` und `commands` prüfen, ob `FrameCapture` Daten versendet werden.

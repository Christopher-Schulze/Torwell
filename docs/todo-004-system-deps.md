# Technical Detail: System Dependencies (Todo-004)

## Problembeschreibung
Das Bauen der App auf Linux (Ubuntu 24.04) erfordert spezifische Systembibliotheken, die nicht offensichtlich sind. Insbesondere `webkit2gtk` und `javascriptcoregtk` Versionen verursachen Probleme.

## Gefundene Probleme
- `javascriptcore-rs-sys` (Rust Crate) sucht oft hardcodiert nach Version 4.0 (`javascriptcoregtk-4.0.pc`).
- Moderne Distros (Ubuntu 24.04) liefern Version 4.1 (`javascriptcoregtk-4.1.pc`).
- Symlinks (`ln -s ... 4.1.pc 4.0.pc`) sind ein Workaround.

## Lösung
Ein Setup-Skript `scripts/setup_linux.sh` erstellen/aktualisieren, das:
1. `apt-get install` für alle Pakete ausführt.
2. Die Symlink-Prüfung vornimmt und ggf. setzt (mit Warnung).

## Paket-Liste
- `libwebkit2gtk-4.1-dev`
- `libjavascriptcoregtk-4.1-dev`
- `libsoup2.4-dev`
- `libgtk-3-dev`
- `libglib2.0-dev`
- `libssl-dev` (oft implizit, aber besser explizit)

## Plan
- Skript erstellen und ausführbar machen.
- In `docs/todo.md` referenzieren.

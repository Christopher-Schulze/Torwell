# Technical Detail: Tauri Configuration (Todo-003)

## Problembeschreibung
Die Datei `src-tauri/tauri.conf.json` enthält Platzhalter und Konfigurationen, die für einen Release ungültig sind.

## Analyse
1. **`distDir`**: Zeigt auf `../build`. Dies ist korrekt für SvelteKit Static Adapter, muss aber existieren.
2. **Updater Config**:
   ```json
   "endpoints": ["https://example.com/update"],
   "pubkey": "AAAAAAAAAAAAAAAA..."
   ```
   Das ist ein Dummy. Ohne echte Keys funktioniert der Updater nicht.
3. **Bundle Identifier**: `com.torwell84.v2.app`. Okay.

## Maßnahmen
1. **Updater:** Da wir keine echten Keys haben, sollte der Updater entweder deaktiviert werden (`active: false`) oder mit einem TODO-Kommentar/Platzhalter dokumentiert bleiben, dass dies vor Release geändert werden muss. Ich werde es vorerst auf `active: false` setzen, um Fehler zur Laufzeit zu vermeiden, bis der Nutzer Keys bereitstellt.
2. **Validation:** Sicherstellen, dass Icons existieren.

## Plan
- `updater.active` auf `false` setzen.
- `endpoints` entfernen oder auskommentieren.

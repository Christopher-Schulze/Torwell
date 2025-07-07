# Cloudflare Worker als Proxy

Dieses Dokument beschreibt, wie du den Beispiel‑Worker aus dem Ordner `cf worker` deployen kannst und wie der Proxy anschließend in Torwell84 eingebunden wird.

## Worker deployen

1. Installiere [Wrangler](https://developers.cloudflare.com/workers/wrangler/) einmalig:
   ```bash
   bun add -g wrangler
   ```
2. Starte ein neues Worker‑Projekt im gewünschten Verzeichnis:
   ```bash
   wrangler init
   ```
   Ersetze die erzeugte Datei `src/index.js` anschließend durch den Inhalt aus `cf worker/Super-HTTPS-Proxy-CF-Worker-.txt`.
3. Lege ein geheimes Token an, das der Worker zur Authentifizierung erwartet:
   ```bash
   wrangler secret put SECRET_TOKEN
   ```
4. Veröffentliche den Worker mit:
   ```bash
   wrangler deploy
   ```
   Der Worker prüft bei jeder Anfrage, ob der Header `X-Proxy-Token` dem gesetzten `SECRET_TOKEN` entspricht.

## Proxy in Torwell84 einrichten

Trage die URL deines Workers in der Anwendung unter **Settings → Worker List** ein und hinterlege das geheime Token im Feld **Worker token**. Du kannst mehrere Adressen hinzufügen. Torwell84 probiert sie nacheinander aus und rotiert automatisch weiter, wenn ein Endpunkt nicht erreichbar ist. Alternativ kannst du Adressen in `src/lib/bridge_presets.json` hinterlegen, damit sie beim ersten Start bereits vorgeschlagen werden.

Um viele Worker-Adressen komfortabel einzubinden, liest das Skript `scripts/import_workers.ts` eine Datei mit jeweils einer URL pro Zeile und übergibt sie per `set_worker_config` an den laufenden Dienst. Im Einstellungsdialog steht zudem der Button **Import Worker List** bereit, der die Liste aus einer Datei übernimmt.
Ab Version 2.3 kannst du die aktuelle Liste auch über **Export Worker List** als Textdatei herunterladen und einfach weitergeben.

Nach dem Speichern der Einstellungen werden alle über den Worker geleiteten Verbindungen mit dem gesetzten Token authentifiziert. Mehrere Worker erhöhen Zuverlässigkeit und ermöglichen eine einfache horizontale Skalierung.

## Token-Verwaltung und Batch-Import

Das Feld **Worker token** sollte den geheimen Wert enthalten, den du beim Deploy des Workers unter `SECRET_TOKEN` definiert hast. Beim Speichern prüft Torwell84 automatisch, ob der Token gültig ist und warnt dich bei Fehlern.

Um sehr große Listen einzubinden, kannst du das Skript `scripts/import_workers.ts` verwenden:

```bash
bun scripts/import_workers.ts worker-list.txt meinToken
```

Damit lassen sich hunderte URLs bequem importieren.

## Hardware Security Module verwenden

Unter **Settings → HSM Configuration** kannst du den Pfad zur PKCS#11‑Bibliothek und den Slot angeben. Nach dem Speichern werden die Werte im Backend übernommen und für neue TLS‑Verbindungen genutzt.

Unter **Settings → Update Interval** legst du fest, in welchem Abstand (in Sekunden) das Zertifikat automatisch aktualisiert wird.

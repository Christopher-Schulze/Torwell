# Cloudflare Worker als Proxy

Dieses Verzeichnis enthält Beispielskripte für einen HTTPS‑Proxy als Cloudflare Worker. Die folgenden Schritte fassen kurz zusammen, wie der Worker bereitgestellt wird. Ausführlichere Hinweise stehen in [docs/Todo-fuer-User.md](../docs/Todo-fuer-User.md).

## Deployment

1. Wrangler installieren:
   ```bash
   bun add -g wrangler
   ```
2. Neues Worker-Projekt anlegen und die generierte `src/index.js` durch `Super-HTTPS-Proxy-CF-Worker-.txt` aus diesem Ordner ersetzen:
   ```bash
   wrangler init
   ```
3. Benötigtes Token setzen:
   ```bash
   wrangler secret put SECRET_TOKEN
   ```
4. Worker veröffentlichen:
   ```bash
   wrangler deploy
   ```
   Der Worker prüft bei jeder Anfrage, ob der Header `X-Proxy-Token` mit dem gesetzten `SECRET_TOKEN` übereinstimmt.

   Die Zieladresse wird im Query-Parameter `url` übergeben, z.B. `https://<worker-url>/?url=https://example.com`.

Nach dem Deployment kann die URL des Workers in Torwell84 unter **Settings → Worker List** eingetragen und das Token hinterlegt werden. Optional lässt sich die Adresse in `src/lib/bridge_presets.json` vorbelegen.
Torwell84 validiert den eingetragenen Token sofort mittels `validate_worker_token`. Erst nach erfolgreichem Test wird die Konfiguration aktiv.

Mehrere Worker lassen sich komfortabel mit dem Skript `scripts/import_workers.ts` importieren.

# Security Audit Findings

This report summarizes security issues discovered during a brief review of the repository.

## 1. Example Certificate URL
- **File:** `src-tauri/certs/cert_config.json`
- **Issue:** `cert_url` points to the default update endpoint `https://certs.torwell.com/server.pem`.
- **Risk:** Deployments using a different update server must change this value; otherwise certificate updates could be fetched from an untrusted source.
- **Recommendation:** Replace `cert_url` with your own HTTPS endpoint before release or override it via `TORWELL_CERT_URL`.

## 2. Local Storage Encryption
- **File:** `src/lib/database.ts`
- **Issue:** Settings are encrypted with AES‑GCM using a 256‑bit key. Earlier versions stored this key (base64 encoded) in the `meta` table of IndexedDB.
- **Risk:** Storing the key in the database meant an attacker with local file access could decrypt the data.
- **Resolution:** The key is now saved in the operating system keychain via the Tauri keychain plugin and removed from IndexedDB on first launch after the update.

## 3. Ping Command Implementation
- **File:** `src-tauri/src/commands.rs`
- **Issue:** The `ping_host` command now calls `icmp::ping_host`, which relies on the `surge_ping` crate to send ICMP packets rather than spawning an external `ping` binary.
- **Risk:** Excessive calls could still be used for resource consumption, but no external process is executed.
- **Recommendation:** Continue validating input and capping the invocation count to mitigate abuse.

No critical vulnerabilities were found, but the above issues should be addressed to improve the overall security posture.

## Audit Vorbereitung

Die Abhängigkeitsanalysen wurden gemäß dem Security Audit Plan durchgeführt.

- **cargo audit** (05.07.2025): Eine Verwundbarkeit (RUSTSEC-2023-0071 im Paket `rsa`) und 14 Warnungen.
- **bun audit**: Der Befehl war nicht verfügbar; es wurden keine Schwachstellen gemeldet.


## 4. Network and IPC Fuzzing

Ein neues Testskript `src-tauri/tests/fuzz_commands.rs` führt zufällige Aufrufe der Tauri-Commands aus. Dabei werden insbesondere `ping_host` und `set_exit_country` mit zufälligen Hostnamen, Aufrufzählern und Länderkürzeln gefüttert. Die Fuzzing-Schleifen helfen, unerwartete Panics und Validierungsfehler frühzeitig zu erkennen.

Bei ersten Durchläufen traten keine Abstürze auf. Ungültige Eingaben wurden korrekt mit Fehlermeldungen beantwortet und das Rate Limiting griff wie erwartet.

## 5. Penetration Tests

Ein weiteres Skript `src-tauri/tests/pentest.rs` simuliert unautorisierte Anfragen
und einen massiven Aufruf von Befehlen.
Ungültige Tokens werden konsequent mit `Error::InvalidToken` abgelehnt. Nach mehr
als 60 gültigen Aufrufen greift der globale Rate Limiter und liefert
`Error::RateLimitExceeded`. Somit funktionieren die Sitzungsprüfung und das
Rate‑Limiting wie vorgesehen.

Seit dem letzten Update enthalten auch Verbindungs- und Identitätsfehler eine
genaue Beschreibung des Schritts, in dem sie auftraten. Beispiele:

```text
Error::ConnectionFailed { step: "bootstrap", source: "bootstrap: connection lost" }
Error::Identity { step: "build_circuit", source: "build_circuit: timeout" }
```

## Aktueller Stand (2025-07-05)

Aktuell sind keine weiteren offenen Findings bekannt.

## Abschluss

Penetration Tests abgeschlossen – keine offenen Punkte.

# Security Audit Findings

This report summarizes security issues discovered during a brief review of the repository.

## 1. Example Certificate URL
- **File:** `src-tauri/certs/cert_config.json`
- **Issue:** `cert_url` points to the default update endpoint `https://certs.torwell.com/server.pem`.
- **Risk:** Deployments using a different update server must change this value; otherwise certificate updates could be fetched from an untrusted source.
- **Recommendation:** Replace `cert_url` with your own HTTPS endpoint before release or override it via `TORWELL_CERT_URL`.

## 2. Local Storage Encryption
- **File:** `src/lib/database.ts`
- **Issue:** Settings are encrypted with AES‑GCM using a 256‑bit key. The key is generated on first run and stored (base64 encoded) in the `meta` table of IndexedDB.
- **Risk:** Because the AES key is saved alongside the encrypted values, an attacker with access to local files can still decrypt the data. The mechanism mainly protects against casual inspection and tampering.
- **Recommendation:** Consider storing the key in a platform keystore or rely on OS‑level permissions to prevent unauthorized access. Local storage remains vulnerable to malware or physical compromise.

## 3. External `ping` Command
- **File:** `src-tauri/src/commands.rs`
- **Issue:** The `ping_host` command spawns the system `ping` binary.
- **Risk:** Although arguments are passed directly, excessive calls could be abused for resource consumption.
- **Recommendation:** Validate input and limit invocations or implement an internal ICMP check instead of spawning a process.

No critical vulnerabilities were found, but the above issues should be addressed to improve the overall security posture.


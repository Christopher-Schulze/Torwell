# Security Audit Findings

This report summarizes security issues discovered during a brief review of the repository.

## 1. Example Certificate URL
- **File:** `src-tauri/certs/cert_config.json`
- **Issue:** `cert_url` points to the default update endpoint `https://certs.torwell.com/server.pem`.
- **Risk:** Deployments using a different update server must change this value; otherwise certificate updates could be fetched from an untrusted source.
- **Recommendation:** Replace `cert_url` with your own HTTPS endpoint before release or override it via `TORWELL_CERT_URL`.

## 2. Local Storage Encryption
- **File:** `src/lib/database.ts`
- **Issue:** Settings are encrypted with AES using a static key.
- **Risk:** While stronger than the previous XOR approach, the static key still means an attacker with local access could decrypt the data.
- **Recommendation:** Implement per-user key management or rely on OS-level permissions to restrict access.

## 3. External `ping` Command
- **File:** `src-tauri/src/commands.rs`
- **Issue:** The `ping_host` command spawns the system `ping` binary.
- **Risk:** Although arguments are passed directly, excessive calls could be abused for resource consumption.
- **Recommendation:** Validate input and limit invocations or implement an internal ICMP check instead of spawning a process.

No critical vulnerabilities were found, but the above issues should be addressed to improve the overall security posture.


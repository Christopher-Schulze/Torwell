# Security Audit Findings

This report summarizes security issues discovered during a brief review of the repository.

## 1. Example Certificate URL
- **File:** `src-tauri/certs/cert_config.json`
- **Issue:** `cert_url` defaults to an example domain.
- **Risk:** Using the placeholder URL may allow an attacker to replace the pinned certificate if the domain is not updated.
- **Recommendation:** Update `cert_url` to a trusted location before deployment.

## 2. Unencrypted Local Storage
- **File:** `src/lib/database.ts`
- **Issue:** Application settings are stored unencrypted in IndexedDB.
- **Risk:** Local attackers could read configuration values such as bridge lists or exit-country preferences.
- **Recommendation:** Consider encrypting sensitive fields or protecting access with OS-level permissions.

## 3. External `ping` Command
- **File:** `src-tauri/src/commands.rs`
- **Issue:** The `ping_host` command spawns the system `ping` binary.
- **Risk:** Although arguments are passed directly, excessive calls could be abused for resource consumption.
- **Recommendation:** Validate input and limit invocations or implement an internal ICMP check instead of spawning a process.

No critical vulnerabilities were found, but the above issues should be addressed to improve the overall security posture.


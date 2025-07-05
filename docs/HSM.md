# Hardware Security Module (HSM) Support

Starting with version 2.4 the backend can manage keys via a PKCS#11 module.
This functionality is compiled in by enabling the `hsm` feature.

## Building with HSM support

```bash
cargo build --release --manifest-path src-tauri/Cargo.toml --features hsm
```

The PKCS#11 library path is read from the `TORWELL_HSM_LIB` environment
variable. If unset, `/usr/lib/softhsm/libsofthsm2.so` is used.

Example using a YubiHSM:

```bash
    TORWELL_HSM_LIB=/usr/local/lib/libyubihsm_pkcs11.so \
    bun tauri build --features hsm
```

## Einrichtung mit SoftHSM

1. Installiere das Paket `softhsm2` auf dem System.
2. Initialisiere ein neues Token im ersten Slot:
   ```bash
   softhsm2-util --init-token --slot 0 \
       --label "torwell" --so-pin 0102030405060708 --pin 1234
   ```
3. Stelle sicher, dass die Umgebungsvariable `TORWELL_HSM_LIB`
   auf `/usr/lib/softhsm/libsofthsm2.so` zeigt (Standard).
4. Baue Torwell anschließend wie oben beschrieben mit dem Feature `hsm`.

## Usage in SecureHttpClient

When the feature is enabled `SecureHttpClient` initialises the PKCS#11
context during TLS configuration. Keys stored on the HSM can be accessed
through the loaded module. The current implementation only loads the
module and finalises it again; you can extend `secure_http.rs` to fetch
certificates or signing keys as needed.

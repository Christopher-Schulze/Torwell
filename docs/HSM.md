# Hardware Security Module (HSM) Support

Starting with version 2.4 the backend can manage keys via a PKCS#11 module.
This functionality is compiled in by enabling the `hsm` feature.

## Building with HSM support

```bash
cargo build --release --manifest-path src-tauri/Cargo.toml --features hsm
```

The PKCS#11 library path is read from the `TORWELL_HSM_LIB` environment
variable. If unset, `/usr/lib/softhsm/libsofthsm2.so` is used. The slot
number can be configured via `TORWELL_HSM_SLOT` (default `0`).
Both values can also be stored in `src-tauri/app_config.json` under
`hsm_lib` and `hsm_slot`.

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
context during TLS configuration. The client looks for objects labelled
`tls-cert` and `tls-key` on the configured slot and will use them for
mutual TLS authentication. For testing you can provide base64 encoded
values through the variables `TORWELL_HSM_MOCK_CERT` and
`TORWELL_HSM_MOCK_KEY`.

Example to run the binary with SoftHSM:

```bash
TORWELL_HSM_LIB=/usr/lib/softhsm/libsofthsm2.so \
TORWELL_HSM_SLOT=0 \
TORWELL_HSM_PIN=1234 \
bun tauri dev --features hsm
```

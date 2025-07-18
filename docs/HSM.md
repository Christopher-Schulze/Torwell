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
2. Lege einen Ordner für die Token-Daten an und erstelle eine Konfigurationsdatei:
   ```bash
   mkdir -p ~/hsm/tokens
   cat >~/hsm/softhsm2.conf <<EOF
   directories.tokendir = ~/hsm/tokens
   EOF
   export SOFTHSM2_CONF=~/hsm/softhsm2.conf
   ```
3. Initialisiere ein neues Token im ersten Slot und importiere einen privaten Schlüssel sowie ein Zertifikat:
   ```bash
   softhsm2-util --init-token --slot 0 \
       --label "torwell" --so-pin 0102030405060708 --pin 1234
   openssl req -x509 -newkey rsa:2048 -nodes -keyout key.pem -out cert.pem \
       -subj "/CN=torwell" -days 365
   softhsm2-util --import key.pem --token torwell --label tls-key --id 01 --pin 1234
   pkcs11-tool --module /usr/lib/softhsm/libsofthsm2.so --slot 0 --pin 1234 \
       -w cert.pem -y cert -d 01 -a tls-cert
   ```
4. Setze die Variablen `TORWELL_HSM_LIB`, `TORWELL_HSM_SLOT` und `TORWELL_HSM_PIN` entsprechend.  
   Für SoftHSM kann beispielsweise folgender Wert verwendet werden:
   ```bash
   export TORWELL_HSM_LIB=/usr/lib/softhsm/libsofthsm2.so
   export TORWELL_HSM_SLOT=0
   export TORWELL_HSM_PIN=1234
   ```
5. Baue Torwell anschließend wie oben beschrieben mit dem Feature `hsm` und starte die Anwendung.

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

## Minimal example for testing

For automated tests or quick experiments you can use SoftHSM without
permanent state. The snippet below creates a temporary token, imports a
key pair and certificate and runs the tests with HSM support enabled.

```bash
TMP=/tmp/hsm-test
mkdir -p "$TMP/tokens"
cat >"$TMP/softhsm2.conf" <<EOF
directories.tokendir = $TMP/tokens
EOF
export SOFTHSM2_CONF="$TMP/softhsm2.conf"
softhsm2-util --init-token --slot 0 --label torwell \
    --so-pin 0102030405060708 --pin 1234
softhsm2-util --import path/to/key.pem --token torwell \
    --label tls-key --id 01 --pin 1234
pkcs11-tool --module /usr/lib/softhsm/libsofthsm2.so --token-label torwell \
    --pin 1234 -w path/to/cert.pem -y cert -d 01 -a tls-cert
export TORWELL_HSM_LIB=/usr/lib/softhsm/libsofthsm2.so
export TORWELL_HSM_SLOT=$(softhsm2-util --show-slots | \
    awk '/Label:\s*torwell/{getline;print $2}')
export TORWELL_HSM_PIN=1234
cargo test --features hsm
```

When using a YubiHSM replace the library path with the location of
`libyubihsm_pkcs11.so` and omit the SoftHSM setup steps.

## Troubleshooting

If initialization fails, verify the configured slot and PIN values. SoftHSM
returns `CKR_SLOT_ID_INVALID` or `CKR_TOKEN_NOT_PRESENT` when the slot number
does not match an available token. Errors like `CKR_PIN_INCORRECT` or
`CKR_PIN_LEN_RANGE` indicate an invalid PIN. Check `TORWELL_HSM_SLOT` via
`softhsm2-util --show-slots` and ensure `TORWELL_HSM_PIN` is correct.

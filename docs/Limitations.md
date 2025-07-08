# Known Limitations

## Circuit Metrics

`TorManager::circuit_metrics` is implemented and relies on the optional
`experimental-api` feature of `arti-client` to query currently open circuits.
Without this feature only an approximation of the circuit count is possible and
metrics like build time remain unknown. They are reported as `null` to the
frontend. Enable the flag with `--features experimental-api` (or use
`task build`, which sets it by default) for fully accurate metrics.

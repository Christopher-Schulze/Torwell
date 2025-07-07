# Known Limitations

## Circuit Metrics

`TorManager::circuit_metrics` relies on the optional `experimental-api` feature
of `arti-client` to query currently open circuits. When this feature is disabled
the library now estimates the number of circuits based on existing isolation
tokens. Other metrics such as build time remain unknown in this mode and are
reported as `null` to the frontend. Enable the flag with `--features
experimental-api` (or use `task build`, which sets it by default) for fully
accurate metrics.

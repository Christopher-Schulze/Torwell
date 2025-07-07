# Known Limitations

## Circuit Metrics

`TorManager::circuit_metrics` relies on the optional `experimental-api` feature
of `arti-client` to query currently open circuits. When the crate is compiled
without this feature the method falls back to returning zero values. Enable the
flag with `--features experimental-api` (or use `task build`, which sets it by
default) for accurate metrics based on the experimental APIs.

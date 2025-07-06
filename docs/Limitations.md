# Known Limitations

## Circuit Metrics

`TorManager::circuit_metrics` relies on the optional `experimental-api` feature
of `arti-client` to query currently open circuits. When compiled without this
feature the method falls back to returning zero values. Accurate metrics are
therefore only available in builds using the experimental APIs.

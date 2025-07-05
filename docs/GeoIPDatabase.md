# GeoIP Database Configuration

Torwell84 can use an external GeoIP database instead of the embedded one shipped with `arti`.  The path to the directory containing the `geoip` and `geoip6` files can be specified in `src-tauri/app_config.json`:

```json
{
  "max_log_lines": 1000,
  "geoip_path": "path/to/geoip_dir"
}
```

If the path is invalid or omitted, the application falls back to the embedded database.
You may also override the setting with the `TORWELL_GEOIP_PATH` environment variable.

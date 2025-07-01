# Troubleshooting

This guide lists common problems encountered during development and how to analyse logs.

## Common Issues

- **Missing system libraries**: `cargo check` may fail if `glib-2.0` or other packages are not installed. Install the required development libraries or set `PKG_CONFIG_PATH` accordingly.
- **Dependencies not installed**: If the frontend will not build, run `bun install` or `pnpm install` to fetch Node packages.
- **Build errors**: Ensure `pnpm run check` and `cargo check` succeed before opening a pull request.

## Debugging & Log Analysis

- Start the app in development mode with `pnpm tauri dev` to view live output.
- The backend writes logs to a persistent file named `torwell.log` in the project directory. Older entries are trimmed once the file exceeds the configured line limit.
- Each line of this file is a JSON object with `level`, `timestamp` and `message` fields.
- If the UI fails to load, open the browser developer tools (`Ctrl+Shift+I`) to inspect console logs and network activity.
- Failed connection attempts are recorded with `WARN` level. The retry counter resets when a new connection starts.
- If `Error::Timeout` occurs, the Tor bootstrap exceeded the allowed time. Check your network or increase the limit.
- The function `connect_with_backoff` enforces a maximum overall connection time and logs each retry.

## Rate Limits

Some commands are throttled to avoid excessive resource usage. By default:

- The `connect` command may be invoked up to **3 times per minute**.
- The `get_logs` command may be invoked up to **10 times per minute**.

Exceeding these limits results in an `Error::RateLimited` response. When a
connection attempt is blocked, the frontend also receives a
`tor-status-update` event with status `RATE_LIMIT` so the user is informed
immediately.


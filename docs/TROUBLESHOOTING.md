# Troubleshooting

This guide lists common problems encountered during development and how to analyse logs.

## Common Issues

- **Missing system libraries**: `cargo check` may fail if `glib-2.0` or other packages are not installed. Install the required development libraries or set `PKG_CONFIG_PATH` accordingly.
- **Dependencies not installed**: If the frontend will not build, run `bun install` or `pnpm install` to fetch Node packages.
- **Build errors**: Ensure `pnpm run check` and `cargo check` succeed before opening a pull request.

## Debugging & Log Analysis

- Start the app in development mode with `pnpm tauri dev` to view live output.
- The backend writes logs to a persistent file named `torwell.log` in the project directory. Older entries are trimmed once the file exceeds the configured line limit.
- If the UI fails to load, open the browser developer tools (`Ctrl+Shift+I`) to inspect console logs and network activity.


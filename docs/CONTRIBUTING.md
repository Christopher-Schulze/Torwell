# Contributing to Torwell84

This project welcomes community contributions. Please follow the guidelines below before submitting a pull request.

## Code Style

- Format Rust code with `cargo fmt`.
- Format frontend files using the configured Prettier rules.
- Keep functions small and well commented.

## Testing

1. Install dependencies with `bun install` (or `pnpm install`).
2. Run `pnpm run check` to verify the Svelte frontend.
3. Run `cargo check` in `src-tauri` to ensure the backend builds.

## Pull Request Process

1. Fork the repository and create a feature branch.
2. Run the tests described above and fix any issues.
3. Update documentation if needed.
4. Open a pull request targeting `main` with a clear description of your changes.


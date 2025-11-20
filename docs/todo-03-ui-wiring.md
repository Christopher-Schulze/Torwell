# UI Wiring & Reliability

## Goal
Ensure the UI accurately reflects the backend state and handles errors gracefully.

## Tasks
1.  **Command Robustness**:
    -   Wrap `connect`, `disconnect` in `commands.rs` with better error logging.
    -   Ensure `TorManager` state updates are emitted to the frontend immediately.
2.  **Debug UI**:
    -   Verify `ActionCard.svelte` logic matches the backend events.
    -   Fix any disconnected signals.

## Implementation Details
-   Check `src-tauri/src/commands.rs`.
-   Check `src-tauri/src/lib.rs` (event loops).

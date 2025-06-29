# Torwell84 V2 - Documentation

## 1. Architecture

Torwell84 V2 is a complete rewrite focusing on a clean, modern, and maintainable architecture. The legacy Go backend and all associated code have been entirely discarded.

The new architecture is unified and consists of the following components:

-   **Frontend:** A SvelteKit single-page application responsible for the entire user interface. It is located in the `/` directory of the V2 project.
-   **Backend (Core):** A single, modular Rust crate located in `/src-tauri`. This crate is directly integrated with the Tauri runtime and serves as the application's core logic. There is no separate backend process.
-   **Communication:** The frontend and backend communicate exclusively and efficiently through Tauri's built-in IPC (Inter-Process Communication) system, using Tauri commands (`invoke`) and events (`listen`). This eliminates the need for any local web server, gRPC, or other network protocols between the frontend and backend.

## 2. Core Components

### 2.1. Rust Backend (`/src-tauri`)

The backend is structured into several logical modules:

-   **`main.rs` & `lib.rs`:** The entry points for the Tauri application, responsible for initializing the builder, managing the application state, and registering commands.
-   **`state.rs`:** Defines the shared `AppState`, which holds the `TorManager` instance and log storage, ensuring a single, consistent state for the Tor client across the application.
-   **`tor_manager.rs`:** A dedicated, robust module that encapsulates all interactions with the `arti-client` library. It handles the lifecycle of the Tor client, including connection, disconnection, circuit management (`get_active_circuit`), and requesting a new identity (`new_identity`).
-   **`commands.rs`:** Implements all Tauri commands that are exposed to the Svelte frontend. These functions act as thin, clean wrappers, delegating all business logic to the `TorManager`. This ensures a clear separation of concerns.
-   **`error.rs`:** Defines a custom, serializable `Error` enum for the entire backend. In addition to `NotConnected` and `AlreadyConnected`, it exposes variants for bootstrap failures, directory lookups, circuit building issues and identity refresh problems. These descriptive errors are serialized and sent to the frontend for user-friendly reporting.

### 2.2. Svelte Frontend (`/src`)

The frontend remains visually and functionally identical to the original design, as per the requirements.

-   **State Management:**
    -   `torStore.ts`: A Svelte store that subscribes to backend events (`tor-status-update`) to reactively display the current Tor connection status, bootstrap progress, and errors.
    -   `uiStore.ts`: A Svelte store for managing the state of the UI, such as open modals. It also handles client-side settings persistence using `Dexie.js`.
-   **Components:** All UI components from the original version are reused without modification to their appearance. The logic within them now communicates with the robust and fully implemented Rust backend via Tauri's `invoke` API. Mock data and placeholders have been removed.

## 3. New Features in V2.1

### 3.1 New Identity Functionality
- Added ability to request new Tor circuits via `new_identity` command
- Full integration with frontend UI using dedicated button
- Uses arti-client's `reconfigure` and `retire_all_circs` for identity refresh

### 3.2 Logging System
- Centralized log storage in AppState with thread-safe access
- Automatic log rotation (max 1000 entries)
- Commands for log retrieval and clearing

### 3.3 Documentation Updates
- Comprehensive changelog tracking
- Task list for future improvements

## 4. Build Process

The application is built as a standard Tauri project:

1.  The SvelteKit frontend is built into a set of static assets (HTML, CSS, JS).
2.  The Rust backend is compiled into a binary.
3.  The Tauri bundler packages the frontend assets and the Rust binary into a single, native executable for the target platform (e.g., `.app` for macOS, `.exe` for Windows).

## 5. Error States

Errors from the backend are emitted through the `tor-status-update` event. The main variants are:

- `NotConnected` – a command requiring an active Tor connection was invoked while disconnected.
- `AlreadyConnected` – a connection was attempted when one already exists.
- `Bootstrap` – the Tor client failed to bootstrap.
- `NetDir` – the network directory could not be retrieved.
- `Circuit` – building or retrieving a circuit failed.
- `Identity` – refreshing the Tor identity was unsuccessful.

The serialized error message is provided in the event's `errorMessage` field so the frontend can display user-friendly feedback.

## 6. Traffic Statistics

Version 2.2 introduces live traffic counters. `TorManager` exposes the total bytes
sent and received via `traffic_stats()`, and the `get_traffic_stats` Tauri command
passes these values to the frontend. The main status card now periodically
displays the aggregate traffic in megabytes.

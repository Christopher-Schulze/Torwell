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
-   **`state.rs`:** Defines the shared `AppState`, which holds the `TorManager` instance, ensuring a single, consistent state for the Tor client across the application.
-   **`tor_manager.rs`:** A dedicated, robust module that encapsulates all interactions with the `arti-client` library. It exclusively handles the lifecycle of the Tor client, including connection, disconnection, circuit management (`get_active_circuit`), and requesting a new identity (`new_identity`).
-   **`commands.rs`:** Implements all Tauri commands that are exposed to the Svelte frontend. These functions act as thin, clean wrappers, delegating all business logic to the `TorManager`. This ensures a clear separation of concerns.
-   **`error.rs`:** Defines a custom, serializable `Error` enum for the entire backend, including specific, descriptive errors like `NotConnected` or `AlreadyConnected`. This ensures that all errors are handled gracefully and can be sent to the frontend in a structured way.

### 2.2. Svelte Frontend (`/src`)

The frontend remains visually and functionally identical to the original design, as per the requirements.

-   **State Management:**
    -   `torStore.ts`: A Svelte store that subscribes to backend events (`tor-status-update`) to reactively display the current Tor connection status, bootstrap progress, and errors.
    -   `uiStore.ts`: A Svelte store for managing the state of the UI, such as open modals. It also handles client-side settings persistence using `Dexie.js`.
-   **Components:** All UI components from the original version are reused without modification to their appearance. The logic within them now communicates with the robust and fully implemented Rust backend via Tauri's `invoke` API. Mock data and placeholders have been removed.

## 3. Build Process

The application is built as a standard Tauri project:

1.  The SvelteKit frontend is built into a set of static assets (HTML, CSS, JS).
2.  The Rust backend is compiled into a binary.
3.  The Tauri bundler packages the frontend assets and the Rust binary into a single, native executable for the target platform (e.g., `.app` for macOS, `.exe` for Windows).
# Migration Study: Research Hub → Rust + Tauri 2.0

**Status**: Study (Implemented)

> **Note:** The full 70+ page technical study is currently only available in Portuguese. Below is an executive summary of the architectural decisions. You can read the [full Portuguese document here](/research_hub/estudo-migracao-rust-tauri/).

## Executive Summary

The Research Hub was originally conceived as a Python/FastAPI web application. However, as the requirements evolved, it became clear that the application was meant to be a single-user, local desktop tool.

### Why Migrate?

1. **Distribution Complexity:** Python environments, `venv` management, and dependencies were too complex for end-users to install.
2. **Over-engineering:** A full HTTP layer, CORS, and multipart uploads were unnecessary for local files.
3. **Performance:** Reading heavy parquet files and relying on network overhead created bottlenecks.

### The New Architecture

We chose **Rust + Tauri 2.0** as the new stack:
- **Core (Rust):** Handles system access, SQLite database (`rusqlite`), and data processing (`arrow-rs`).
- **Desktop App (Tauri):** Provides a native window and IPC (Inter-Process Communication) without requiring a local web server.
- **Frontend (Astro + React):** The existing frontend was preserved, compiled statically, and embedded directly into the Rust binary.

### Key Benefits

- **Single Binary:** Users just download a `.deb`, `.rpm`, `.AppImage`, or `.exe` and run it. No installations required.
- **Zero Network:** True offline-first desktop application.
- **Improved Performance:** Direct memory access to local SQLite instead of HTTP API calls.

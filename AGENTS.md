# Repository Guidelines

Research Hub is a **desktop application**: a Rust core on Tauri 2.0 with an
Astro/React frontend embedded in the binary. There is no server, no HTTP layer
and no Python.

- Business logic lives in `src-tauri/src/`. The frontend renders and calls
  commands; it never holds SQL, file access or duplicated rules.
- All JS→Rust traffic goes through `frontend/src/services/api.js`. Keep it that
  way — it is the only coupling point, and it is why the React components
  survived the migration untouched.
- Storage is a single SQLite file in the OS app-data directory. Do not
  introduce an engine that needs a server.
- Entity metadata belongs in `src-tauri/src/registry.rs` — one place, not three.
- Tests are mandatory (`cargo test`, `npm test`). Write them first.

See `.specify/memory/constitution.md` for the binding rules and
`docs/estudo-migracao-rust-tauri.md` for the reasoning behind the architecture.

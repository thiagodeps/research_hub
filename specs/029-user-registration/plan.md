# Implementation Plan: Account Creation and Password Hashing

**Branch**: `029-user-registration` | **Date**: 2026-09-22 | **Spec**: [specs/029-user-registration/spec.md](spec.md)  
**Input**: Feature specification from `/specs/029-user-registration/spec.md`

## Summary

Enable user registration with secure password hashing and local desktop authentication for ResearchHub. The implementation introduces a dedicated account creation page (`/register`) with split-screen visual identity, validates inputs (email structure, matching confirmation, minimum 8 characters), executes cryptographically secure salted hashing via `bcrypt` in the Rust backend, persists accounts in SQLite (`admins` table), and redirects users to the initial login screen with confirmation feedback so they can authenticate into the workspace.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021) / JavaScript (Node.js 20+, React 18, Astro)  
**Primary Dependencies**: Tauri 2.0 (`tauri`), `rusqlite 0.32` (bundled), `bcrypt 0.15`, React, TailwindCSS  
**Storage**: SQLite embedded single-file (`hub.db` in OS app-data directory)  
**Testing**: `cargo test` (Rust unit & integration with in-memory SQLite), `npm test` (Vitest for React components and IPC bridge)  
**Target Platform**: Linux and Windows (desktop application)  
**Project Type**: desktop-app  
**Performance Goals**: Registration & password hashing completed in <50ms; responsive UI feedback (<100ms)  
**Constraints**: 100% offline-capable, zero external network/HTTP dependencies, single SQLite database file, business logic exclusively in Rust  
**Scale/Scope**: Single-tenant local desktop curation tool, accounts stored in SQLite `admins` table  

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **I — TDD Adherence**: Test planning is prioritized and tasks clearly follow the Red-Green-Refactor cycle. Tests for registration, bcrypt hashing, error variants, duplicate email rejection, and component rendering will be written before implementation.
- [x] **II — Functional Parity**: The existing seeded admin (`admin@admin.com` / `admin123`) remains intact and functional. Parity with data import/export pipelines is unaffected because the `admins` table is excluded from parquet exports by definition (`registry::exported`).
- [x] **III — Test Strategy**: Rust core uses `cargo test` covering validation, bcrypt hashing, and database persistence against in-memory SQLite. Frontend uses `npm test` (Vitest) for `RegisterForm.jsx`, `LoginForm.jsx` links/alerts, and `api.js` IPC mapping.
- [x] **IV — Operations**: Direct CRUD operations by Admin only; all registered accounts operate with local Admin privileges, maintaining the single-role architecture without approval flows.
- [x] **V — Desktop Architecture**: Single-process Tauri 2.0 desktop application; Astro/React frontend statically compiled and embedded; SQLite single-file storage in the OS app-data directory; no HTTP server or external APIs; works 100% offline. Interface strictly adheres to Figma split-screen branding.
- [x] **VI — Business Logic in Rust**: Credential validation (email regex, password length, confirmation equality), duplicate checking, and bcrypt hashing reside strictly in `src-tauri/src/auth.rs`. All JS↔Rust communication is routed through `frontend/src/services/api.js`.
- [x] **Deferred Scope**: No macOS-specific code, no code signing, no auto-update, and no external/non-SQLite database engine introduced.

## Project Structure

### Documentation (this feature)

```text
specs/029-user-registration/
├── spec.md              # Feature specification
├── plan.md              # Implementation plan (this document)
├── research.md          # Phase 0 technical decisions & rationale
├── data-model.md        # Phase 1 data entities and validation rules
├── quickstart.md        # Phase 1 verification and testing guide
├── contracts/           # Phase 1 IPC and API bridge contracts
│   └── api.md
├── checklists/          # Quality verification checklists
│   └── requirements.md
└── tasks.md             # Phase 2 task decomposition (via /speckit-tasks)
```

### Source Code Layout

```text
src-tauri/
├── src/
│   ├── auth.rs          # Registration logic (validate_registration, create_account, bcrypt hashing)
│   ├── commands.rs      # Tauri IPC command `register`
│   ├── error.rs         # AppError::Validation variants
│   └── lib.rs           # Command registration in `tauri::generate_handler!`
└── tests/ (or in-module tests in auth.rs)

frontend/
├── src/
│   ├── components/
│   │   ├── LoginForm.jsx     # Navigation link to /register + success alert on ?registered=true
│   │   └── RegisterForm.jsx  # New registration form component (email, password, confirm)
│   ├── pages/
│   │   ├── login.astro       # Login page using SplitScreen layout
│   │   └── register.astro    # New registration page using SplitScreen layout
│   └── services/
│       └── api.js            # POST /auth/register IPC bridge route
└── tests/
    ├── LoginForm.test.jsx
    ├── RegisterForm.test.jsx
    └── api.test.js
```

**Structure Decision**: Multi-layer desktop layout preserving strict separation:
1. `src-tauri/src/auth.rs` owns password hashing and account persistence.
2. `src-tauri/src/commands.rs` exposes the thin `register` IPC command.
3. `frontend/src/services/api.js` encapsulates the IPC `invoke('register', ...)`.
4. `frontend/src/pages/register.astro` and `RegisterForm.jsx` provide the user interface.

## Complexity Tracking

*No constitutional violations; no additional architectural complexity introduced.*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|:---|:---|:---|
| *None* | *N/A* | *N/A* |

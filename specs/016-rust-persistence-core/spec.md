# Feature Specification: Rust Persistence Core

**Feature Branch**: `016-rust-persistence-core`
**Created**: 2026-09-02
**Status**: Draft
**Reference**: `docs/estudo-migracao-rust-tauri.md` §AD-02, §AD-07, §AD-08, §10.4

## User Scenarios & Testing

### User Story 1 - Data survives restart (Priority: P1)

Curated data persists across application restarts, in a single file the user can copy as a backup.

**Why this priority**: Every later feature reads or writes through this layer. Nothing else can be built first.

**Independent Test**: Write a record through the storage layer, restart the process, read it back.

**Acceptance Scenarios**:

1. **Given** a fresh install, **When** the app starts, **Then** `hub.db` is created in the OS app-data directory with all 16 tables.
2. **Given** an existing `hub.db`, **When** the app starts again, **Then** existing rows are preserved and no migration re-runs.
3. **Given** the app has never run, **When** it starts, **Then** a default admin exists with username `admin@admin.com`.
4. **Given** an admin already exists, **When** the app restarts, **Then** no duplicate admin is created and the stored password hash is unchanged.

---

### User Story 2 - Entity metadata resolved from one place (Priority: P1)

Every entity is described once: its frontend route, its table name, its columns and its searchable column.

**Why this priority**: Rust has no runtime reflection, so the Python trick of iterating ORM metadata has no equivalent. Today this mapping is duplicated in three places (`postgres_adapter` dict, `link_service.column_map`, `EntityForm.getRoute`), which is how `groups` vs `research_groups` drift happens.

**Independent Test**: Resolve every known route and assert the returned table, columns and search column.

**Acceptance Scenarios**:

1. **Given** route `groups`, **When** resolved, **Then** the table is `research_groups`.
2. **Given** an unknown route, **When** resolved, **Then** a typed error is returned — not a panic, not a generic failure.
3. **Given** the registry, **When** compared against the SQL schema, **Then** every declared column exists in its table and every table column is declared.

---

### Edge Cases

- App-data directory missing on first run → create it.
- `hub.db` present but from an older schema → migration steps apply in order, tracked by `PRAGMA user_version`.
- `hub.db` corrupted or unreadable → fail with a clear typed error, never a panic that kills the window.
- Concurrent access from the ETL connection and the interactive connection → WAL plus `busy_timeout` must let both proceed.
- Explicit `id` supplied on insert (parquet import does this) → must be honored, not overwritten.

## Requirements

### Technical & Architectural Constraints

- **CON-001**: Single-process Tauri 2.0 desktop app, static Astro frontend embedded, works offline.
- **CON-002**: SQLite single file, embedded via `rusqlite` with feature `bundled`, stored in the OS app-data directory. No server-based engine.
- **CON-003**: All business logic in Rust. No SQL in JavaScript.
- **CON-004**: Full CRUD per entity, Admin-only. *(Exercised in SEP-017; this feature only provides the layer.)*
- **CON-005**: Parity before improvement. Schema mirrors the current Python schema except for the deliberate `universities` drop.
- **CON-006**: Linux and Windows only.

### Functional Requirements

- **FR-001**: The system MUST open a SQLite connection at `app_data_dir()/hub.db`, creating the directory if absent.
- **FR-002**: The system MUST enable WAL journal mode and a busy timeout.
- **FR-003**: The system MUST apply versioned migrations tracked by `PRAGMA user_version`, idempotently.
- **FR-004**: Migration `001_init` MUST create 16 tables: `admins` plus the 15 exported entities. Every column MUST be `TEXT` except `id INTEGER PRIMARY KEY`, mirroring `backend/src/models/orm.py`.
- **FR-005**: The schema MUST NOT include `universities` (orphan entity, dropped by decision Q5).
- **FR-006**: A static registry MUST describe each entity: frontend route, table name, ordered column list, searchable column, and whether it is exported.
- **FR-007**: Route lookup MUST return a typed error for unknown routes, closing the surface that today accepts any string and fails deep with a 500.
- **FR-008**: The registry MUST record which column names sort by `length()` rather than by value, matching the current Python behavior for JSON relationship columns.
- **FR-009**: The system MUST seed a default admin (`admin@admin.com` / `admin123`, bcrypt-hashed) exactly once, and MUST NOT overwrite an existing one.
- **FR-010**: The admin schema MUST be unified on `username` + `hashed_password`, resolving the divergence between `orm.py` and `scripts/seed.py` (bug 10.3.4). *(Declared correction, per CON-005.)*
- **FR-011**: Errors crossing the IPC boundary MUST be a serializable typed value carrying a human-readable message; panics MUST NOT be used for expected failures.
- **FR-012**: The connection and session state MUST be exposed as Tauri managed state, with the ETL path able to open its own connection so long operations never hold the interactive lock.

### Key Entities

- **EntityDef**: route, table, columns, search column, exported flag. One entry per entity; the single source of truth for route↔table mapping.
- **AppState**: the interactive database connection plus the (still empty) session slot.
- **AppError**: typed, serializable failure crossing the IPC boundary.

## Success Criteria

- **SC-001**: 16 tables created on first run; verified by querying the schema, not by inspection.
- **SC-002**: Registry and SQL schema agree on every column, both directions — a drift test fails loudly.
- **SC-003**: Restarting the app twice leaves exactly one admin row with an unchanged hash.
- **SC-004**: Every route in the registry resolves; an unknown route yields a typed error.
- **SC-005**: `cargo test` passes with no network and no pre-existing database file.
- **SC-006**: No SQL statement is issued from JavaScript.

## Assumptions

- Column lists are taken verbatim from `backend/src/models/orm.py`, which remains the schema of record until SEP-024.
- Storing everything as `TEXT` is intentional parity, not an oversight: original types live in the imported ZIP and are restored on export (SEP-019).
- The default admin credentials match the current system; hardening is out of scope.
- No data migration from an existing `backend/test.db` — the import flow (SEP-018) is the supported path. A migration note for existing users is SEP-024.

## Out of Scope

- IPC commands and CRUD operations (SEP-017).
- Parquet reading or writing (SEP-018, SEP-019).
- Real authentication and session enforcement (SEP-021); this feature only stores the admin row.

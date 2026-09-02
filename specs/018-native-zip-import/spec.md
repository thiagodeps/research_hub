# Feature Specification: Native ZIP Import

**Feature Branch**: `018-native-zip-import`
**Created**: 2026-09-02
**Status**: Draft
**Reference**: `docs/estudo-migracao-rust-tauri.md` §AD-06, §10.2, §10.4, §18 (Q1)

## User Scenarios & Testing

### User Story 1 - Load the DataLake archive (Priority: P1)

The curator picks `exports_canonical.zip` from disk — through a native file dialog or by dropping it on the window — and the 15 canonical tables are loaded, replacing whatever was there before.

**Why this priority**: Until data can get in, the application has nothing to curate. Every other feature operates on what this one loads.

**Independent Test**: Import the reference archive and compare per-table row counts against the Python importer over the same file.

**Acceptance Scenarios**:

1. **Given** the reference archive, **When** imported, **Then** each of the 15 tables holds exactly the row count the Python importer produces.
2. **Given** an archive whose parquet has columns the schema does not know, **When** imported, **Then** the unknown columns are dropped and the known ones load.
3. **Given** a previous import, **When** a new one runs, **Then** the old rows are gone — the archive replaces the base, it does not merge into it.
4. **Given** a successful import, **When** the archive is inspected afterwards, **Then** an untouched copy was kept for the export step.
5. **Given** a file that is not a ZIP, **When** chosen, **Then** the import is refused with a readable message and the database is untouched.

---

### User Story 2 - See the import happening (Priority: P2)

The curator watches progress table by table instead of staring at a frozen button.

**Why this priority**: The import wipes and repopulates everything; a silent multi-second freeze is indistinguishable from a hang. The current version shows only "Processando...".

**Independent Test**: Import while observing emitted progress events.

**Acceptance Scenarios**:

1. **Given** an import in progress, **When** each table finishes, **Then** an event reports the table and how many rows it loaded.
2. **Given** an import in progress, **When** it runs, **Then** the window stays responsive.

---

### User Story 3 - Survive a bad import (Priority: P2)

If an import fails or turns out to be wrong, the previous curated state is not lost.

**Why this priority**: The import destroys all curated work by design. Hours of manual correction can be erased by one wrong file, with no undo. This is the only feature in the migration that can lose user data.

**Independent Test**: Import, confirm a snapshot exists, corrupt the import, restore from the snapshot.

**Acceptance Scenarios**:

1. **Given** an existing database with curated rows, **When** an import starts, **Then** a snapshot is written before anything is deleted.
2. **Given** an import that fails midway, **When** it aborts, **Then** the database is left in its pre-import state, not half-loaded.

---

### Edge Cases

- Archive containing none of the 15 known tables → report zero rows loaded rather than silently succeeding.
- Parquet column present in the schema but absent from the file → column left NULL.
- Explicit `id` values in the parquet → preserved, since the JSON relationship arrays reference them.
- Archive larger than available memory → entries must be streamed, not fully materialized.
- Nested archive entry (`data_snapshot.zip`, 22.6 MB) → copied as an opaque entry, never recompressed.
- Same file imported twice in a row → same final state, not doubled rows.
- Dialog dismissed without choosing → no-op, no error.

## Requirements

### Technical & Architectural Constraints

- **CON-001**: Single-process Tauri desktop app, offline.
- **CON-002**: SQLite single file, embedded.
- **CON-003**: All business logic in Rust. The frontend may not read files; the dialog is opened from Rust (decision Q1).
- **CON-004**: Admin-only operation.
- **CON-005**: Parity first. Row counts and column selection must match the Python importer. Snapshotting is a declared addition.
- **CON-006**: Linux and Windows only.

### Functional Requirements

- **FR-001**: The system MUST open a native file dialog from Rust, filtered to `.zip`, and MUST treat dismissal as a no-op.
- **FR-002**: The system MUST accept a `.zip` dropped onto the window, following the same path as the dialog.
- **FR-003**: The system MUST reject non-ZIP input with a readable error before touching the database.
- **FR-004**: The system MUST read every `*.parquet` entry whose derived table name matches a registry table, deriving the name by stripping the directory and the `_canonical` suffix.
- **FR-005**: The system MUST load only columns declared in the registry, ignoring the rest, and MUST leave missing columns NULL.
- **FR-006**: The system MUST convert parquet values to text for storage, preserving the original values as written; original types are restored on export, not here.
- **FR-007**: The system MUST preserve explicit `id` values from the parquet.
- **FR-008**: The system MUST delete all rows of the 15 exported tables before loading, leaving `admins` untouched.
- **FR-009**: The whole import MUST be one transaction: either every table loads or the database is unchanged.
- **FR-010**: The system MUST store an untouched copy of the archive in the app-data directory for the export step.
- **FR-011**: The system MUST write a snapshot of the database before deleting anything. *(Declared addition; the Python version has no undo.)*
- **FR-012**: The system MUST emit a progress event per table with its name and row count.
- **FR-013**: The import MUST run off the interactive connection so the window stays responsive and the CRUD lock is never held for the duration.
- **FR-014**: The system MUST return a per-table summary of rows loaded.

### Key Entities

- **ImportSummary**: per-table row counts plus the snapshot location.
- **ImportProgress**: table name and row count, emitted as each table completes.

## Success Criteria

- **SC-001**: Row counts after import match the Python importer for all 15 tables on the reference archive.
- **SC-002**: `admins` survives an import with its password hash unchanged.
- **SC-003**: A failing import leaves the row counts exactly as they were.
- **SC-004**: A snapshot exists after any import that had prior data.
- **SC-005**: Importing the same archive twice yields identical row counts.
- **SC-006**: No file is read by JavaScript; the frontend declares no filesystem permission.

## Assumptions

- The reference archive `exports_canonical.zip` is the format of record.
- Table names derive from filenames; `research_groups_canonical.parquet` maps to the `research_groups` table, which the registry exposes on route `groups`.
- Storing values as text is deliberate parity with the current schema.
- Snapshots are kept as plain database copies; pruning old ones is out of scope.

## Out of Scope

- Export (SEP-019).
- Restoring from a snapshot through the interface — the file is written and its path reported; restoring is manual for now.
- Reading the non-canonical parquet files, graphs and marts; they are preserved for export but never parsed.

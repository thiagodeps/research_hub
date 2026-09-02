# Feature Specification: CRUD Commands and IPC Bridge

**Feature Branch**: `017-crud-commands-ipc-bridge`
**Created**: 2026-09-02
**Status**: Draft
**Reference**: `docs/estudo-migracao-rust-tauri.md` §AD-01, §8.5, §8.6, §10.3, §10.5

## User Scenarios & Testing

### User Story 1 - Browse and edit records (Priority: P1)

The curator opens an entity page, sees a paginated table, searches, sorts, and creates, edits or deletes records — exactly as in the web version, but with no server running.

**Why this priority**: This is the first feature where the desktop app does real work. It also proves the central bet of the migration: that replacing REST with IPC does not require rewriting the interface.

**Independent Test**: Seed rows directly into `hub.db`, open an entity page, and exercise list, search, sort, create, edit and delete.

**Acceptance Scenarios**:

1. **Given** 120 seeded records, **When** the page loads, **Then** 50 are shown and the counter reads the true total.
2. **Given** a search term, **When** it is typed, **Then** results filter on the entity's search column and pagination resets to the first page.
3. **Given** a sortable column, **When** its header is clicked twice, **Then** order flips ascending to descending, with nulls last in both.
4. **Given** a JSON relationship column, **When** sorted, **Then** rows order by number of links, not by serialized text.
5. **Given** a new record, **When** saved, **Then** it persists and appears in the list.
6. **Given** an existing record, **When** edited and saved, **Then** only the submitted fields change.
7. **Given** a record, **When** deleted, **Then** it disappears from the list without an error dialog.

---

### User Story 2 - Interface survives the transport change (Priority: P1)

The React components keep working without being touched, because only the module they all call through is rewritten.

**Why this priority**: The whole cost estimate of the migration rests on this. If components must be rewritten per entity, the project is several times larger than planned.

**Independent Test**: `git diff --stat frontend/src/components/` must be empty after the feature, while the application works.

**Acceptance Scenarios**:

1. **Given** the finished feature, **When** the component diff is inspected, **Then** no React component was modified.
2. **Given** a failing operation, **When** the error reaches the interface, **Then** it is an `Error` whose message is human-readable, matching what components already expect.
3. **Given** the running app, **When** network activity is inspected, **Then** no HTTP request is issued.

---

### Edge Cases

- Unknown entity in a call → typed error, no SQL executed.
- `sort` naming a column that does not exist → ignored, as today, not an error.
- `limit` outside 1..1000 or negative `offset` → clamped, as today.
- Search term containing `%` or `_` → treated as literal text, not as wildcards. *(The Python version leaks these into `LIKE`; declared correction.)*
- Update carrying an `id` different from the path id → path id wins, as today.
- Delete of a nonexistent record → not-found error, not a silent success.
- Empty payload on create → row created with defaults, as today.

## Requirements

### Technical & Architectural Constraints

- **CON-001**: Single-process Tauri desktop app, offline.
- **CON-002**: SQLite single file, embedded.
- **CON-003**: All business logic in Rust; no SQL in JavaScript; the JS↔Rust boundary stays in one frontend module.
- **CON-004**: Full CRUD per entity, Admin-only, no approval flow.
- **CON-005**: Parity first. Two behavior corrections are declared here: the delete error dialog (FR-013) and `LIKE` wildcard escaping (FR-006).
- **CON-006**: Linux and Windows only.

### Functional Requirements

- **FR-001**: The system MUST expose five commands — list, get, create, update, delete — generic over the entity registry.
- **FR-002**: List MUST return the page plus the unfiltered-by-page total, so the interface can render "showing X to Y of Z".
- **FR-003**: List MUST accept limit (default 100, clamped 1..1000), offset (clamped at 0), search, sort and order, matching the current endpoint.
- **FR-004**: Sorting MUST only accept columns declared in the registry; anything else is ignored rather than interpolated into SQL.
- **FR-005**: Sorting MUST place nulls last in both directions, and MUST order JSON relationship columns by `length()`.
- **FR-006**: Search MUST match case-insensitively on the entity's search column, and MUST escape `%` and `_` so a user searching for them finds literal text. *(Declared correction.)*
- **FR-007**: Entities without a search column MUST ignore the search term rather than fail.
- **FR-008**: All SQL MUST use bound parameters for values; identifiers MUST come from the registry, never from caller input.
- **FR-009**: Get and update MUST return the stored record; a missing record MUST produce a not-found error.
- **FR-010**: Update MUST merge the payload over the stored row, leaving unmentioned columns untouched, and MUST ignore keys that are not registry columns.
- **FR-011**: Create MUST honor an explicit `id` when supplied and otherwise let SQLite assign one.
- **FR-012**: `services/api.js` MUST be rewritten to dispatch the existing `apiFetch(endpoint, options)` calls to commands, preserving its observable contract: same arguments, parsed result, and `Error` with a readable message on failure.
- **FR-013**: A successful delete MUST NOT surface an error in the interface. *(Declared correction: `apiFetch` currently parses the body before checking status, and the 204 empty body makes every successful delete throw and skip the refresh.)*
- **FR-014**: No React component may be modified.
- **FR-015**: Values MUST cross the boundary as JSON, with SQL NULL becoming `null`, so the JSON-column rendering added in feature 013 keeps working.

### Key Entities

- **Page**: the records of one page plus the total matching count.
- **Record**: an untyped map of column to JSON value — the schema is all TEXT, so no per-entity struct is needed.

## Success Criteria

- **SC-001**: All five operations work on all 15 entities, verified by automated tests rather than by clicking.
- **SC-002**: `git diff --stat frontend/src/components/` is empty.
- **SC-003**: Pagination, search and sort produce the same results as the Python endpoint for the same database.
- **SC-004**: Deleting a record refreshes the table and shows no error dialog.
- **SC-005**: No HTTP request leaves the application.
- **SC-006**: A search for `100%` returns rows containing that text, not every row.

## Assumptions

- The database is populated out of band for now; the import flow arrives in SEP-018.
- Authentication is not yet enforced; commands run unauthenticated until SEP-021. This matches the current system, where no endpoint validates the token.
- Entity page column and field definitions stay in the `.astro` pages, unchanged.
- SQLite's `LIKE` remains ASCII-only case-insensitive, so accented search stays as imperfect as it is today (risk 10.5); improving it is a separate feature.

## Out of Scope

- Merge and link operations (SEP-020).
- Import and export (SEP-018, SEP-019).
- Login enforcement (SEP-021).
- Accent-insensitive search.

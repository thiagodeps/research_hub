# Feature Specification: Decommission the Python Backend

**Feature Branch**: `024-decommission-python-backend`
**Created**: 2026-09-02

## User Scenarios & Testing

### User Story 1 - One thing to run (Priority: P1)

A developer clones the repository and finds a single application, not a retired one alongside its replacement.

**Why this priority**: The Python backend was kept as a parity oracle. With import and export verified against the reference archive, it has no remaining job, and leaving it behind means two sources of truth and stale instructions.

**Acceptance Scenarios**:

1. **Given** the repository, **When** inspected, **Then** no Python backend and no two-process orchestration remain.
2. **Given** the README, **When** followed, **Then** the instructions describe the desktop application.
3. **Given** the removal, **When** the suite runs, **Then** everything still passes.

### Edge Cases

- The parity script referenced the oracle and must be rewritten, not left broken.
- Existing users have a `test.db` from the web version; they need a documented path.

## Requirements

- **FR-001**: Remove `backend/` and the loose scripts that supported it.
- **FR-002**: Rewrite the Makefile for the Tauri workflow.
- **FR-003**: Rewrite the README: installation, usage, data location, backup, SmartScreen note.
- **FR-004**: Rewrite AGENTS.md for the Rust architecture.
- **FR-005**: Rewrite the parity script around the round-trip test, since the oracle is gone.
- **FR-006**: Document the migration path for users of the web version.

## Success Criteria

- **SC-001**: No Python file remains in the repository.
- **SC-002**: `make test` passes.
- **SC-003**: The README contains no reference to `uvicorn`, `venv` or `STORAGE_TYPE`.
- **SC-004**: The full suite still passes after the removal.

## Assumptions

- Reimporting the canonical archive is the supported migration; the base was always derived from it.

## Out of Scope

- Automatic migration of an existing `test.db`.

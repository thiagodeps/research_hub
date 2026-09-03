# Feature Specification: Canonical Export Fidelity

**Feature Branch**: `019-canonical-export-fidelity`
**Created**: 2026-09-02
**Reference**: `docs/estudo-migracao-rust-tauri.md` §10.1, §10.2, §10.3

## User Scenarios & Testing

### User Story 1 - Deliver curated data back to the DataLake (Priority: P1)

The curator exports the corrected base as a canonical package the downstream pipelines accept without change.

**Why this priority**: This is the output of the entire tool. Everything before it is preparation.

**Acceptance Scenarios**:

1. **Given** an edited base, **When** exported, **Then** each of the 15 tables is written as parquet and JSON with the original column types and order.
2. **Given** a boolean column, **When** exported, **Then** it is boolean again, not the text SQLite stored.
3. **Given** an unparseable numeric value, **When** exported, **Then** it becomes null rather than aborting the export.
4. **Given** no original archive, **When** exported, **Then** everything is written as text and the export still succeeds.

### User Story 2 - Lose nothing the tool does not manage (Priority: P1)

Every file the system never reads comes back untouched.

**Why this priority**: The archive carries 596 entries the tool never parses — graphs, marts, reports, a PDF and a 22.6 MB nested archive. Dropping or recompressing any of them destroys DataLake data with no warning.

**Acceptance Scenarios**:

1. **Given** the reference archive, **When** exported, **Then** the package has the same 626 entries.
2. **Given** the nested `data_snapshot.zip`, **When** exported, **Then** it is byte-identical.

### Edge Cases

- Table failing mid-export → whole export fails loudly. *(Declared correction: today the exception is printed and the table silently vanishes from the package.)*
- Column in the database but not in the original → dropped, matching pandas.
- Empty table → valid empty parquet and `[]` JSON.
- JSON relationship column → emitted as structure, not escaped text.

## Requirements

- **FR-001**: Read the original archive kept by the import and use it as the record of column types and order.
- **FR-002**: Cast text back to the original type per column; unparseable values become null.
- **FR-003**: Write column order exactly as the original declares it, intersected with managed columns.
- **FR-004**: Emit `parquet/{table}_canonical.parquet` and `{table}_canonical.json` for the 15 exported tables; never for `admins`.
- **FR-005**: JSON MUST use 4-space indent, unescaped UTF-8, `null` for missing values, and parse JSON-looking strings into structure.
- **FR-006**: Copy every non-replaced entry from the original without recompressing it.
- **FR-007**: Open a native save dialog from Rust, defaulting to `portal_export_canonical.zip`; dismissal is a no-op.
- **FR-008**: A failing table MUST abort the export with a readable error. *(Declared correction.)*
- **FR-009**: Run off the interactive connection and emit per-table progress.

## Success Criteria

- **SC-001**: Round-tripping the reference archive yields 626 entries, 596 of them preserved.
- **SC-002**: All 15 tables come back with the original column types and order.
- **SC-003**: The nested 22.6 MB archive is byte-identical.
- **SC-004**: JSON matches the Python formatting rules.
- **SC-005**: Export works without an original archive.

## Assumptions

- Byte comparison of parquet files is not a valid criterion — compression and writer metadata differ between implementations. Schema plus values is the correct check.
- The original archive is present whenever an import has run.

## Out of Scope

- Selecting which tables to export.
- Recompressing or optimizing preserved entries.

# Feature Specification: Merge and Link Operations

**Feature Branch**: `020-merge-and-link-operations`
**Created**: 2026-09-02
**Reference**: `docs/estudo-migracao-rust-tauri.md` §10.3.1, §10.3.2

## User Scenarios & Testing

### User Story 1 - Link a child record to a parent (Priority: P1)

The curator attaches a research group to a researcher and the relationship is stored.

**Why this priority**: This feature has never worked. `LinkService.link_entities` calls `BaseRepository.save`, a method that does not exist, so every link with something to append raises AttributeError and returns a 500. The interface reports success only on the path that does nothing.

**Acceptance Scenarios**:

1. **Given** a researcher with no groups, **When** a group is linked, **Then** the relationship column holds it with its id and display name.
2. **Given** an existing link, **When** the same child is linked again, **Then** nothing is duplicated and no error is raised.
3. **Given** a relationship column holding invalid content, **When** a link is added, **Then** it is treated as an empty list rather than failing.
4. **Given** a parent without a matching column, **When** a link is attempted, **Then** a typed error explains which column is missing.

### User Story 2 - Merge duplicate records atomically (Priority: P2)

Duplicates collapse into one record, all or nothing.

**Why this priority**: The current merge saves the primary and then deletes the others in separate calls. A failure in between leaves orphans with no way to tell what happened.

**Acceptance Scenarios**:

1. **Given** three duplicates, **When** merged, **Then** the first holds the resolved data and the others are gone.
2. **Given** a failure during the merge, **When** it aborts, **Then** no record was modified or deleted.

### Edge Cases

- Route and column names differ (`groups` links into `research_groups`).
- A source record already deleted → merge completes anyway.
- Empty source list → typed error.

## Requirements

- **FR-001**: Link MUST append `{id, name}` to the parent's relationship column, deriving the label from the child's name or title.
- **FR-002**: Link MUST be idempotent and MUST NOT duplicate an existing child.
- **FR-003**: Link MUST treat null, empty or non-array content as an empty list.
- **FR-004**: Link MUST map the child route to the parent column, matching the existing `column_map`.
- **FR-005**: Link MUST return a typed error when parent, child or target column is missing.
- **FR-006**: Merge MUST run in a single transaction. *(Declared correction.)*
- **FR-007**: Merge MUST apply the resolved data to the first source and delete the rest.
- **FR-008**: Merge MUST tolerate a source that no longer exists.

## Success Criteria

- **SC-001**: Linking works — verified by a test that fails against the current Python behavior.
- **SC-002**: Linking twice leaves one entry.
- **SC-003**: A failed merge leaves row counts and values unchanged.
- **SC-004**: Merging three records leaves exactly one.

## Assumptions

- The relationship label follows the existing convention: child `name`, else `title`, else `"{type} {id}"`.
- Link only appends; removing a link is done by editing the record.

## Out of Scope

- Unlinking through a dedicated operation.
- Reciprocal links on the child side.

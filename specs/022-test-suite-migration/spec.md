# Feature Specification: Test Suite Migration

**Feature Branch**: `022-test-suite-migration`
**Created**: 2026-09-02

## User Scenarios & Testing

### User Story 1 - The suite tests the new system (Priority: P1)

Tests exercise the Rust core and the IPC bridge instead of a Python API and an HTTP client that no longer exist.

**Why this priority**: Constitution Principle I requires TDD, and Principle III names the tools. A suite pointing at the retired stack gives false confidence.

**Acceptance Scenarios**:

1. **Given** the frontend suite, **When** it runs, **Then** it mocks the IPC bridge rather than `fetch`.
2. **Given** the end-to-end suite, **When** it runs, **Then** it drives the real bundled application.

### Edge Cases

- Playwright cannot drive a Tauri webview; it must be replaced, not reconfigured.
- A stale binary would silently test old code, so the E2E run rebuilds first.

## Requirements

- **FR-001**: Frontend unit tests MUST mock `@tauri-apps/api/core` and assert command names and arguments.
- **FR-002**: The delete case MUST be covered, since it regressed in the old transport.
- **FR-003**: Route ordering MUST be covered, so `/link` is never read as a create.
- **FR-004**: The error contract MUST be covered: components receive an `Error` with a readable message.
- **FR-005**: Playwright MUST be removed and replaced by WebdriverIO with `tauri-driver`.
- **FR-006**: The E2E config MUST rebuild the application before running.
- **FR-007**: A parity script MUST run import/export on both implementations over the same archive and compare structurally, never byte for byte.

## Success Criteria

- **SC-001**: `npm test` passes with no network and no backend running.
- **SC-002**: `cargo test` covers persistence, CRUD, import, export, merge, link and session.
- **SC-003**: The parity script reports the same entry set and schemas for both implementations.

## Assumptions

- E2E runs on Linux and Windows, which is every release target.
- The Python backend remains available as the oracle until SEP-024.

## Out of Scope

- Coverage thresholds and CI gating (SEP-023).

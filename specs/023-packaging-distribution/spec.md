# Feature Specification: Packaging and Distribution

**Feature Branch**: `023-packaging-distribution`
**Created**: 2026-09-02

## User Scenarios & Testing

### User Story 1 - Install on a clean machine (Priority: P1)

The curator installs the tool on a machine with no Python, no Node and no toolchain, and it runs.

**Why this priority**: This is the payoff of the migration — the reason for leaving a two-process web stack.

**Acceptance Scenarios**:

1. **Given** a clean Linux machine, **When** the package is installed, **Then** the application launches with no other software present.
2. **Given** a clean Windows machine, **When** the installer runs, **Then** the application launches.
3. **Given** an unsigned build on Windows, **When** it is first launched, **Then** the documented SmartScreen path lets it through.

### Edge Cases

- Tauri has no reliable cross-compilation, so each OS needs its own runner.
- A bundle built from untested code would ship regressions; bundling depends on the test job.

## Requirements

- **FR-001**: Produce `.deb`, `.rpm` and `.AppImage` on Linux and an NSIS installer on Windows.
- **FR-002**: The pipeline MUST run one runner per target OS.
- **FR-003**: Bundling MUST depend on the test job passing.
- **FR-004**: No code signing and no updater. *(Decisions Q3, Q6.)*
- **FR-005**: macOS MUST NOT be a target. *(Decision Q2; Principle III forbids a release target without automated tests.)*
- **FR-006**: The README MUST document the SmartScreen prompt.
- **FR-007**: Releases MUST be created as drafts, so a human decides what is published.

## Success Criteria

- **SC-001**: A tagged push produces installers for both platforms.
- **SC-002**: Installing on a clean machine needs no Python, Node or toolchain.
- **SC-003**: A failing test blocks the bundle.

## Assumptions

- GitHub Actions is the pipeline; runners are available for both targets.
- Icons remain the Tauri defaults until branding assets exist.

## Out of Scope

- macOS, code signing, notarization, auto-update, app store distribution.

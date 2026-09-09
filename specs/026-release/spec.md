# Feature Specification: Release Actions

**Feature Branch**: `026-release`  
**Created**: 2026-09-09  
**Status**: Draft  

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create multi-platform releases (Priority: P1)

As a developer, I want to automatically build and release executables for Windows, macOS, and Linux when pushing a new version tag, so that users on any desktop platform can easily install the application without manually compiling it.

**Why this priority**: It is the core goal of the feature request, explicitly addressing the need for Windows, macOS, and Linux executables.

**Independent Test**: Push a new tag `v1.0.0` and verify that a draft GitHub Release is created containing `.dmg`, `.app` (macOS), `.msi`, `.exe` (Windows), and `.deb`, `.rpm`, `.AppImage` (Linux) files.

**Acceptance Scenarios**:

1. **Given** code is pushed to the repository, **When** a tag matching `v*.*.*` is created, **Then** the `Release` GitHub Action workflow triggers and builds installers for Windows, macOS, and Linux.
2. **Given** a manual workflow dispatch is triggered, **When** the workflow runs, **Then** it builds and uploads installers as GitHub Actions artifacts for Windows, macOS, and Linux without creating a GitHub release.

## Requirements *(mandatory)*

### Technical & Architectural Constraints

- **CON-001**: A aplicação DEVE ser um app desktop de processo único em Tauri 2.0.
- **CON-002**: A persistência DEVE usar SQLite em arquivo único.
- **CON-003**: Toda regra de negócio DEVE residir no processo Rust.
- **CON-004**: Todas as entidades do domínio exigem CRUD completo operado unicamente por perfil de Admin.
- **CON-005**: Paridade funcional precede melhoria.
- **CON-006**: *OVERRIDE*: A pedido do usuário, a plataforma macOS está sendo adicionada aos builds de release, apesar da restrição original.

### Functional Requirements

- **FR-001**: The system MUST define a GitHub Actions workflow named `Release` triggered by tag pushes (`v*.*.*`) or manual dispatch.
- **FR-002**: The workflow MUST use a matrix strategy to build on `ubuntu-latest`, `macos-latest`, and `windows-latest`.
- **FR-003**: For Linux builds, system dependencies (`libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, etc.) MUST be installed prior to building.
- **FR-004**: The workflow MUST use Node.js and Rust toolchains to compile the Astro frontend and Rust backend.
- **FR-005**: The workflow MUST use `tauri-apps/tauri-action` to build the application and automatically create a draft GitHub release if triggered by a tag.
- **FR-006**: The workflow MUST upload the built executables as artifacts on manual dispatch runs.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A single GitHub Actions run successfully compiles the Tauri app for Linux, Windows, and macOS.
- **SC-002**: The generated release contains artifacts for all three platforms.

## Assumptions

- Users understand that the binaries are unsigned and may trigger warnings on Windows (SmartScreen) or macOS (Gatekeeper).
- Existing test workflows run in separate jobs or workflows; this specific workflow focuses purely on building and releasing.

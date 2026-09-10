# Implementation Plan: Table Page Navigation

**Branch**: `028-table-page-nav` | **Date**: 2026-09-10 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/028-table-page-nav/spec.md`

## Summary

This feature adds the ability for users to navigate directly to a specific page number in any paginated table. The implementation will update the `EntityPage.jsx` component to include a text input field for the page number between the "Anterior" and "Próxima" buttons, with bounds checking to ensure the requested page is valid.

## Technical Context

**Language/Version**: JavaScript (React)
**Primary Dependencies**: React (useState, useEffect), Tailwind CSS
**Storage**: N/A
**Testing**: Vitest (for frontend components, if any exist)
**Target Platform**: Desktop App (Tauri 2.0 / Astro)
**Project Type**: desktop-app / frontend modification
**Performance Goals**: N/A
**Constraints**: Must gracefully handle out of bounds inputs
**Scale/Scope**: Impacts all entity tables rendered by `EntityPage.jsx`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] I — TDD Adherence: Test planning is prioritized and tasks clearly follow the Red-Green-Refactor cycle.
- [x] II — Functional Parity: No new functionality precedes verified import→edit→export parity. Any fix to
      pre-existing behavior is declared as explicit scope and registered as an expected diff against the
      Python oracle. No silent behavior changes.
- [x] III — Test Strategy: Rust core specifies `cargo test` (unit for domain logic, integration against
      in-memory SQLite); frontend specifies Vitest (components/IPC layer) and WebdriverIO + `tauri-driver`
      (E2E). Every release target is covered by automated tests — no platform is claimed on manual scripts.
- [x] IV — Operations: Direct CRUD operations by Admin only, no approval flows.
- [x] V — Desktop Architecture: Single-process Tauri 2.0 app; Astro frontend built statically and embedded;
      SQLite single-file storage in the OS app-data directory; no HTTP layer, no database server; works
      fully offline. Identity/branding strictly follows Figma specs.
- [x] VI — Business Logic in Rust: No SQL, no file I/O, and no duplicated business rules in JavaScript.
      The JS↔Rust boundary stays confined to a single frontend module.
- [x] Deferred Scope: The plan does not reintroduce macOS support, code signing, auto-update, or any
      non-SQLite database engine (see Constitution §Escopo Diferido).

## Project Structure

### Documentation (this feature)

```text
specs/028-table-page-nav/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (to be generated)
```

### Source Code (repository root)

```text
frontend/
├── src/
│   ├── components/
│   │   └── EntityPage.jsx
```

**Structure Decision**: The feature is a simple UI change in the React component `frontend/src/components/EntityPage.jsx`.

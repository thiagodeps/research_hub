# Tasks: Table Page Navigation

**Input**: Design documents from `/specs/028-table-page-nav/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md

**Tests**: TDD is OBRIGATÓRIO (MANDATORY). All features must follow the Test-First approach (red-green-refactor). No feature can be implemented without a prior failing test.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

*(No shared setup tasks required for this feature, as it is a minor UI addition)*

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

*(No foundational tasks required for this feature)*

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Navigate to a specific page number (Priority: P1) 🎯 MVP

**Goal**: As a user viewing a paginated table, I want to be able to jump to a specific page number directly, instead of repeatedly clicking "next" or "previous", so that I can quickly reach the data I am looking for.

**Independent Test**: Can be fully tested by opening any paginated table with multiple pages, entering a valid page number in the pagination control, and verifying the table updates to display that exact page.

### Tests for User Story 1 (MANDATORY - Write tests first) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T001 [P] [US1] Create unit tests for page number input navigation in `frontend/tests/unit/EntityPage.test.jsx` asserting valid input updates the page, and invalid input is clamped or ignored.

### Implementation for User Story 1

- [x] T002 [US1] Modify `frontend/src/components/EntityPage.jsx` to add a number `<input>` between the "Anterior" and "Próxima" buttons showing the current page number (1-indexed).
- [x] T003 [US1] Implement event handler in `frontend/src/components/EntityPage.jsx` to handle the input change/blur/Enter key and update the `page` state based on valid inputs (must clamp between 1 and the max page calculated from `total` and `limit`).

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T004 Adjust styling of the new input field in `frontend/src/components/EntityPage.jsx` to match the existing Tailwind CSS aesthetic.
- [x] T005 Review edge cases manually in the application to ensure out-of-bounds page entries behave correctly.

---

## Dependencies & Execution Order

### User Story Dependencies

- **User Story 1 (P1)**: Can start immediately, no dependencies.

### Within Each User Story

- Tests MUST be written and FAIL before implementation (T001 must be done before T002).
- T002 and T003 can be implemented sequentially to fulfill the test.

### Parallel Opportunities

- T001 can be started independently.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (Tests)
2. Complete T002 and T003 (Implementation)
3. Complete T004 and T005 (Polish)
4. **STOP and VALIDATE**: Test User Story 1 independently in the browser.

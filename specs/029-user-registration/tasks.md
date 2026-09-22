# Tasks: Account Creation and Password Hashing

**Input**: Design documents from `/specs/029-user-registration/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md, quickstart.md  

**Tests**: TDD is OBRIGATÓRIO (MANDATORY). All features must follow the Test-First approach (red-green-refactor). No feature can be implemented without a prior failing test.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Exact file paths are specified for every task

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, dependency verification, and shared configuration

- [X] T001 Verify and ensure dependencies for bcrypt in `src-tauri/Cargo.toml` and testing tools in `frontend/package.json`
- [X] T002 [P] Register `/register` route in routing verification tests in `src-tauri/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core error types and IPC route mappings that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: Foundational tasks must complete before user story implementation begins

- [X] T003 Define error variant `Validation(String)` and serialization handling in `src-tauri/src/error.rs`
- [X] T004 [P] Wire IPC route `POST /auth/register` in centralized bridge `frontend/src/services/api.js`

**Checkpoint**: Core types and IPC routing defined - User Story implementation can begin.

---

## Phase 3: User Story 1 - Create a new account with email and password (Priority: P1) 🎯 MVP

**Goal**: Allow users to submit their email and password on a dedicated registration page, securely validate and hash the password with bcrypt, persist the account in SQLite `admins` table, and redirect to the login screen.

**Independent Test**: Register a new account via `/register`, verify that the `admins` table contains the new record with bcrypt hash (`$2b$12$...`) and zero plaintext passwords, and verify redirection to `/login?registered=true`.

### Tests for User Story 1 (MANDATORY - Write tests first) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T005 [P] [US1] Write failing unit tests for account registration and password hashing in `src-tauri/src/auth.rs` (covering valid registration with bcrypt hash, email trimming and lowercasing, duplicate email rejection, short password rejection, password mismatch rejection, and invalid email format rejection)
- [X] T006 [P] [US1] Write failing unit tests for `RegisterForm.jsx` in `frontend/tests/unit/RegisterForm.test.jsx` (covering form field rendering, validation error display, and form submission via `apiFetch('/auth/register')`)

### Implementation for User Story 1

- [X] T007 [US1] Implement `validate_registration` and `register_admin` with bcrypt hashing and SQLite persistence in `src-tauri/src/auth.rs` (depends on T005)
- [X] T008 [US1] Implement `register` command in `src-tauri/src/commands.rs` and register it in `tauri::generate_handler!` in `src-tauri/src/lib.rs` (depends on T007)
- [X] T009 [US1] Implement `RegisterForm.jsx` component with input validation, error handling, and redirection in `frontend/src/components/RegisterForm.jsx` (depends on T006)
- [X] T010 [US1] Create the registration page using `SplitScreen.astro` layout in `frontend/src/pages/register.astro` (depends on T009)

**Checkpoint**: At this point, User Story 1 is fully functional and testable independently. An account can be registered, hashed, and stored in the database.

---

## Phase 4: User Story 2 - Authenticate on initial login screen with new account (Priority: P2)

**Goal**: Enable users to log in on the initial login screen using credentials created through the registration page.

**Independent Test**: Register a new account, navigate to `/login`, enter the registered credentials, and verify successful authentication and redirection to `/dashboard`.

### Tests for User Story 2 (MANDATORY - Write tests first) ⚠️

- [X] T011 [P] [US2] Write integration test verifying that a newly registered account successfully authenticates with `verify_credentials` in `src-tauri/src/auth.rs`
- [X] T012 [P] [US2] Write frontend test verifying that entering newly registered credentials on `LoginForm.jsx` successfully calls `/auth/login` and redirects to `/dashboard` in `frontend/tests/unit/LoginForm.test.jsx`

### Implementation for User Story 2

- [X] T013 [US2] Ensure email normalization (lowercased and trimmed) is enforced during login verification in `src-tauri/src/auth.rs` (depends on T011)
- [X] T014 [US2] Ensure `LoginForm.jsx` trims email input and handles authentication errors gracefully in `frontend/src/components/LoginForm.jsx` (depends on T012)

**Checkpoint**: At this point, User Stories 1 AND 2 work seamlessly together: users can register an account and immediately log in with it on the initial login screen.

---

## Phase 5: User Story 3 - Navigation between Login and Registration pages (Priority: P3)

**Goal**: Provide clear navigation links between login and registration, and display a confirmation success alert on `/login` after successful registration.

**Independent Test**: Navigate back and forth between `/login` and `/register` using UI links, and verify the green success banner appears on `/login` when accessed via `?registered=true`.

### Tests for User Story 3 (MANDATORY - Write tests first) ⚠️

- [X] T015 [P] [US3] Write frontend test verifying navigation links and the `?registered=true` success notification banner in `frontend/tests/unit/LoginForm.test.jsx`

### Implementation for User Story 3

- [X] T016 [US3] Add "Não tem uma conta? Cadastre-se" link and success alert for `?registered=true` query parameter in `frontend/src/components/LoginForm.jsx` (depends on T015)
- [X] T017 [US3] Add "Já tem uma conta? Entrar" link pointing to `/login` in `frontend/src/components/RegisterForm.jsx`

**Checkpoint**: All user stories are complete, connected, and independently functional.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: End-to-end verification, security checks, and code quality

- [X] T018 [P] Execute automated test suite and manual verification checklist defined in `specs/029-user-registration/quickstart.md`
- [X] T019 Audit database and persistence code to guarantee zero plaintext password leakage in `src-tauri/src/db.rs` and `src-tauri/src/auth.rs`
- [X] T020 Run code formatting and lint checks across Rust in `src-tauri/` and JavaScript in `frontend/`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational completion.
- **User Story 2 (Phase 4)**: Depends on User Story 1 completion (requires user account to exist).
- **User Story 3 (Phase 5)**: Depends on User Story 1 & 2 completion (connects the screens and states).
- **Polish (Phase 6)**: Depends on all user stories being complete.

### Within Each User Story

1. Write tests first and verify they FAIL (Red).
2. Implement backend domain / service logic (Green).
3. Implement IPC commands and frontend bridge.
4. Implement UI components and pages.
5. Refactor and verify all tests pass.

### Parallel Opportunities

- **Setup**: `T002` can run in parallel with `T001`.
- **Foundational**: `T004` can run in parallel with `T003`.
- **User Story 1 Tests**: `T005` (Rust unit test) and `T006` (Frontend component test) can run in parallel.
- **User Story 2 Tests**: `T011` (Rust auth test) and `T012` (Frontend login test) can run in parallel.
- **Polish**: `T018` can run in parallel with `T019`.

---

## Parallel Example: User Story 1

```bash
# Launch tests for User Story 1 in parallel:
Task T005: "Write failing unit tests for account registration and password hashing in src-tauri/src/auth.rs"
Task T006: "Write failing unit tests for RegisterForm.jsx in frontend/tests/unit/RegisterForm.test.jsx"

# Launch backend and frontend implementations:
Task T007: "Implement validate_registration and register_admin in src-tauri/src/auth.rs"
Task T009: "Implement RegisterForm.jsx in frontend/src/components/RegisterForm.jsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (`T001`, `T002`)
2. Complete Phase 2: Foundational (`T003`, `T004`)
3. Complete Phase 3: User Story 1 (`T005` to `T010`)
4. **STOP and VALIDATE**: Test User Story 1 independently by registering an account and inspecting the database for bcrypt hash.
5. Demonstrate MVP!

### Incremental Delivery

1. Complete Setup + Foundational → Core types and routes ready.
2. Deliver User Story 1 → Users can create accounts with bcrypt hashes.
3. Deliver User Story 2 → Users can log in on initial page with new accounts.
4. Deliver User Story 3 → Visual navigation and confirmation feedback connected.
5. Run Polish Phase → Full test coverage, zero plaintext guarantee verified.

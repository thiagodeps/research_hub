---
description: "Task list for Portal Admin e Gestão de Pesquisa implementation"
---

# Tasks: Portal Admin e Gestão de Pesquisa

**Input**: Design documents from `specs/001-admin-portal/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md

**Tests**: TDD is OBRIGATÓRIO (MANDATORY). All features must follow the Test-First approach (red-green-refactor). No feature can be implemented without a prior failing test.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create monorepo base structure `backend/` and `frontend/`
- [x] T002 [P] Initialize FastAPI project and dependencies in `backend/requirements.txt`
- [x] T003 [P] Initialize Astro project and dependencies in `frontend/package.json`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented
**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 [P] Write integration tests for database connection in `backend/tests/integration/test_db_config.py`
- [x] T005 Implement database connection and memory/postgres adapter in `backend/src/database/core.py`
- [x] T006 [P] Write tests for API fetch client in `frontend/tests/unit/api.test.js`
- [x] T007 Implement frontend API client wrapping fetch in `frontend/src/services/api.js`
- [x] T008 Configure error handling middleware in `backend/src/core/exceptions.py`

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Autenticação do Admin (Priority: P1) 🎯 MVP

**Goal**: Permitir login de Administrador usando e-mail e senha com tela split-screen.
**Independent Test**: Interface renderiza layout dividido; admin consegue logar e receber JWT.

### Tests for User Story 1 (MANDATORY - Write tests first) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T009 [P] [US1] Write contract test for POST `/api/v1/auth/login` in `backend/tests/contract/test_auth.py`
- [x] T010 [P] [US1] Write unit tests for JWT generation in `backend/tests/unit/test_security.py`
- [x] T011 [P] [US1] Write e2e test for login form flow in `frontend/tests/e2e/login.spec.js`

### Implementation for User Story 1

- [x] T012 [P] [US1] Implement Admin User schema in `backend/src/models/admin.py`
- [x] T013 [US1] Implement JWT security module in `backend/src/core/security.py`
- [x] T014 [US1] Implement Auth Service (password validation) in `backend/src/services/auth_service.py`
- [x] T015 [US1] Implement login endpoint router in `backend/src/api/auth.py`
- [x] T016 [P] [US1] Create Split-Screen Layout in `frontend/src/layouts/SplitScreen.astro`
- [x] T017 [US1] Implement LoginForm component in `frontend/src/components/LoginForm.jsx`
- [x] T018 [US1] Implement login page integrating layout and form in `frontend/src/pages/login.astro`

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Gestão de Dados (CRUD Completo) (Priority: P1)

**Goal**: Permitir listar, ver detalhe, criar, editar e deletar entidades.
**Independent Test**: Navegar no dashboard, visualizar tabela de pesquisadores (Researchers) e criar novo registro.

### Tests for User Story 2 (MANDATORY - Write tests first) ⚠️

- [x] T019 [P] [US2] Write integration tests for generic CRUD endpoints in `backend/tests/integration/test_crud_endpoints.py`
- [x] T020 [P] [US2] Write unit tests for Reusable Table/Form in `frontend/tests/unit/components.test.jsx`

### Implementation for User Story 2

- [x] T021 [P] [US2] Create base repositories for `research_domain` in `backend/src/database/repositories.py`
- [x] T022 [US2] Implement CRUD services mapping to domain in `backend/src/services/crud_service.py`
- [x] T023 [US2] Implement generic CRUD REST endpoints router in `backend/src/api/crud.py`
- [x] T024 [P] [US2] Create Reusable EntityTable in `frontend/src/components/EntityTable.jsx`
- [x] T025 [P] [US2] Create Reusable EntityForm in `frontend/src/components/EntityForm.jsx`
- [x] T026 [US2] Implement Dashboard base layout in `frontend/src/layouts/Dashboard.astro`
- [x] T027 [US2] Implement list/edit pages para entidades em `frontend/src/pages/dashboard/[entity].astro`

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Operações Avançadas (Fusão e Herança) (Priority: P2)

**Goal**: Realizar Fusão (merge com soft delete) e Herança (link) via UI.
**Independent Test**: Selecionar dois pesquisadores e realizar merge com sucesso.

### Tests for User Story 3 (MANDATORY - Write tests first) ⚠️

- [x] T028 [P] [US3] Write integration test for POST `/api/v1/merge/{entity_type}` in `backend/tests/integration/test_merge.py`
- [x] T029 [P] [US3] Write integration test for POST `/api/v1/link` in `backend/tests/integration/test_link.py`

### Implementation for User Story 3

- [x] T030 [P] [US3] Implement Merge Service com soft delete logic em `backend/src/services/merge_service.py`
- [x] T031 [P] [US3] Implement Link Service em `backend/src/services/link_service.py`
- [x] T032 [US3] Implement endpoints for Merge and Link em `backend/src/api/special_ops.py`
- [x] T033 [US3] Create MergeModal component em `frontend/src/components/MergeModal.jsx`
- [x] T034 [US3] Create LinkModal component em `frontend/src/components/LinkModal.jsx`
- [x] T035 [US3] Integrate modals into generic EntityTable em `frontend/src/components/EntityTable.jsx`

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user### Validation & Polish

- [x] T036 [P] Documentation updates in `quickstart.md` para comandos de DB
- [x] T037 [P] Code cleanup e padronização de logs no FastAPI em `backend/src/core/logging.py`
- [x] T038 Optimize frontend build configuration in `frontend/astro.config.mjs`

---

## Dependencies & Execution Order

### Phase Dependencies
- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel ou sequentially in priority order.
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### Parallel Opportunities
- Todos os testes unitários/contrato (marcados com `[P]`) podem ser iniciados em paralelo por diferentes devs.
- T016 (Layout Frontend) e T012 (Modelagem Backend) podem ocorrer de forma assíncrona.
- Os modais de frontend da Phase 5 (T033, T034) podem ser desenvolvidos de forma independente dos serviços de backend correspondentes.

## Implementation Strategy

### MVP First (User Story 1 Only)
1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. O admin é capaz de ver a página estática e logar no sistema!

### Incremental Delivery
1. Add User Story 2 → CRUD table working for Universities
2. Add User Story 3 → Admin curates data (Merge/Link)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Verify tests fail before implementing (Strict TDD mandatory).
- Commit after each task or logical group.

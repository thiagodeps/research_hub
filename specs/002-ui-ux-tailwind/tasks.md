# Feature Implementation Tasks: Refinamento de UI/UX com Tailwind CSS

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Instalar dependências do Tailwind CSS no projeto rodando `npx astro add tailwind -y` em `frontend/`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T002 Configurar cores e fontes base da marca (Theme) no arquivo `frontend/tailwind.config.mjs`
- [ ] T003 Configurar layout base com reset/estilos globais aplicados ao `<body>` em `frontend/src/layouts/SplitScreen.astro`
- [ ] T004 Configurar layout base com reset/estilos globais aplicados ao `<body>` em `frontend/src/layouts/Dashboard.astro`

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Aprimoramento da Tela de Login (Priority: P1) 🎯 MVP

**Goal**: Login com design split-screen moderno e formulário refinado.

**Independent Test**: Pode ser testado visualmente acessando a rota `/login` no frontend para validar responsividade (stack no mobile, split no desktop).

### Tests for User Story 1 (MANDATORY - Write tests first) ⚠️

- [ ] T005 [P] [US1] Atualizar E2E tests em `frontend/tests/e2e/login.spec.js` para garantir que classes chave de visibilidade (Tailwind) estão presentes e o layout não quebrou

### Implementation for User Story 1

- [ ] T006 [US1] Remover estilos inline e implementar layout split-screen (grid/flex) em `frontend/src/layouts/SplitScreen.astro` usando classes Tailwind
- [ ] T007 [US1] Remover estilos inline e implementar formulário estilizado em `frontend/src/components/LoginForm.jsx` (inputs com hover/focus e botões interativos)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Dashboard e Tabelas de Entidades (Priority: P2)

**Goal**: Listagem de entidades (CRUD) em tabelas polidas, bem espaçadas e com ações claras.

**Independent Test**: Visualizar `/dashboard/universities` garantindo hover nas linhas e legibilidade.

### Tests for User Story 2 (MANDATORY - Write tests first) ⚠️

- [ ] T008 [P] [US2] Atualizar testes de componente em `frontend/tests/unit/components.test.jsx` para validar renderização correta da tabela sem quebras estruturais.

### Implementation for User Story 2

- [ ] T009 [US2] Remover estilos inline de `frontend/src/components/EntityTable.jsx` e estilizá-la com Tailwind (borders suaves, padding nas células, thead em destaque, hover nas linhas `hover:bg-gray-50`)
- [ ] T010 [US2] Substituir botões de "Ações" crus na `EntityTable.jsx` por botões padronizados (ou ícones) usando Tailwind
- [ ] T011 [US2] Aplicar espaçamento e grid estrutural ao layout da página `frontend/src/layouts/Dashboard.astro` para acomodar a tabela centralizada

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Modais Modernos (Priority: P3)

**Goal**: Modais sobrepostos polidos (overlays) para Fusão e Link.

**Independent Test**: Testar visualmente os overlays nas ações "Vincular" e "Fundir".

### Tests for User Story 3 (MANDATORY - Write tests first) ⚠️

- [ ] T012 [P] [US3] Atualizar/Adicionar testes para validar se os modais estão sendo renderizados corretamente após refatoração em `frontend/tests/unit/components.test.jsx`

### Implementation for User Story 3

- [ ] T013 [P] [US3] Refatorar `frontend/src/components/MergeModal.jsx` removendo estilos inline e aplicando backdrop escuro fixo (`fixed inset-0 bg-black/50`) e card centralizado
- [ ] T014 [P] [US3] Refatorar `frontend/src/components/LinkModal.jsx` removendo estilos inline e aplicando backdrop escuro fixo e card centralizado

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T015 [P] Otimização das classes (remover estilos não utilizados) no build final rodando `npm run build`
- [ ] T016 [P] Validação com Lighthouse (Score > 90) em acessibilidade nas rotas principais
- [ ] T017 [P] Revisar `frontend/src/components/EntityPage.jsx` para garantir que o formulário genérico de criação siga o design padronizado

---

## Dependencies & Execution Order

- **Phase 1 & 2**: Sequencial. Instalação e temas base bloqueiam a implementação das telas.
- **Phase 3, 4, 5**: As telas e componentes (Login, Tabela, Modais) podem ser estilizadas em paralelo pois não dependem umas das outras.
- Os testes e refatoração de código seguem a regra TDD de refatoramento visual: garante que os componentes continuam na tela (ex: o formulário envia o callback `onLogin`) mas que os elementos visuais inline não existem mais.

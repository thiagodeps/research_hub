---

description: "Task list for feature implementation"
---

# Tasks: Área de Dados do Projeto SRC com Seleção de Projeto

**Input**: Design documents from `/specs/033-src-data-tab/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/ (src-consolidated-json.md, ipc-commands.md, frontend-routes.md), quickstart.md

**Tests**: INCLUÍDOS E OBRIGATÓRIOS — a Constituição (Princípio I: TDD Obrigatório; Princípio III) exige teste primeiro para toda funcionalidade. Cada fase começa pelos testes (vermelho) e só então implementa (verde).

**Organization**: Tasks agrupadas por user story (US1 seleção de projeto, US2 import/export do consolidado, US3 curadoria do SRC) para implementação e validação independentes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Núcleo Rust: `src-tauri/src/` (+ fixtures em `src-tauri/tests/fixtures/`, migrações em `src-tauri/migrations/`)
- Front-end: `frontend/src/` (testes em `frontend/tests/unit/` e `frontend/tests/e2e/`)
- Contratos de referência: `specs/033-src-data-tab/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Base material e dependências para tudo que vem depois

- [x] T001 Criar fixture do consolidado em `src-tauri/tests/fixtures/src_consolidado_exemplo.json` seguindo `specs/033-src-data-tab/contracts/src-consolidated-json.md`: 3 ações — (1) com participações de ambos os tipos (`"Público-alvo"` com `"Nome"`, `"CPF"`, `"E-mail"`, `"Situação"`; `"Equipe de execução"` com `"Nome"`, `"Função"`, `"Vínculo"`), (2) sem participações (`"total_participacoes": 0`, `"participacoes": []`), (3) com chave de rótulo extra não canônica e um campo ausente; ação 3 com `"Ação vinculante"` apontando para o processo da ação 1; contadores da raiz coerentes com o conteúdo (C8); texto com acentuação para o teste de serialização (C9)
- [x] T002 [P] Adicionar feature `preserve_order` ao `serde_json` em `src-tauri/Cargo.toml` (contrato C9: ordem de chaves dos objetos originais preservada no export)
- [x] T003 Rodar `cargo test` e `npm test` e confirmar baseline verde na branch antes de qualquer mudança

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Esquema, metadados e infraestrutura que TODAS as user stories exigem

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 [P] Escrever teste (vermelho) em `src-tauri/src/db.rs` (`#[cfg(test)]`): após `db::initialize`, existem `src_meta` (1 linha: `campus` NULL, `imported_at` NOT NULL), `src_acoes` (`id` INTEGER PK AUTOINCREMENT; `acao_id` TEXT NOT NULL UNIQUE; `raw_json` TEXT NOT NULL; `processo`, `titulo`, `natureza`, `tipo`, `coordenador`, `acao_vinculante`, `campus` TEXT NULL; `total_participacoes` INTEGER NOT NULL DEFAULT 0) e `src_participacoes` (`id` PK; `acao_row_id` INTEGER NOT NULL REFERENCES src_acoes(id) ON DELETE CASCADE; `ord` INTEGER NOT NULL; `tipo` TEXT NOT NULL; `atividade_num`, `atividade_id`, `atividade`, `nome` TEXT NULL; `raw_json` TEXT NOT NULL)
- [x] T005 [P] Escrever teste (vermelho) em `src-tauri/src/registry.rs`: `by_route("src_acoes")` é **rejeitado** no caminho genérico (FR-004/FR-005 — comandos do Horizon não enxergam SRC) e a nova seção `SRC_ENTITIES` resolve `src_acoes` (tabela `src_acoes`, busca por `titulo`, `exported: false`)
- [x] T006 Criar `src-tauri/migrations/002_src_tables.sql` com as 3 tabelas acima (constraints verbatim do data-model.md: `acao_id` NOT NULL + UNIQUE, `raw_json` NOT NULL, `tipo` NOT NULL, `ord` NOT NULL, `acao_row_id` NOT NULL ON DELETE CASCADE, `total_participacoes` NOT NULL DEFAULT 0) e registrar a migração em `src-tauri/src/db.rs` (PRAGMA user_version) — verde para T004
- [x] T007 [P] Adicionar seção `SRC_ENTITIES: &[EntityDef]` em `src-tauri/src/registry.rs` (rota `src_acoes`, tabela `src_acoes`, colunas de projeção + `raw_json`, `search_column: Some("titulo")`, `exported: false`) com lookup próprio — NÃO entrar em `ENTITIES` — verde para T005
- [x] T008 [P] Refatorar `src-tauri/src/import.rs`: tornar o helper `snapshot()` reutilizável com prefixo de arquivo `src-` (assinatura com sufixo/prefixo do projeto), sem mudar o comportamento do import canônico — `cargo test` existente permanece verde

**Checkpoint**: Esquema SRC pronto, registry SRC isolado do genérico, snapshot reutilizável — user stories podem começar

---

## Phase 3: User Story 1 — Escolher o projeto de trabalho na entrada do aplicativo (Priority: P1) 🎯 MVP

**Goal**: Nova página principal `/projects` com seleção Horizon | SRC; áreas de trabalho separadas; troca em sessão

**Independent Test**: login → `/projects` → escolher Horizon (dashboard atual, zero conteúdo SRC) → "Trocar projeto" → escolher SRC (área SRC vazia, zero conteúdo Horizon). Sem escolha, nenhuma base carrega.

### Tests for User Story 1 ⚠️ (TDD — escrever primeiro, ver falhar)

- [x] T009 [P] [US1] Escrever teste (vermelho) `frontend/tests/unit/ProjectSelector.test.jsx`: renderiza dois cartões distinguíveis ("Horizon", "SRC"), dispara callback `onSelect` com o projeto escolhido, e renderiza aviso de confirmação quando `pendingWarning` está ativo (edição não salva — US1-5)
- [x] T010 [P] [US1] Escrever teste (vermelho) em `src-tauri/src/lib.rs`: `ROUTES` passa a exigir `/projects` e `/src` resolvendo no binário e contagem 18 → 20 (`/src/acoes` entra no US3)

### Implementation for User Story 1

- [x] T011 [US1] Criar `frontend/src/components/ProjectSelector.jsx` (cartões Horizon e SRC, callback de seleção, aviso de confirmação para mudança não salva) — verde para T009
- [x] T012 [US1] Criar `frontend/src/pages/projects.astro` — página principal pós-login com `ProjectSelector`; sem carregar nenhuma base antes da escolha (FR-001/FR-002)
- [x] T013 [US1] Modificar `frontend/src/pages/index.astro`: autenticado redireciona para `/projects` (antes: `/dashboard`)
- [x] T014 [US1] Criar `frontend/src/layouts/SrcDashboard.astro` (menu lateral: Resumo, Ações — link desabilitado até US3, "Trocar projeto" → `/projects`) e `frontend/src/pages/src/index.astro` com resumo em estado vazio ("Nenhum dado carregado — importe o JSON consolidado do SRC")
- [x] T015 [US1] Modificar `frontend/src/layouts/Dashboard.astro`: adicionar apenas o atalho "Trocar projeto" → `/projects` (nenhum item Horizon removido — FR-005)
- [x] T016 [US1] Atualizar `ROUTES` e testes de navegação em `src-tauri/src/lib.rs` (inclui `/projects`, `/src`; mantém `/register` no teste próprio) — verde para T010
- [x] T017 [US1] Escrever smoke e2e `frontend/tests/e2e/projetos.spec.js` (WebdriverIO + tauri-driver): login → `/projects` → Horizon → dashboard → Trocar projeto → SRC → `/src` vazio; cobre SC-001 e SC-005 no fluxo navegável

**Checkpoint**: US1 funcional e testável isoladamente — seleção de projeto com separação navegável; `/src` existe com estado vazio (US2/US3 preenchem)

---

## Phase 4: User Story 2 — Carregar e devolver o JSON consolidado do SRC (Priority: P2)

**Goal**: Importar o consolidado (validação, snapshot, substituição, progresso, contagens) e exportar fiel à estrutura original

**Independent Test**: importar `src_consolidado_exemplo.json` → contagens corretas informadas; reimportar → snapshot criado; exportar sem editar → `diff` vazio contra a fixture (quickstart §3)

### Tests for User Story 2 ⚠️ (TDD — escrever primeiro, ver falhar)

- [x] T018 [P] [US2] Escrever testes (vermelho) de validação em `src-tauri/src/src_domain.rs` (`#[cfg(test)]`, fixture T001): JSON válido carrega contagens exatas (ações, participações por tipo); não-JSON, raiz sem `acoes`, item sem `acao_id`, `acao_id` duplicado e `participacoes` não-lista → erro com mensagem clara ANTES de qualquer escrita (FR-007); `acoes: []` → import válido com base vazia informada (edge case da spec)
- [x] T019 [P] [US2] Escrever teste (vermelho) de round-trip em `src-tauri/src/src_domain.rs`: importar fixture → exportar sem editar → equivalente campo a campo (chaves, ordem, valores, contadores da raiz; exceção declarada C8) — SC-003
- [x] T020 [P] [US2] Escrever teste (vermelho) em `src-tauri/src/src_domain.rs`: import sobre base existente cria snapshot `src-*` e SUBSTITUI (nunca mescla); snapshot de SRC não interfere no Horizon (FR-006/FR-015)

### Implementation for User Story 2

- [x] T021 [US2] Implementar em `src-tauri/src/src_domain.rs`: parse + validação do consolidado (contrato C1–C6) e persistência transacional — substitui `src_meta` (`campus` verbatim, `imported_at`), `src_acoes` (uma linha por ação: `acao_id`, `raw_json` do objeto íntegro, projeções `processo`/`titulo`/`natureza`/`tipo`/`coordenador` extraídas das chaves de rótulo) e `src_participacoes` (uma linha por entrada com `ord` da posição na lista, `tipo`, contexto de atividade, `nome` de `"Nome"`, `raw_json` íntegro) — verde para T018/T020
- [x] T022 [US2] Implementar em `src-tauri/src/src_domain.rs`: remontagem do export (C9: `raw_json` emitido com ordem de chaves preservada; `total_participacoes` e `participacoes` por ação em `ord`; contadores da raiz recomputados — `total_acoes`, `acoes_com_participacoes`, `total_publico_alvo`, `total_equipe`, `total_atividades` por distintos `atividade_num`; serialização `indent=2` sem escape de não-ASCII) — verde para T019
- [x] T023 [US2] Criar comandos `import_src_json` e `export_src_json` em `src-tauri/src/commands.rs` (contratos/ipc-commands.md: diálogo aberto no Rust, `require_session`, eventos `src-import://progress`, retorno com contagens + snapshot) e registrá-los em `src-tauri/src/lib.rs`
- [x] T024 [P] [US2] Escrever teste (vermelho) `frontend/tests/unit/api.test.js`: rotas novas em `frontend/src/services/api.js` — `POST /src/import`, `POST /src/export`, `GET|PUT /src/meta` mapeiam para os comandos corretos
- [x] T025 [US2] Adicionar as rotas `/src/import`, `/src/export`, `/src/meta` em `frontend/src/services/api.js` (antes das rotas genéricas `/:entity`) — verde para T024
- [x] T026 [P] [US2] Escrever teste (vermelho) `frontend/tests/unit/SrcDataControl.test.jsx`: botões importar/exportar, exibição de progresso do evento `src-import://progress`, mensagem de sucesso com contagens, mensagem de erro sem apagar estado, drag-drop reusa o caminho do diálogo
- [x] T027 [US2] Criar `frontend/src/components/SrcDataControl.jsx` espelhando `frontend/src/components/DataControlCenter.jsx` (invoke via `api.js`, listen de progresso, drag-drop) — verde para T026
- [x] T028 [US2] Completar `frontend/src/pages/src/index.astro`: resumo com campus/totais após o import e `SrcDataControl` integrado (FR-009)

**Checkpoint**: US2 funcional isoladamente — import com validação/snapshot/contagens e export fiel (diff vazio na base intacta)

---

## Phase 5: User Story 3 — Curar os dados do SRC (ações e participações) (Priority: P3)

**Goal**: CRUD completo de ações (tabela com busca/ordenação) e de participações aninhadas, com vínculos programa→filhas explícitos

**Independent Test**: com base importada — criar, editar e excluir uma ação e suas participações; cada alteração aparece na listagem e no export; exclusão de ação usada como "Ação vinculante" exige escolha explícita

### Tests for User Story 3 ⚠️ (TDD — escrever primeiro, ver falhar)

- [x] T029 [P] [US3] Escrever testes (vermelho) de CRUD de ações em `src-tauri/src/src_domain.rs`: criar (raw_json construído com chaves canônicas — `"Processo nº"`, `"Título ação"`, `"Natureza"`, `"Tipo ação"`, `"Coordenador(a)"`), atualizar grava projeção E `raw_json` sincronizados, `acao_id` preservado, `acao_id` vazio/duplicado → Validation, listagem com busca por `titulo`, ordenação e paginação (`limit`/`offset` com total)
- [x] T030 [P] [US3] Escrever testes (vermelho) de CRUD de participações em `src-tauri/src/src_domain.rs`: criar com `ord` = max+1 e chaves canônicas (`atividade_num`, `atividade_id`, `atividade`, `tipo`, `"Nome"`, campos da pessoa), atualizar/delete mantêm ordem contígua, `tipo` vazio → Validation, entradas existentes preservam chaves extras (C5); excluir ação referenciada como `"Ação vinculante"` por outra → `Conflict` com lista de filhas, e `force` desvincula com aviso (FR-013)
- [x] T031 [P] [US3] Escrever teste (vermelho) `frontend/tests/unit/api.test.js`: rotas `GET|POST /src/acoes`, `GET|PUT|DELETE /src/acoes/:id`, `GET|POST /src/acoes/:id/participacoes`, `PUT|DELETE /src/participacoes/:id`
- [x] T032 [P] [US3] Escrever teste (vermelho) `frontend/tests/unit/ParticipacoesEditor.test.jsx`: lista participações da ação agrupadas por `tipo` e contexto de atividade, adiciona/edita/remove linha com confirmação, exibe campos da pessoa pelos rótulos do arquivo

### Implementation for User Story 3

- [x] T033 [US3] Implementar CRUD de ações em `src-tauri/src/src_domain.rs` (sincronização projeção↔raw_json como regra Rust — Princípio VI; validação de `acao_id` único; consulta de vínculo por igualdade textual em `acao_vinculante`) — verde para T029
- [x] T034 [US3] Implementar CRUD de participações em `src-tauri/src/src_domain.rs` (ord contígua, extras preservados, Conflict/force do vínculo) — verde para T030
- [x] T035 [US3] Criar comandos `src_list_acoes`, `src_get_acao`, `src_create_acao`, `src_update_acao`, `src_delete_acao`, `src_list_participacoes`, `src_create_participacao`, `src_update_participacao`, `src_delete_participacao`, `src_get_meta`, `src_update_meta` em `src-tauri/src/commands.rs` e registrá-los em `src-tauri/src/lib.rs` (contratos/ipc-commands.md — comandos genéricos do Horizon invários)
- [x] T036 [US3] Adicionar rotas de ações/participações em `frontend/src/services/api.js` — verde para T031
- [x] T037 [US3] Criar `frontend/src/pages/src/acoes.astro` (EntityTable/EntityPage com `columns={['titulo','processo','natureza','tipo','coordenador']}`, busca e ordenação — FR-010/FR-011) e atualizar `ROUTES` em `src-tauri/src/lib.rs` (18+5 → 21; guard `contagem_de_rotas_bate_com_o_menu_lateral` atualizado) + habilitar o link "Ações" do `SrcDashboard.astro`
- [x] T038 [US3] Criar `frontend/src/components/ParticipacoesEditor.jsx` integrado ao editor da ação (coleção aninhada — FR-012; exibe aviso de vínculo quebrado quando `acao_vinculante` não resolve) — verde para T032
- [x] T039 [US3] Integrar o aviso de edição não salva na troca de projeto: `ProjectSelector`/layouts consultam o estado do formulário ativo e confirmam antes de navegar (US1-5, FR-003) — verde para o cenário 12 do quickstart

**Checkpoint**: US3 funcional isoladamente — curadoria completa do SRC com vínculos explícitos

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Verificações transversais que afetam todas as stories

- [x] T040 [P] Auditoria de PII/Logs (contrato C7): verificar que mensagens de erro, eventos de progresso e logs do Rust (`log::`) não incluem conteúdo de participações (CPF/e-mail/nomes); corrigir qualquer vazamento em `src-tauri/src/src_domain.rs`/`commands.rs`
- [x] T041 [P] Verificação de desempenho (SC-006): gerar consolidado sintético com ~2.000 ações/~10.000 participações em `src-tauri/tests/fixtures/` (script one-off descartável), importar via teste de integração e assegurar conclusão < 30 s com progresso; confirmar listagem paginada sem carregar tudo
- [x] T042 Executar `specs/033-src-data-tab/quickstart.md` completo: `cargo test`, `npm test`, smoke e2e, cenário manual de 12 passos e `diff` do round-trip (SC-001…SC-006); registrar resultado no PR

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sem dependências — T001/T002/T003 imediatos
- **Foundational (Phase 2)**: depende do Setup — BLOQUEIA todas as user stories
- **US1 (Phase 3)**: depende da Foundational (T006/T007 para o teste de rotas não confundirem o registry; T014 usa o estado vazio)
- **US2 (Phase 4)**: depende de Foundational (migração, snapshot T008, src_domain) — independente de US1 exceto pelas páginas de destino (T014 entrega `/src` vazio)
- **US3 (Phase 5)**: depende de US2 (a base precisa estar importável para o CRUD ter dados; comandos compartilham `src_domain.rs`)
- **Polish (Phase 6)**: depende de todas as stories desejadas estarem completas

### User Story Dependencies

- **US1 (P1)**: após Foundational — nenhuma dependência de outras stories (entrega `/projects` + áreas navegáveis, `/src` vazio)
- **US2 (P2)**: após Foundational — usa apenas a página vazia que US1 cria; testável isoladamente com a fixture
- **US3 (P3)**: após US2 — CRUD opera sobre a base carregada; editor e aviso de troca integram com US1

### Within Each User Story

- Testes PRIMEIRO (vermelho), depois implementação (verde) — Constituição I
- Domínio Rust (`src_domain.rs`) antes dos comandos; comandos antes das rotas `api.js`; rotas antes dos componentes; componentes antes das páginas
- Checkpoint de story: validar isoladamente antes de avançar

### Parallel Opportunities

- T001/T002, T004/T005, T006/T007/T008, T018/T019/T020, T024/T026, T029/T030/T031/T032, T040/T041 rodam em paralelo (arquivos distintos, sem dependência)
- US1 (front-end/rotas) e US2 (domínio Rust) podem avançar em paralelo após a Foundational, por pessoas diferentes
- Testes de uma mesma story marcados [P] rodam juntos

---

## Parallel Example: User Story 2

```bash
# Testes do domínio (arquivos/módulos independentes, mesmos arquivos de fixture):
Task: "T018 validações do consolidado em src-tauri/src/src_domain.rs"
Task: "T019 round-trip em src-tauri/src/src_domain.rs"
Task: "T020 snapshot/substituição em src-tauri/src/src_domain.rs"

# Front-end (após comandos prontos):
Task: "T024 rotas api.js (teste)"
Task: "T026 SrcDataControl (teste)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 Setup (T001–T003)
2. Phase 2 Foundational (T004–T008) — CRÍTICO
3. Phase 3 US1 (T009–T017)
4. **STOP and VALIDATE**: login → seleção → áreas separadas (quickstart §2 passos 1–4)
5. Demo: separação de projetos navegável com SRC em estado vazio

### Incremental Delivery

1. Setup + Foundational → fundação pronta
2. US1 → validar isoladamente → **MVP** (seleção e separação)
3. US2 → validar isoladamente (import + export fiel, diff vazio)
4. US3 → validar isoladamente (curadoria completa)
5. Polish → quickstart completo + auditorias

### Parallel Team Strategy

1. Equipe completa Setup + Foundational juntos
2. Dev A: US1 (front-end/rotas) — Dev B: US2 (domínio Rust/import-export)
3. Após US2: Dev A → US3 front-end — Dev B → US3 domínio/comandos
4. Polish em conjunto (T042 é a validação final compartilhada)

---

## Notes

- [P] tasks = arquivos diferentes, sem dependência de tarefas incompletas
- [Story] mapeia a task para a user story da spec (rastreabilidade)
- Verificar que cada teste falha ANTES de implementar (Constituição I)
- Commit após cada task ou grupo lógico
- Parar em qualquer checkpoint para validar a story isoladamente
- Evitar: tasks vagas, conflito de mesmo arquivo entre tarefas paralelas, dependências entre stories que quebram a independência

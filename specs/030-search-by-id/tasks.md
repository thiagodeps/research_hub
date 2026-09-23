---
description: "Task list for 030-search-by-id implementation"
---

# Tasks: Pesquisa de Registros por ID

**Input**: Design documents from `/specs/030-search-by-id/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/api.md, quickstart.md

**Tests**: INCLUDED — obrigatórios. A Constituição (Princípio I — TDD Obrigatório) exige
escrever o teste primeiro, vê-lo falhar (red), implementar até passar (green). Cada fase de
User Story lista os testes antes da implementação.

**Organization**: Tasks agrupadas por User Story. Atenção à realidade do design (plan.md):
**toda a mudança vive em `src-tauri/src/crud.rs`** (função `list` + módulo de testes) —
portanto tasks no mesmo arquivo NÃO recebem `[P]`. A única task genuinamente paralela é a
suíte Vitest do frontend, que não toca arquivo nenhum (regressão).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Desktop app (Tauri)**: núcleo em `src-tauri/src/`, front-end em `frontend/src/`
- Nenhum arquivo novo de código é criado nesta feature (plan.md — Structure Decision)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Estabelecer linha de base verde antes de qualquer mudança

- [X] T001 Run baseline test suites and confirm all green before any change: `cargo test` in `src-tauri/` and `npm test` in `frontend/` — record results; any pre-existing failure blocks all subsequent tasks *(cargo: 82 passed; npm: 26 passed — após instalar build-essential, Node 22 e libs Tauri Linux)*
- [X] T002 Confirm working tree touches only files listed in plan.md Source Code Layout (i.e., none yet) via `git status` in repository root — the feature must not modify `frontend/`, `migrations/`, or `registry.rs` *(verificado: nenhum diff em src-tauri/ ou frontend/)*

**Checkpoint**: Baseline verde confirmada; pode começar a User Story 1.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Pré-requisitos compartilhados entre as histórias

**Nenhuma task nesta fase — justificativa**: o design concentra a regra em `crud.rs::list`
(plan.md); não há infraestrutura nova, dependência nova, migração nem modelo novo. O helper
de classificação de termo (research.md D1) é criado dentro da US1, pois só ela o consome
inicialmente. Sem este desbloqueio implícito, todas as histórias dependem apenas do estado
atual de `crud.rs`.

**Checkpoint**: N/A — User Stories podem iniciar imediatamente após Phase 1.

---

## Phase 3: User Story 1 - Localizar um registro pelo seu ID (Priority: P1) 🎯 MVP

**Goal**: Um termo puramente numérico digitado no campo de busca retorna o registro cujo ID
é exatamente aquele número, na primeira posição da listagem, em entidades com coluna
textual pesquisável.

**Independent Test**: Com o banco semeado (`seeded()` de `crud.rs`: campuses 1–4), buscar
`"3"` retorna somente o campus de ID 3 (Cariacica) como primeiro item; limpar o termo
restaura a listagem integral; buscar um ID inexistente devolve lista vazia sem erro.

### Tests for User Story 1 (write FIRST, ensure they FAIL before implementation) ⚠️

> Todos em `src-tauri/src/crud.rs`, módulo `#[cfg(test)]` existente (mesmo arquivo —
> executar em sequência, sem [P]).

- [X] T003 [US1] Write test `search_by_exact_id_returns_record_first` in `src-tauri/src/crud.rs` tests: with `seeded()` fixture, `list(&conn, "campuses", None, None, Some("3"), None, None)` returns `total == 1`, `items[0]["id"] == 3` and `items[0]["name"] == "Cariacica"` (US1 scenario 1, SC-003); confirm it FAILS (today "3" matches nothing textually and `total == 0`)
- [X] T004 [US1] Write test `search_by_id_with_leading_zeros_matches_same_record` in `src-tauri/src/crud.rs` tests: `Some("003")` returns the campus with `id == 3` (research.md D1 — leading zeros normalize via i64 parse); confirm it FAILS
- [X] T005 [US1] Write test `search_by_missing_id_returns_empty_page_without_error` in `src-tauri/src/crud.rs` tests: `Some("9999999")` returns `items` empty and `total == 0`, `Ok` result (US1 scenario 3); confirm it FAILS (today: textually ignored — actually passes as `total == 4`, so assert `total == 0` makes it fail)
- [X] T006 [US1] Write test `empty_or_whitespace_search_lists_everything` in `src-tauri/src/crud.rs` tests: `None`, `Some("")` and `Some("   ")` all return `total == 4` (US1 scenario 2 + research.md D6 trim); confirm current behavior for `Some("   ")` FAILS (textually matches nothing today)

### Implementation for User Story 1

- [X] T007 [US1] Add term classification helper `fn parse_id_term(term: &str) -> Option<i64>` in `src-tauri/src/crud.rs`: return `Some(n)` only when the trimmed term is non-empty, all chars are ASCII digits (`chars().all(|c| c.is_ascii_digit())`) and `str::parse::<i64>()` succeeds (research.md D1 — parse alone would accept `-3`/`+7`; overflow degrades to text-only); do NOT trim the LIKE term beyond what today's code does
- [X] T008 [US1] Rework WHERE construction in `list()` in `src-tauri/src/crud.rs` (research.md D2): when `parse_id_term` yields `Some(id)` and the entity has `search_column`, build ` WHERE (id = ?1 OR {col} LIKE ?2 ESCAPE '\')` with LIKE term `%{escaped}%` exactly as today (reuse `escape_like`); switch count and page queries to explicit numbered params `?1..?4` (`?3` = LIMIT, `?4` = OFFSET) so `?1` can be reused; when `parse_id_term` yields `None`, keep the existing textual branch byte-for-byte (` WHERE {col} LIKE ?1 ESCAPE '\'`) — FR-002, FR-003, FR-005
- [X] T009 [US1] Add relevance ordering in `list()` in `src-tauri/src/crud.rs` (research.md D3): when the numeric branch is active, prefix ORDER BY with `(id = ?1) DESC,` before the existing sort expression (user sort / nulls-last rule untouched for remaining rows) — SC-003
- [X] T010 [US1] Run `cargo test` in `src-tauri/`: all T003–T006 tests turn green AND every pre-existing test (including `searches_case_insensitively`, `search_treats_wildcards_as_literal_text`) stays green

**Checkpoint**: MVP funcional — buscar "3" em campuses traz Cariacica primeiro; busca
textual intacta. Parar aqui já entrega valor demonstrável (US1 completa).

---

## Phase 4: User Story 2 - A busca textual continua funcionando como hoje (Priority: P2)

**Goal**: Garantir comportamento aditivo e zero regressão: termo numérico que também
ocorre no texto traz a união dos matches; termos não-numéricos seguem 100% textuais.

**Independent Test**: Com campuses contendo "Bloco 7" (id 7), buscar `"7"` retorna o
registro de ID 7 primeiro E "Bloco 7" na mesma lista; buscar `"abc123"`, `"-3"`, `"12.5"`
e um número com overflow retorna exatamente o que a busca textual de hoje retorna.

### Tests for User Story 2 (write FIRST; they must FAIL or expose gaps before US2 is "done")

> Mesmo arquivo `src-tauri/src/crud.rs` — sequencial.

- [X] T011 [US2] Write test `numeric_search_returns_union_of_id_and_text_matches` in `src-tauri/src/crud.rs` tests: extend `seeded()` usage with `INSERT INTO campuses (id, name) VALUES (7, 'Bloco 7')`, then `Some("7")` returns `total == 2` with `items[0]["id"] == 7` first and the "Bloco 7" row also present (FR-004, SC-003); confirm it FAILS before this story's fix (note: it may already pass after US1's union SQL — if it passes immediately, record that and keep the test as regression guard)
- [X] T012 [US2] Write test `non_numeric_terms_use_text_search_only` in `src-tauri/src/crud.rs` tests covering the data-model grammar T4: insert `('Bloco abc123', 'Meta 12.5', 'Débito -3')` rows and assert `Some("abc123")`, `Some("12.5")`, `Some("-3")` match ONLY their textual rows (never by ID), and a 30-digit overflow term matches nothing (FR-002, research.md D1); confirm FAIL/gap before implementation detail (classification edge: today a numeric-looking `-3` would be… textually matched, so assert exact textual-only behavior)
- [X] T013 [US2] Write test `user_sort_still_applies_after_id_relevance` in `src-tauri/src/crud.rs` tests: with the ID-7/Bloco-7 fixture and `Some("7")` plus `sort=Some("name"), order=Some("asc")`, `items[0]["id"] == 7` still first while the remaining rows follow name ASC (research.md D3); confirm FAIL if ordering prefix is missing

### Implementation for User Story 2

- [X] T014 [US2] Fix whatever T011–T013 expose in `src-tauri/src/crud.rs` (expected: param re-indexing and ORDER BY prefix details from T008/T009; no new SQL shapes) — keep the textual WHERE branch untouched (FR-005)
- [X] T015 [P] [US2] Run `npm test` in `frontend/` and confirm the existing Vitest suite passes with ZERO modifications to any frontend file (SC-002 regression guarantee; `api.js` and `EntityPage.jsx` must show no diff in `git status`)

**Checkpoint**: US1 + US2 ambas verdes: comportamento aditivo comprovado, zero regressão
textual, front-end intocado.

---

## Phase 5: User Story 3 - Pesquisa por ID disponível em todas as entidades (Priority: P3)

**Goal**: A busca por ID funciona de forma uniforme em TODAS as entidades do registry,
incluindo as sem coluna textual pesquisável (hoje `proficiencies`).

**Independent Test**: Em `proficiencies` (única entidade com `search_column: None`), buscar
o ID de um registro retorna exatamente esse registro; termo textual continua ignorado
(listagem integral); repetido para cada entidade de `registry::exported()`, o ID do próprio
registro sempre volta.

### Tests for User Story 3 (write FIRST, ensure they FAIL before implementation) ⚠️

- [X] T016 [US3] Write test `numeric_search_filters_by_id_for_entities_without_search_column` in `src-tauri/src/crud.rs` tests: in `db::open_in_memory()`, insert two `proficiencies` rows, then `list(&conn, "proficiencies", None, None, Some("<id>"), None, None)` returns `total == 1` with that exact row, while `Some("qualquer")` still returns `total == 2` (FR-006, research.md D4 — the existing `search_is_ignored_for_entities_without_search_column` keeps passing unchanged); confirm the numeric half FAILS today (today: filter ignored, `total == 2`)
- [X] T017 [US3] Write test `numeric_search_works_uniformly_across_all_entities` in `src-tauri/src/crud.rs` tests: for every `def` in `registry::exported()`, create one record via `create()` (pattern of `all_fifteen_entities_support_full_crud`), capture its generated `id`, then `list(&conn, def.route, None, None, Some(&id.to_string()), None, None)` returns `total == 1` with `items[0]["id"] == id` (US-3 scenario 2, SC-004); confirm it FAILS for `proficiencies` (and any other `search_column: None` entity) before implementation

### Implementation for User Story 3

- [X] T018 [US3] Extend the numeric branch in `list()` in `src-tauri/src/crud.rs` (research.md D2/D4): when `parse_id_term` yields `Some(id)` and the entity has NO `search_column`, build ` WHERE id = ?1` with `ORDER BY (id = ?1) DESC` prefix and explicit `?2`/`?3` for LIMIT/OFFSET (count query mirrors the same WHERE); textual-only terms on these entities remain fully ignored — no WHERE — exactly as today (FR-006)
- [X] T019 [US3] Run `cargo test` in `src-tauri/`: T016–T017 green, plus full suite green including `search_is_ignored_for_entities_without_search_column` untouched

**Checkpoint**: Todas as histórias funcionais: a busca por ID é uniforme no sistema inteiro.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Validação ponta-a-ponta e conformidade com o plano

- [X] T020 Verify scope discipline in repository root: `git status`/`git diff --stat` shows changes ONLY in `src-tauri/src/crud.rs` (function `list`, helper `parse_id_term`, test module) and `specs/030-search-by-id/` — any other source file is a plan violation to be reverted (plan.md Source Code Layout)
- [X] T021 Run the complete quickstart.md automated validation: `cargo test` in `src-tauri/` and `npm test` in `frontend/` both 100% green, mapping each of the 7 quickstart scenarios to its passing test (cite test names in the run notes)
- [X] T022 Execute the manual E2E table from `specs/030-search-by-id/quickstart.md` (steps 1–10 with `npm run tauri dev`, login `admin@admin.com`) and record outcomes; confirm SC-001 (find by ID < 10s), SC-003 (first position) and SC-004 (uniform behavior across entity pages) *(validado pelo usuário em 2026-09-23 — app rodou via `cargo run` + astro dev; todos os passos ok)*

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sem dependências — executar imediatamente; baseline verde BLOQUEIA todo o resto
- **Foundational (Phase 2)**: vazia por design (ver justificativa)
- **US1 (Phase 3)**: depende apenas da Phase 1 — 🎯 MVP
- **US2 (Phase 4)**: depende da US1 (o SQL da união e a ordenação são entregues lá); suas tasks são majoritariamente testes de garantia
- **US3 (Phase 5)**: depende da US1 (branch numérico já existe); independente da US2 — poderia rodar em paralelo com ela por outro desenvolvedor, pois só acrescenta o ramo sem `search_column`
- **Polish (Phase 6)**: depende de todas as histórias desejadas estarem completas

### User Story Dependencies

- **US1 (P1)**: independente — constrói classificação + WHERE união + ordenação
- **US2 (P2)**: verifica/consolida US1; nenhum formato SQL novo
- **US3 (P3)**: estende US1 para entidades sem coluna textual; não conflita com US2 (ambas editam `crud.rs`, então na prática execute-as sequencialmente no mesmo arquivo)

### Within Each User Story

- Testes PRIMEIRO, vendo-os falhar (Constituição I — red/green/refactor)
- Helper de classificação antes do WHERE; WHERE antes do ORDER BY
- Suíte completa verde encerra cada história

### Parallel Opportunities

- T015 (suíte Vitest) é a única task [P] real: roda em paralelo com qualquer task Rust, pois não modifica arquivos
- T016/T017 (US3) podem ser escritas em paralelo com T011–T013 (US2) por desenvolvedores diferentes, mas o commit em `crud.rs` deve ser serializado
- US2 e US3 são independentes em conteúdo; em equipe, divida: um assume US2, outro US3, sincronizando no mesmo arquivo

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: baseline verde
2. Phase 3: US1 completa (T003–T010)
3. **STOP and VALIDATE**: buscar "3" traz Cariacica primeiro; buscar nome continua igual
4. Já é entregável: o pedido original ("pesquisa por id") funciona nas entidades principais

### Incremental Delivery

1. MVP (US1) → validar
2. US2 → garantia de união + zero regressão → validar
3. US3 → uniformidade em todas as entidades (resgata páginas com busca inerte) → validar
4. Polish → quickstart E2E + disciplina de escopo

### Single-Developer Strategy (recomendado para esta feature)

Arquivo único (`crud.rs`) ⇒ executar estritamente em ordem T001→T022. Os pontos de parada
úteis são os Checkpoints das fases 3, 4 e 5.

---

## Notes

- [P] tasks = arquivos diferentes, sem dependências — nesta feature quase nada é paralelizável (arquivo único)
- [Story] label mapeia a task para a User Story da spec (US1/US2/US3)
- Cada história é independentemente completável e testável (ver Independent Test de cada fase)
- Verifique os testes falhando ANTES de implementar (Constituição I)
- Commit após cada task ou grupo lógico; pare em qualquer Checkpoint para validar a história
- Evitado por design: tasks vagas, conflito de arquivos entre histórias simultâneas, dependência cruzada que quebre a independência US2×US3

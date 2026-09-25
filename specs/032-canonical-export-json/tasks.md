---

description: "Task list for Adaptação ao Novo Export Canonical (Somente JSON + Aba Campus)"
---

# Tasks: Adaptação ao Novo Export Canonical (Somente JSON + Aba Campus)

**Input**: Design documents from `/specs/032-canonical-export-json/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: INCLUÍDOS — a Constituição (Princípio I: TDD Obrigatório) e a spec (SC-006) exigem teste primeiro para toda regra nova. Cada task de teste deve falhar antes da implementação correspondente.

**Organization**: Tasks agrupadas por user story para implementação e teste independentes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Pode rodar em paralelo (arquivos diferentes, sem dependência de task incompleta)
- **[Story]**: User story dona da task (US1, US2, US3)
- Caminhos exatos em todas as descrições

## Path Conventions

- Núcleo Rust: `src-tauri/src/` (testes embutidos em `#[cfg(test)]` em cada módulo)
- Front-end: `frontend/src/` (testes Vitest em `frontend/src/` conforme padrão do repo)
- Fixture de referência: `exports_canonical.zip` na raiz do repo (formato novo, JSON-only, 395 entradas); fixture legado: `~/Downloads/exports_canonical.zip` (626 entradas, parquet + JSON)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Baseline verde antes de qualquer mudança

- [X] T001 Rodar `cargo test` e `npm test` na branch `032-canonical-export-json` e registrar o baseline atual como verde (nenhuma mudança de código ainda; falhas aqui bloqueiam tudo)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Módulo de leitura JSON compartilhado por US1 (import) e US3 (export) — nenhuma user story começa sem ele

**⚠️ CRITICAL**: Nenhuma user story pode começar antes deste phase

### Tests for Foundational (TDD — escrever primeiro, ver falhar)

- [X] T002 [P] Escrever testes do leitor canônico JSON em `src-tauri/src/json_io.rs` (módulo novo `#[cfg(test)]`): parse de array de objetos → `Table { columns, rows: Vec<Vec<Option<String>>> }` filtrando só as colunas pedidas; campo booleano vira texto `"True"/"False"`; número `23` vira `"23"` e `23.0` vira `"23.0"`; `null` vira `None`; array/objeto vira texto JSON compacto; campo não pedido é ignorado; campo pedido ausente vira `None`; arquivo vazio `[]` → tabela com 0 linhas; JSON malformado → erro `AppError`

### Implementation for Foundational

- [X] T003 Criar o módulo `src-tauri/src/json_io.rs` com `pub struct Table` (mesma forma de `parquet_io::Table`), `pub fn read_columns(bytes: &[u8], wanted: &[&str]) -> Result<Table, AppError>` (decisão R5/R6 do research.md) e registrar `mod json_io;` em `src-tauri/src/lib.rs`; rodar `cargo test` até T002 passar

**Checkpoint**: Foundation pronta — US1, US2 e US3 podem começar (US2 depende só de T001)

---

## Phase 3: User Story 1 - Importar o pacote canônico em JSON (Priority: P1) 🎯 MVP

**Goal**: O pacote JSON-only do DataLake carrega as 15 tabelas com o fluxo vigente (substituição, snapshot, ids, progresso), abortando sem apagar nada se faltar tabela

**Independent Test**: importar um ZIP contendo apenas `{tabela}_canonical.json` na raiz → 15 tabelas carregadas com contagens corretas; remover o JSON de uma tabela → erro nomeando a tabela e base intacta

### Tests for User Story 1 (TDD — escrever primeiro, ver falhar)

- [X] T004 [US1] Escrever teste `loads_canonical_json_tables` em `src-tauri/src/import.rs`: fixture ZIP (helper novo) com `campuses_canonical.json` e `languages_canonical.json` na raiz, **sem** nenhum `.parquet` → tabelas carregadas com as linhas do JSON, ids explícitos preservados, progresso por tabela, admins intactos
- [X] T005 [US1] Escrever teste `aborts_when_managed_table_file_missing` em `src-tauri/src/import.rs` (FR-014, clarificação Q1): ZIP com `campuses_canonical.json` mas sem o arquivo de `languages` → `import_archive` retorna erro nomeando a tabela faltante, base mantém os dados anteriores e `summary.snapshot` nunca é criado
- [X] T006 [US1] Escrever testes de compatibilidade legada em `src-tauri/src/import.rs` (FR-006): ZIP v1 (parquet + JSON idênticos) carrega as 15 tabelas; ZIP híbrido (JSON de uma tabela + parquet de outra) carrega ambas; quando JSON e parquet existem para a mesma tabela, o conteúdo do JSON prevalece
- [X] T007 [US1] Atualizar o teste de referência `reference_archive_matches_python_row_counts` em `src-tauri/src/import.rs` para o pacote novo (`exports_canonical.zip` JSON-only): manter a regra (contagens recontadas do pacote atual) e o assert de 15 tabelas carregadas; adicionar caso de tabela vazia `[]` carregando como 0 linhas

### Implementation for User Story 1

- [X] T008 [US1] Implementar em `src-tauri/src/import.rs` a resolução de fonte por entidade (decisão R1): iterar `registry::exported()`, procurar `{tabela}_canonical.json` na raiz e cair para `parquet/{tabela}_canonical.parquet` via `parquet_io` quando o JSON não existir; carga usa `json_io::read_columns` para JSON e o caminho parquet atual para o fallback
- [X] T009 [US1] Implementar a validação de completude (FR-014, decisão R7) em `src-tauri/src/import.rs`: após abrir o ZIP e antes do snapshot e dos `DELETE`, verificar que cada tabela gerenciada tem fonte (JSON ou parquet); erro `AppError` nomeando a(s) tabela(s) faltante(s) sem criar snapshot nem apagar dados
- [X] T010 [US1] Fazer `cargo test` passar integralmente (T004–T007 verdes) sem regressão nos testes existentes de snapshot/transação/progresso; confirmar que `original.zip` só é gravado após commit bem-sucedido

**Checkpoint**: User Story 1 funcional e testável isoladamente — o pacote novo importa (MVP)

---

## Phase 4: User Story 2 - Aba Campus sem o campus aninhado (Priority: P2)

**Goal**: Campus tratado só pelos campos reais; campo fantasma "Campus (Vínculos)" some da listagem e do formulário

**Independent Test**: importar o pacote novo e abrir a aba Campuses → listagem exibe exatamente `id` e `name`; formulário sem o campo fantasma; dado aninhado de pacote antigo é descartado sem erro

### Tests for User Story 2 (TDD — escrever primeiro, ver falhar)

- [X] T011 [US2] Primeiro fazer falhar o teste de sincronia `dashboard_pages_only_declare_columns_that_exist` em `src-tauri/src/registry.rs` removendo `campus` dos `columns` de campuses **no teste mental do fluxo TDD**: adicionar em `frontend/src/` (teste Vitest da página ou fixture usada pelo teste do registry) a expectativa de que `frontend/src/pages/dashboard/campuses.astro` declara `columns={['id', 'name']}` e não contém o formField `json_readonly` "Campus (Vínculos)" — o teste fica vermelho contra a página atual
- [X] T012 [P] [US2] Escrever teste `discards_nested_campus_field` em `src-tauri/src/json_io.rs` (FR-008): fixture JSON de campus no formato v1 (com `"campus": {"id": 6, "name": "Serra"}` aninhado) → carrega sem erro, o campo aninhado não aparece entre as colunas carregadas e não pode reaparecer no export

### Implementation for User Story 2

- [X] T013 [US2] Remover `"campus"` dos `columns` da `EntityDef` de campuses em `src-tauri/src/registry.rs` (R3: coluna física `campus TEXT` de `src-tauri/migrations/001_init.sql` permanece dormente — **sem** migration destrutiva); testes do registry voltam a verde
- [X] T014 [US2] Atualizar `frontend/src/pages/dashboard/campuses.astro`: `columns={['id', 'name']}` (clarificação Q2) e remover o formField `{ name: 'campus', label: 'Campus (Vínculos)', type: 'json_readonly' }`; testes Vitest de T011 verdes
- [X] T015 [US2] Verificar `frontend/src/components/EntityForm.jsx` e demais páginas: os vínculos `campus` legítimos de researchers/students/articles etc. permanecem intactos (nenhuma outra página muda); `npm test` verde

**Checkpoint**: User Stories 1 E 2 funcionam independentemente

---

## Phase 5: User Story 3 - Exportar no novo formato JSON (Priority: P3)

**Goal**: Export gera `{tabela}_canonical.json` para as 15 tabelas, sem parquet, preservando não gerenciados byte a byte e restaurando tipos a partir do JSON do original

**Independent Test**: importar o pacote novo, exportar sem editar → 15 JSONs, zero parquet, não gerenciados idênticos byte a byte, tipos preservados (`true` booleano, `23` número, `null` null, vínculos como estrutura)

### Tests for User Story 3 (TDD — escrever primeiro, ver falhar)

- [X] T016 [US3] Escrever teste `exports_only_canonical_json_files` em `src-tauri/src/export.rs` (FR-009): export com e sem `original.zip` → contém as 15 `{tabela}_canonical.json` na raiz, **nenhuma** entrada `parquet/`; substituir os testes `exports_fifteen_tables_and_skips_admins` e `empty_table_still_produces_valid_files` (que esperam 30 entradas e parquet) pelas novas expectativas
- [X] T017 [US3] Escrever teste `infers_types_from_original_json` em `src-tauri/src/export.rs` (FR-011, decisão R2): `original.zip` com `{tabela}_canonical.json` cuja primeira linha define booleano, inteiro `23`, decimal `23.0`, `null` e estrutura → valores textuais do banco voltam com esses tipos; coluna sem valor de referência sai como string; sem `original.zip` tudo sai como texto (caminho degradado)
- [X] T018 [US3] Escrever teste `legacy_parquet_entries_are_dropped_not_preserved` em `src-tauri/src/export.rs` (R4): `original.zip` legado → parquets de tabelas gerenciadas **não** aparecem no export, não contam em `preserved_entries`, e as entradas não gerenciadas continuam byte a byte (substituir o fixture parquet `typed_original` por fixture JSON equivalente; atualizar `real_archive_round_trip_preserves_schema_and_entries` para o pacote novo: 395 entradas, `preserved = 380`, tipos e ordem de colunas via JSON)

### Implementation for User Story 3

- [X] T019 [US3] Implementar em `src-tauri/src/json_io.rs` a inferência de tipos (R2): `pub fn infer_column_types(bytes: &[u8]) -> Result<HashMap<String, ColType>, AppError>` — tipo do primeiro valor não-null por coluna (Bool/Int/Float/Text/Structure), relendo o JSON do `original.zip`; testes T017 guiam a API
- [X] T020 [US3] Reescrever o loop de geração em `src-tauri/src/export.rs` (R4): remover `write_parquet`/`parquet_io::schema_of`/`parquet_io::column_order` do caminho de saída; gerar somente `{base}.json` via `write_json` existente; tipo por coluna vem de `json_io::infer_column_types` sobre o `{tabela}_canonical.json` do original (fallback texto); preservação byte a byte mantida com a exceção declarada dos `parquet/{tabela}_canonical.parquet` gerenciados (descartados)
- [X] T021 [US3] Fazer `cargo test` passar integralmente (T016–T018 verdes), incluindo o round-trip real do pacote novo e a coerção `as_bool` existente; `preserved_entries` do resumo reflete a nova regra

**Checkpoint**: Todas as user stories funcionam independentemente — ciclo completo importar→editar→exportar no formato novo

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Ajustes que atravessam as histórias

- [X] T022 [P] Atualizar menções ao formato nas docs: `README.md`, `docs/index.md`, `docs/index.en.md` (parquet → JSON canônico; fluxo de import/export; remover referência a "arrow-rs — reading and writing canonical .parquet files")
- [X] T023 [P] Revisão de consistência do vocabulário no código: comentários de `src-tauri/src/import.rs`, `export.rs` e `parquet_io.rs` atualizados para refletir JSON como formato primário e parquet como fallback legado (R1/R4), sem mudança de comportamento
- [X] T024 Rodar a validação completa de `specs/032-canonical-export-json/quickstart.md`: `cargo test`, `npm test` e o e2e manual (import do pacote novo → aba Campuses → export sem edição → round-trip com curadoria → falha controlada com tabela faltante), conferindo SC-001 a SC-006

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: imediato; baseline verde bloqueia tudo
- **Foundational (Phase 2)**: depende de T001; **bloqueia US1 e US3** (US2 só precisa de T001)
- **US1 (Phase 3)**: depende de Phase 2 (json_io)
- **US2 (Phase 4)**: depende de T001 (e de T003 para T012); pode rodar em paralelo com US1
- **US3 (Phase 5)**: depende de Phase 2; pode rodar em paralelo com US1/US2 (arquivos distintos), mas o teste T018 usa o import novo → preferir após US1 em execução sequencial
- **Polish (Phase 6)**: depende de todas as stories desejadas estarem completas

### User Story Dependencies

- **US1 (P1)**: Foundational → testes T004–T007 (vermelhos) → T008 → T009 → T010
- **US2 (P2)**: independente de US1; T011 (vermelho) → T013 → T014 → T015; T012 usa o módulo foundational
- **US3 (P3)**: Foundational → testes T016–T018 (vermelhos) → T019 → T020 → T021; semanticamente posterior ao MVP (export só faz sentido com o import novo)

### Within Each User Story

- Testes primeiro, vermelhos, antes da implementação (Princípio I)
- Registro/leitor antes do serviço (import/export)
- Implementação antes da integração; `cargo test`/`npm test` verdes fecha a story

### Parallel Opportunities

- T011/T012 podem rodar junto com o par US1 (arquivos distintos)
- T022/T023 [P] no Polish (docs vs comentários)
- Com dois executores: Executor A = US1 → US3; Executor B = US2 → Polish parcial

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. T001 (baseline) → Phase 2 (T002–T003)
2. Phase 3 completa (T004–T010) → **STOP and VALIDATE**: pacote novo importa, FR-014 protege a base
3. Deploy/demo se pronto

### Incremental Delivery

1. Setup + Foundational → foundation pronta
2. US1 → validar (MVP: app utilizável com o pacote novo)
3. US2 → validar (aba Campuses limpa)
4. US3 → validar (round-trip completo no formato novo)
5. Polish → docs + validação quickstart integral

---

## Notes

- Fixture real: `exports_canonical.zip` (395 entradas) é a referência dos testes; fixture legado em `~/Downloads/exports_canonical.zip` (626 entradas) cobre FR-006/R4
- Contagens esperadas no round-trip (data-model.md): export do pacote novo → 395 entradas, `preserved_entries = 380`
- Commit após cada task ou grupo lógico; parar em qualquer checkpoint para validar a story isoladamente
- Evitar: tasks vagas, conflito de mesmo arquivo sem ordem, dependência entre stories que quebre independência

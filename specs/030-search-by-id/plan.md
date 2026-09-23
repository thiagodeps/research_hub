# Implementation Plan: Pesquisa de Registros por ID

**Branch**: `030-search-by-id` | **Date**: 2026-09-23 | **Spec**: [specs/030-search-by-id/spec.md](spec.md)
**Input**: Feature specification from `/specs/030-search-by-id/spec.md`

## Summary

Extend the generic entity-list search so that a purely numeric term also matches the
record's exact ID, on top of the existing case-insensitive text match (FR-001..FR-009).
The rule lives entirely in `crud.rs::list` — the search term classification (digits-only →
ID candidate), the SQL union (`id = ?` OR `col LIKE ?`), the ID-first relevance ordering and
the consistent COUNT all happen in the Rust core. Entities without a searchable text column
gain ID search (currently their search box is inert). The frontend needs **zero changes**:
the `search` parameter already flows from `EntityPage.jsx` through `api.js` to the
`list_entities` command unchanged.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021) / JavaScript (Node.js 20+, React 18, Astro — somente leitura nesta feature)
**Primary Dependencies**: Tauri 2.0 (`tauri`), `rusqlite 0.32` (bundled) — nenhuma dependência nova
**Storage**: SQLite embedded single-file (`hub.db` no app-data do SO) — sem mudança de schema
**Testing**: `cargo test` (unitários/integração com SQLite em memória), `npm test` (Vitest — regressão, sem testes novos)
**Target Platform**: Linux and Windows (desktop application)
**Performance Goals**: Busca por ID exato O(log n) via PK (`id INTEGER PRIMARY KEY`, rowid alias); listagem com busca responde no mesmo patamar de hoje (<100ms nas maiores tabelas, ~4–5 mil linhas)
**Constraints**: 100% offline, arquivo único SQLite, lógica de negócio exclusivamente em Rust, nenhuma migração de banco
**Scale/Scope**: 15 entidades exportadas + admins; alteração concentrada em `src-tauri/src/crud.rs::list`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **I — TDD Obrigatório**: Os testes novos em `crud.rs` (match por ID, união ID+texto, ordenação ID-primeiro, entidades sem coluna pesquisável, zeros à esquerda, termos mistos/não-inteiros, COUNT consistente) entram como primeiro passo das tasks, no ciclo red-green-refactor.
- [x] **II — Paridade Funcional**: O oráculo Python já foi removido do repositório (migração concluída). A busca textual permanece byte-a-byte igual (FR-005: mesmo LIKE case-insensitive, wildcards literais); o comportamento novo (match por ID) está declarado na spec como aprimoramento/diferença esperada.
- [x] **III — Testes Rigorosos**: Núcleo coberto por `cargo test` contra SQLite em memória. Front-end não muda — a suíte Vitest existente deve continuar passando sem alterações (regressão implícita).
- [x] **IV — CRUD Completo**: Reforço apenas de leitura (listagem); criar/editar/deletar/fundir/vincular não são afetados.
- [x] **V — Aplicação Desktop Local**: Sem novo motor de dados, sem rede, sem migração; `hub.db` e pragmas permanecem intocados.
- [x] **VI — Lógica de Negócio em Rust**: Classificação do termo, montagem do SQL e ordenação ficam em `crud.rs`. Nenhum JS novo; `api.js` e `EntityPage.jsx` intocados.
- [x] **Escopo Diferido**: Nada de macOS, assinatura de código, auto-update ou banco alternativo.

## Project Structure

### Documentation (this feature)

```text
specs/030-search-by-id/
├── spec.md              # Feature specification
├── plan.md              # Implementation plan (this document)
├── research.md          # Phase 0 technical decisions & rationale
├── data-model.md        # Phase 1 classification do termo + entidades afetadas
├── quickstart.md        # Phase 1 roteiro de validação
├── contracts/           # Phase 1 contrato IPC da listagem
│   └── api.md
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 output (/speckit.tasks — não criado aqui)
```

### Source Code Layout

```text
src-tauri/
├── src/
│   └── crud.rs          # ÚNICO arquivo alterado: list() classifica o termo,
│                        # monta WHERE/ORDER BY e mantém COUNT consistente;
│                        # testes novos no módulo #[cfg(test)] existente
└── migrations/          # Intocado (sem mudança de schema)

frontend/                # INTACTO — zero mudanças de código
└── src/
    ├── components/EntityPage.jsx   # campo de busca existente, sem alteração
    └── services/api.js             # parâmetro search já flui como está
```

**Structure Decision**: Alteração de um único ponto: `crud.rs::list`. A superfície IPC
(`list_entities`), a ponte `api.js` e a UI já transportam o termo de busca genérico; a
interpretação "número → ID ou texto" é regra de negócio e, pela Constituição VI, pertence
ao núcleo Rust. Sem novos arquivos de código.

## Complexity Tracking

*Nenhuma violação constitucional; nenhuma complexidade arquitetural adicional.*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|:---|:---|:---|
| *None* | *N/A* | *N/A* |

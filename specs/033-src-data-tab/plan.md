# Implementation Plan: Área de Dados do Projeto SRC com Seleção de Projeto

**Branch**: `033-src-data-tab` | **Date**: 2026-09-29 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/033-src-data-tab/spec.md`

## Summary

O Research Hub passa a servir dois projetos de dados de forma separada: o **Horizon** (as 15 tabelas canônicas de hoje, intocadas) e o **SRC** (SRC_ETL), cujos dados vivem em um único arquivo JSON consolidado. Após o login, uma nova página principal (`/projects`) deixa o curador escolher o projeto; cada área de trabalho expõe somente seus dados (FR-004/FR-005). A área SRC carrega o consolidado (`import_src_json`: validação estrutural, snapshot, substituição, progresso e contagem), dá CRUD completo de ações (tabela com busca/ordenação; projeções sincronizadas com o objeto original `raw_json`, garantindo round-trip fiel — SC-003) e de participações aninhadas (público-alvo/equipe com contexto da atividade), e devolve o arquivo com a mesma estrutura do recebido. Decisões-chave em [research.md](./research.md): tabelas dedicadas `src_*` no mesmo SQLite (R1), formato real do consolidado com participações achatadas por pessoa contendo PII local (R2 — correção declarada na spec), raw_json + projeções (R3), contadores recomputados no export (R4), comandos IPC dedicados com Horizon invário (R6).

## Technical Context

**Language/Version**: Rust (edition 2021, ferramentas do `src-tauri/Cargo.toml`) + JavaScript/React no front-end Astro (build estático embarcado)

**Primary Dependencies**: `serde_json 1.0` (parse/remontagem do consolidado — com feature `preserve_order` para manter a ordem das chaves, C9), `rusqlite 0.32` (bundled, storage), `tauri_plugin_dialog` (diálogos de arquivo abertos no Rust — padrão SEP-018), Tauri 2.0 (IPC + eventos)

**Storage**: SQLite em arquivo único no diretório app-data (`rusqlite` bundled; migration nova `002_src_tables.sql` via `PRAGMA user_version`; snapshots pela API `rusqlite::backup` — mesmo mecanismo do `import.rs`)

**Testing**: `cargo test` (unit + integração em SQLite em memória, sem Tauri) e `npm test` (Vitest) + smoke e2e existente (`frontend/tests/e2e`, WebdriverIO + tauri-driver) para o fluxo de seleção de projeto

**Target Platform**: Desktop Linux e Windows (Tauri 2.0)

**Performance Goals**: SC-006 — import de consolidado na ordem de milhares de ações em < 30 s com progresso; troca de projeto < 5 s sem reiniciar (SC-001); listagem paginada como no Horizon

**Constraints**: 100% offline; PII de participações nunca sai do dispositivo nem aparece em logs (C7); falha de importação aborta atomicamente com base intacta (FR-007); registros não editados sobrevivem byte-objetuais ao round-trip (SC-003); nenhum comando genérico do Horizon muda (FR-005)

**Scale/Scope**: 3 tabelas novas (`src_meta`, `src_acoes`, `src_participacoes`), 12 comandos IPC novos, 2 páginas novas + 1 modificação de redirect + 2 layouts; consolidation real de referência ≈ centenas de ações por campus

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Status (pré-Phase 0) | Evidência | Re-check pós-design |
|---|---|---|---|
| I. TDD Obrigatório | ✅ Passa | Testes primeiro para cada regra (fixtures do consolidado, round-trip, validações, sincronização projeção↔raw_json); ver quickstart.md §1 | ✅ Mantido — quickstart lista a cobertura mínima por regra |
| II. Paridade Funcional Antes de Melhoria | ✅ Passa | O oráculo da área SRC é o próprio arquivo consolidado: round-trip importar→exportar comparado objeto a objeto (SC-003, C8 declara a única diferença esperada). Correção factual na spec (R2: participações achatadas com PII, e não contagens) é **declarada** no research e na spec — nada silencioso | ✅ Mantido — contrato C8 registra a diferença esperada |
| III. Testes Rigorosos (Rust e Front-end) | ✅ Passa | `cargo test` (import/export/CRUD/registry, SQLite em memória) + `npm test` (páginas novas, api.js, editor) + smoke e2e do fluxo login→projeto→SRC | ✅ Mantido |
| IV. CRUD Completo e Operações Diretas | ✅ Passa | CRUD completo de ações E de participações (criar/ver/editar/deletar), perfil único de admin, operação direta sem fluxo de aprovação | ✅ Mantido |
| V. Aplicação Desktop Local e Autocontida | ✅ Passa | Mesmo arquivo SQLite único (migration apenas adiciona tabelas `src_*`), zero rede, zero servidor; PII permanece local — reforça o princípio | ✅ Mantido |
| VI. Lógica de Negócio em Rust | ✅ Passa | Parse/validação/remontagem/sincronização raw_json no núcleo; front-end renderiza e invoca; fronteira única em `api.js` | ✅ Mantido |

Nenhuma violação a justificar no Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/033-src-data-tab/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── src-consolidated-json.md   # contrato do arquivo de entrada/saída
│   ├── ipc-commands.md            # comandos IPC novos (Horizon invário)
│   └── frontend-routes.md         # rotas api.js + páginas Astro + ROUTES
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src-tauri/
├── migrations/
│   └── 002_src_tables.sql        # NOVO: src_meta, src_acoes, src_participacoes ( Horizon invário)
├── src/
│   ├── src_domain.rs             # NOVO: tipos + regras do domínio SRC — validação do consolidado,
│   │                             #   sincronização projeção↔raw_json, recomputação de contadores,
│   │                             #   remontagem do export (testável sem Tauri, como os módulos vizinhos)
│   ├── import.rs                 # VERIFICAR: extrair/refatorar snapshot() para reuso (prefixo src-)
│   ├── registry.rs               # MODIFICAR: seção SRC (rotas/colunas/busca das entidades src_*)
│   ├── commands.rs               # MODIFICAR: registrar comandos src_* (contratos/ipc-commands.md)
│   ├── lib.rs                    # MODIFICAR: pub mod src_domain; invoke_handler; ROUTES 18→22
│   ├── crud.rs                   # INVÁRIO: comandos genéricos não enxergam rotas SRC (FR-004/005)
│   └── (demais módulos)          # INVÁRIOS
├── tests/
│   └── fixtures/
│       └── src_consolidado_exemplo.json   # NOVO: 3 ações (com/sem participações, campo extra)
└── testes embutidos em cada módulo (#[cfg(test)])

frontend/
├── src/
│   ├── pages/
│   │   ├── index.astro           # MODIFICAR: autenticado → /projects
│   │   ├── projects.astro        # NOVO: página principal — seleção Horizon | SRC
│   │   └── src/
│   │       ├── index.astro       # NOVO: resumo da base SRC + import/export
│   │       └── acoes.astro       # NOVO: tabela de ações (EntityTable) + CRUD
│   ├── layouts/
│   │   ├── Dashboard.astro       # MODIFICAR: + atalho "Trocar projeto" → /projects
│   │   └── SrcDashboard.astro    # NOVO: menu lateral da área SRC (Resumo, Ações, Trocar projeto)
│   ├── components/
│   │   ├── ProjectSelector.jsx   # NOVO: cartões de seleção de projeto
│   │   ├── ParticipacoesEditor.jsx # NOVO: CRUD aninhado no editor da ação
│   │   └── SrcDataControl.jsx    # NOVO: import/export do JSON (espelha DataControlCenter; progresso src-import://progress)
│   └── services/
│       └── api.js                # MODIFICAR: rotas /src/... (contracts/frontend-routes.md); único ponto de acoplamento
└── tests/
    ├── unit/                     # NOVOS: ProjectSelector, rotas api.js, ParticipacoesEditor
    └── e2e/                      # NOVO smoke: login → /projects → SRC (tauri-driver)
```

**Structure Decision**: estrutura existente do repositório (núcleo Rust em `src-tauri/src/`, front-end em `frontend/src/`), com um módulo novo (`src_domain.rs`) espelhando o padrão dos módulos vizinhos — lógica de domínio testável sem Tauri — e migração aditiva (`002_src_tables.sql`). Nenhuma pasta fora do padrão atual.

## Complexity Tracking

> Vazio — nenhum princípio da Constituição foi violado; nenhuma justificativa necessária.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |

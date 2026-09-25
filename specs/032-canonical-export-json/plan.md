# Implementation Plan: Adaptação ao Novo Export Canonical (Somente JSON + Aba Campus)

**Branch**: `032-canonical-export-json` | **Date**: 2026-09-25 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/032-canonical-export-json/spec.md`

## Summary

O DataLake mudou o pacote `exports_canonical.zip`: as tabelas canônicas agora vêm somente como `{tabela}_canonical.json` na raiz do ZIP (o diretório `parquet/` não existe mais) e o bug do campus aninhado foi corrigido upstream. O Research Hub precisa: (1) importar o formato JSON preservando o comportamento vigente de substituição de base, snapshot, ids explícitos e progresso por tabela — com verificação prévia de completude (as 15 tabelas devem estar presentes, senão aborta sem apagar nada); (2) remover o campo fantasma "Campus (Vínculos)" da aba Campuses (listagem fica `id` + `name`); (3) exportar no novo formato — somente JSON, sem parquet — preservando entradas não gerenciadas byte a byte e restaurando tipos a partir dos tipos inferidos do JSON do pacote original (substituindo o mecanismo de schema de parquet, que deixa de existir). Pacotes legados (parquet + JSON) continuam aceitos com precedência do JSON.

## Technical Context

**Language/Version**: Rust (edition 2021, ferramentas do `src-tauri/Cargo.toml`) + JavaScript/React no front-end Astro (build estático embarcado)

**Primary Dependencies**: `zip 2` (leitura/escrita do arquivo), `serde_json 1.0` (parse dos JSONs canônicos), `arrow 53`/`parquet 53` (mantidos apenas para o fallback legado), `rusqlite 0.32` (bundled, storage), Tauri 2.0 (IPC, diálogos)

**Storage**: SQLite em arquivo único no diretório app-data (`rusqlite` bundled, migrations via `PRAGMA user_version`)

**Testing**: `cargo test` (unit + integração em SQLite), `npm test` (Vitest no front-end)

**Target Platform**: Desktop Linux e Windows (Tauri 2.0)

**Project Type**: desktop-app

**Performance Goals**: import do pacote real (≈400 MB descomprimido, ~60 mil registros em researchers) em até 2 minutos no hardware de referência (SC-001)

**Constraints**: funciona 100% offline; armazenamento todo-TEXT no SQLite (exceto `id`); falhas de import/export abortam atomicamente sem corromper a base; entradas não gerenciadas do pacote devem sobreviver byte a byte (sem recompressão)

**Scale/Scope**: 15 tabelas gerenciadas + admins; pacote real com ~395 entradas e ≈590 mil linhas somadas; 1 aba de UI afetada (Campuses)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Status | Evidência |
|---|---|---|
| I. TDD Obrigatório | ✅ Passa | Plano exige testes primeiro para cada regra nova (fixtures JSON, abort por arquivo ausente, round-trip); ver quickstart.md |
| II. Paridade Funcional Antes de Melhoria | ✅ Passa | O Python de referência foi descomissionado (SEP-024); o oráculo agora é o próprio pacote novo do DataLake: round-trip importar→exportar comparado entrada a entrada. Remoção do parquet no export é diferença **declarada** pela spec (FR-009), não silenciosa |
| III. Testes Rigorosos (Rust e Front-end) | ✅ Passa | `cargo test` cobre import/export/registry; `npm test` cobre a página Campuses (colunas declaradas) |
| IV. CRUD Completo e Operações Diretas | ✅ Passa | CRUD de campuses intacto; única mudança de UI é a coluna fantasma (dados reais permanecem editáveis) |
| V. Aplicação Desktop Local e Autocontida | ✅ Passa | Nenhum servidor, nenhuma rede; SQLite em arquivo único; migração nova destrutiva **não** é necessária |
| VI. Lógica de Negócio em Rust | ✅ Passa | Todo o parse/escrita de JSON e a regra de completude ficam no núcleo Rust; front-end só renderiza e invoca |

Nenhuma violação a justificar no Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/032-canonical-export-json/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src-tauri/
├── src/
│   ├── import.rs        # MODIFICAR: descoberta {tabela}_canonical.json + fallback parquet + verificação de completude (FR-014) antes do snapshot/DELETE
│   ├── export.rs        # MODIFICAR: gera somente {tabela}_canonical.json; tipos inferidos do JSON do original; sem parquet
│   ├── parquet_io.rs    # MANTER (fallback legado); nova leitura JSON entra em json_io.rs
│   ├── json_io.rs       # NOVO: parse de {tabela}_canonical.json → Table (mesma forma de parquet_io::Table) + inferência de tipos por coluna
│   ├── registry.rs      # MODIFICAR: remover coluna `campus` da EntityDef de campuses
│   └── commands.rs      # INVÁRIO: import_canonical_zip/export_canonical_zip, mesmos summaries e eventos
├── migrations/          # INVÁRIO: sem migração destrutiva (coluna `campus` de campuses fica dormente)
└── tests embutidos em cada módulo (#[cfg(test)])
frontend/
├── src/pages/dashboard/campuses.astro   # MODIFICAR: columns={['id','name']}; remover formField 'campus' json_readonly
└── src/components/EntityForm.jsx        # VERIFICAR: campos derivam do registry; página Campuses não deve mais renderizar o campo fantasma
```

**Structure Decision**: estrutura existente do repositório (núcleo Rust em `src-tauri/src/`, front-end em `frontend/src/`), sem diretórios novos além do módulo `json_io.rs` espelhando `parquet_io.rs` — o leitor JSON fica testável sem Tauri, como os módulos vizinhos.

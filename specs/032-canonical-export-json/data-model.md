# Data Model: Adaptação ao Novo Export Canonical (Somente JSON + Aba Campus)

**Branch**: `032-canonical-export-json` | **Date**: 2026-09-25

## Entidades

### 1. Pacote Canônico (`exports_canonical.zip`) — artefato de fronteira

| Aspecto | Formato novo (v2) | Formato legado (v1) |
|---|---|---|
| Tabela canônica gerenciada | `{tabela}_canonical.json` na raiz | `parquet/{tabela}_canonical.parquet` **e** `{tabela}_canonical.json` |
| Entradas totais observadas | 395 (26 JSONs canônicos, zero parquet) | 626 (30 gerenciados + 596 não gerenciados) |
| Entradas não gerenciadas | grafos, trackings, marts, `data_snapshot.zip`, relatórios, PDFs, `*_cols.json` | idem |
| Papel no sistema | fonte da base no import; template de tipos no export; base do pacote devolvido | idem (aceito por compatibilidade, FR-006) |

Regras (FR-001, FR-006, FR-010, FR-014, R1, R4):

- Por tabela gerenciada: JSON tem precedência; parquet é fallback; **nenhuma das duas → import aborta** (base intacta, snapshot não é criado).
- No export: `{tabela}_canonical.json` é regenerado para as 15 tabelas; `parquet/{tabela}_canonical.parquet` nunca é gerado e, se existir no original (legado), é **descartado** (resíduo declarado do formato antigo).
- Toda entrada que não seja JSON canônico gerado nem parquet legado de tabela gerenciada é copiada byte a byte, sem recompressão.

### 2. Tabela Canônica Gerenciada (15 + admins excluídos)

Fonte única de colunas: `registry::ENTITIES` (`src-tauri/src/registry.rs`). Nenhuma mudança de colunas nestas entidades nesta feature, exceto a listada abaixo. Armazenamento: tudo TEXT (exceto `id INTEGER PRIMARY KEY`); `null` → SQL NULL; booleanos como `"True"/"False"`; números em forma decimal textual; vínculos como texto JSON.

**Mudança única**: `campuses` perde a coluna `campus` do registry (fica dormente no schema físico; ver Entidade 3).

### 3. Campus

| Campo | Tipo no pacote | No SQLite |
|---|---|---|
| `id` | inteiro | `INTEGER PRIMARY KEY` |
| `name` | texto \| null | TEXT |
| `description` | texto \| null | TEXT |
| `short_name` | texto \| null | TEXT |
| `organization_id` | inteiro \| null | TEXT |
| `parent_id` | inteiro \| null | TEXT (autorreferência) |
| ~~`campus`~~ | ~~objeto aninhado (bug v1)~~ | coluna física dormente — ignorada em leitura/escrita |

Validações (FR-007, FR-008): listagem exibe exatamente `id` + `name`; formulário sem campo fantasma; dado aninhado de pacote antigo é descartado sem erro e nunca reaparece no export. Volume atual do pacote: 23 registros.

### 4. Tipo de Coluna (inferido, não persistido)

Entidade efêmera: no export, para cada tabela gerenciada, lê-se o `{tabela}_canonical.json` do `original.zip` e o **primeiro valor não-null** de cada coluna define o tipo de conversão do texto armazenado:

| Tipo inferido | Conversão na saída |
|---|---|
| booleano | `as_bool` (`1/True/true/0/False/false` → booleano; outro valor → null, coerção) |
| número | parse `i64`, com fallback `f64` (`"12.0"` → `12.0`); inválido → null (coerção) |
| lista/objeto | revalida como JSON; inválido → string |
| texto/ausente | string textual (null → JSON null) |

Sem `original.zip`: tudo sai como texto (caminho degradado vigente).

### 5. Snapshot (`snapshots/hub-<epoch>.db`)

Sem mudança de modelo. Nova regra de ordem (R7): só é criado **após** a validação de completude (FR-014) e antes dos `DELETE`.

### 6. Sumários IPC (`ImportSummary` / `ExportSummary`)

Contrato com o front-end inalterado: `tables: [{table, rows}]`, `total_rows`, `snapshot` / `tables`, `preserved_entries`, `path`. Eventos de progresso por tabela mantidos. Semântica nova dentro dos mesmos campos: um import abortado por FR-014 **não** emite sucesso parcial (erro tipado com o nome da tabela faltante); `preserved_entries` no export passa a excluir os parquets legados descartados (R4).

## Transições de estado da base

```text
[base atual] --import válido--> [snapshot opcional] --> [DELETE 15 tabelas] --> [carga transacional] --> [nova base + original.zip atualizado]
[base atual] --import inválido (zip/JSON malformado/tabela faltante)--> [base idêntica ao anterior, sem snapshot]
[base] --export--> [pacote v2: 15 JSONs + não gerenciados do original] (nada muda na base)
```

## Mapeamento requisito → mecanismo

| Requisito | Mecanismo (research.md) |
|---|---|
| FR-001, FR-006 | R1 (descoberta JSON → parquet) |
| FR-002, FR-003 | R5, R6 (conversão TEXT e campos extras/ausentes) |
| FR-004 | comportamento vigente reafirmado + R7 |
| FR-005 | R1/R7 (validação + transação única) |
| FR-014 | R1/R7 (verificação prévia antes de snapshot/DELETE) |
| FR-007, FR-008 | R3 (registry + página Campuses, coluna dormente) |
| FR-009, FR-010 | R4 (saída JSON-only, preservação seletiva) |
| FR-011, FR-012 | R2, R5 (inferência de tipos no export) |
| FR-013 | comportamento vigente do export (erro aborta, nada é omitido silenciosamente) |

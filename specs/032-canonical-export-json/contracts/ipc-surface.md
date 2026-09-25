# Contract: Superfície IPC (comandos Tauri)

**Branch**: `032-canonical-export-json` | **Date**: 2026-09-25

A superfície JS→Rust **não muda** nesta feature. Documentada aqui como baseline de não-regressão: o front-end continua chamando apenas via `frontend/src/services/api.js`.

## `import_canonical_zip`

- Assinatura: `(path?: string) => Promise<ImportSummary | null>` (sem `path`, abre diálogo nativo no Rust; cancelamento → `null`).
- `ImportSummary`: `{ tables: Array<{ table: string, rows: number }>, total_rows: number, snapshot: string | null }`.
- Eventos de progresso: um por tabela carregada (mesmo payload `TableLoaded`).
- **Novidade semântica**: erro tipado quando faltar o arquivo canônico de uma tabela gerenciada (FR-014) — a mensagem nomeia a tabela e a base permanece intacta. O resumo de sucesso continua sendo emitido só após o commit.

## `export_canonical_zip`

- Assinatura: `() => Promise<ExportSummary | null>` (diálogo de salvar no Rust, default `portal_export_canonical.zip`; cancelamento → `null`).
- `ExportSummary`: `{ tables: Array<{ table: string, rows: number }>, preserved_entries: number, path: string }`.
- **Novidade semântica**: `preserved_entries` exclui os parquets legados descartados (R4); o pacote gerado não contém `parquet/`.

## Entidades (CRUD)

Nenhuma mudança de rotas ou payloads. Única diferença visível: a entidade `campuses` deixa de aceitar/exibir o campo `campus` (removido do registry); as demais continuam com seus campos, inclusive o vínculo `campus` legítimo de researchers/students/artifacts etc.

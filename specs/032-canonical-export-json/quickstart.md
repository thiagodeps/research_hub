# Quickstart: Validação da Adaptação ao Novo Export Canonical

**Branch**: `032-canonical-export-json`

Guia de validação ponta a ponta. Os cenários seguem os da spec (`spec.md`) e os mecanismos de `research.md`; os detalhes de implementação ficam em `tasks.md`.

## Pré-requisitos

- Rust toolchain estável e Node.js (mesmos usados no repo).
- Fixture: `exports_canonical.zip` (formato novo, JSON-only) na raiz do repositório — usada pelos testes de referência.
- Para o cenário legado: um pacote no formato antigo (com `parquet/` + JSON), como `~/Downloads/exports_canonical.zip`.

## 1. Suíte automatizada (TDD, Princípio I)

```bash
cargo test        # núcleo Rust: import JSON, fallback legado, FR-014, export JSON-only, round-trip
npm test          # front-end: página Campuses sem o campo fantasma
```

Resultados esperados (novos testes que provam a feature — escritos antes da implementação):

- **Import JSON**: pacote com `{tabela}_canonical.json` na raiz carrega as 15 tabelas; ids explícitos preservados; progresso por tabela; admins intactos; snapshot criado quando havia dados.
- **FR-014 (Q1)**: pacote sem o JSON (e sem parquet) de uma tabela gerenciada → erro nomeando a tabela, **nenhum snapshot**, base com os dados anteriores intactos.
- **Fallback legado (Q: precedência)**: pacote v1 (parquet + JSON) carrega as 15 tabelas com as mesmas contagens de hoje; JSON prevalece quando ambos existem.
- **Export JSON-only**: saída contém as 15 `{tabela}_canonical.json`, zero `parquet/`; entradas não gerenciadas byte a byte; parquet legado do original descartado (R4).
- **Tipos (R2/R5)**: `true` volta booleano, `23` número, `23.0` decimal, `null` null, vínculos como estrutura; coluna sem referência sai como string.
- **Registry/página**: `dashboard_pages_only_declare_columns_that_exist` passa com Campuses declarando só `id`/`name`.

## 2. Validação manual end-to-end (app desktop)

```bash
npm run tauri dev
```

1. **Import do pacote novo**: arraste ou escolha o `exports_canonical.zip` (JSON-only).
   - Esperado: progresso tabela a tabela; resumo com 15 tabelas; snapshot informado (se havia base); Campus mostra 23 registros com colunas **id** e **nome** apenas — sem "Campus (Vínculos)".
2. **Aba Campuses**: abrir a listagem e o formulário de criar/editar.
   - Esperado: sem campo fantasma; nome/descrição/sigla/organização/campus-pai editáveis; busca por nome funcionando.
3. **Export sem edição**: exportar logo após o import.
   - Esperado: pacote com as 15 JSONs regeneradas, sem parquet; grafos/trackings/`data_snapshot.zip` preservados; comparar `campuses_canonical.json` do export com o do original (mesmos 23 registros, mesmos tipos).
4. **Round-trip com curadoria**: editar o nome de um campus, exportar.
   - Esperado: só o valor editado muda no JSON; tipos dos demais campos preservados.
5. **Falha controlada**: importar um ZIP sem `researchers_canonical.json`.
   - Esperado: erro nomeando `researchers`; base anterior intacta (reabrir e conferir os dados).

## 3. Critérios de aceite (da spec)

- SC-001: 15 tabelas, contagens iguais aos JSONs, ≤ 2 min no hardware de referência.
- SC-002: round-trip sem edição → 15 JSONs, zero parquet gerado, 100% das não gerenciadas idênticas, tipos preservados.
- SC-003: nenhuma menção a "Campus (Vínculos)" na listagem/formulário de Campuses.
- SC-004: pacote legado importa com as mesmas contagens da versão atual.
- SC-005: todos os cenários de falha deixam a base idêntica ao anterior (cobertos por teste).
- SC-006: 100% das regras novas com teste primeiro.

Ver também: [contracts/canonical-archive.md](./contracts/canonical-archive.md) · [contracts/ipc-surface.md](./contracts/ipc-surface.md) · [data-model.md](./data-model.md)

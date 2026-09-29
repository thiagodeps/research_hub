# Quickstart: Validação ponta a ponta — Área SRC com Seleção de Projeto

**Branch**: `033-src-data-tab` | **Spec**: [spec.md](./spec.md) | **Contratos**: [contracts/](./contracts/)

## Pré-requisitos

- Rust + `cargo` (toolchain do `src-tauri/`), Node/npm (front-end em `frontend/`)
- Fixture de teste: `src-tauri/tests/fixtures/src_consolidado_exemplo.json` (criada na implementação — 3 ações: uma com participações de ambos os tipos, uma sem participações, uma com chave de rótulo extra e campo ausente; ver `contracts/src-consolidated-json.md`)
- Opcional: um `*_consolidado.json` real produzido por `src-etl-consolidate` no projeto SRC — **contém PII; manter local, nunca commitar**

## 1. Verificação automatizada (TDD — os testes são escritos primeiro)

```bash
cargo test          # núcleo Rust: round-trip, validações, CRUD, snapshot, registry
npm test            # Vitest: /projects, api.js, editor de participações
```

**Round-trip (asserção central — SC-003)**: o teste carrega a fixture, exporta sem editar e compara objeto a objeto (chaves, ordem, valores) com a original, inclusive contadores da raiz (exceto o caso declarado C8).

**Cobertura mínima exigida** (`cargo test`):
- Import válido → contagens corretas (ações, participações, tipos) — SC-002
- Import inválido (não-JSON, sem `acoes`, `acao_id` ausente/duplicado) → erro claro e base intacta — FR-007
- Import sobre base existente → snapshot criado e informado — FR-006
- `acoes: []` → base vazia informada — edge case
- CRUD de ação → projeções e `raw_json` sincronizados; `acao_id` preservado
- CRUD de participações → ordem (`ord`) e chaves canônicas corretas
- Exclusão de ação referenciada como "Ação vinculante" → `Conflict` com filhas listadas — FR-013
- `registry.rs`: rotas SRC não vazam para os comandos genéricos do Horizon — FR-004/FR-005

## 2. Cenário manual ponta a ponta

```bash
npm run dev          # ou o fluxo de build/roda do repositório (tauri dev)
```

| Passo | Ação | Resultado esperado |
|---|---|---|
| 1 | Login como admin | Redireciona para **/projects** (não mais direto à dashboard) |
| 2 | Sem escolher projeto | Nenhuma base carregada (FR-002) |
| 3 | Escolher **Horizon** | Dashboard atual, 15 abas, zero conteúdo SRC (FR-005/SC-005) |
| 4 | "Trocar projeto" → escolher **SRC** | Área SRC: resumo vazio + botões de import/export — sem nenhum dado Horizon |
| 5 | Importar a fixture do consolidado | Progresso exibido; resumo informa contagens (SC-002) |
| 6 | Reimportar a mesma fixture | Aviso de snapshot da base anterior; base substituída, não mesclada (FR-006) |
| 7 | Abrir **Ações** | Tabela com título/processo/natureza/tipo/coordenador; busca e ordenação funcionam (FR-010) |
| 8 | Editar o título de uma ação | Persiste; listagem e export refletem (FR-011/012) |
| 9 | No editor da ação: adicionar/remover participações | Linhas criadas com as chaves canônicas; ordem preservada |
| 10 | Tentar excluir ação usada como "Ação vinculante" | Aviso com as filhas; confirmação exige escolha explícita (FR-013) |
| 11 | Exportar sem editar nada (base recém-importada) | Arquivo equivalente campo a campo ao importado (SC-003; conferir com `diff`/`jq`) |
| 12 | Trocar para Horizon com formulário aberto no SRC | Aviso de mudança não salva antes de prosseguir (US1-5) |

## 3. Verificação de fidelidade do export

```bash
jq -S . importado.json > a.json && jq -S . exportado.json > b.json && diff a.json b.json
# base recém-importada, sem edições: diff vazio (SC-003)
```

Para o caso com edição: verificar que só o campo editado/registro tocado difere.

## 4. Critérios de aceite da feature

- [ ] `cargo test` e `npm test` verdes (incl. round-trip e validações)
- [ ] Cenário manual completo, sem nenhum dado cruzando entre projetos
- [ ] Export da base intata idêntico ao import (SC-003)
- [ ] Nenhum log/output exibindo PII de participações (contrato C7)

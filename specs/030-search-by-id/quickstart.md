# Quickstart: Pesquisa de Registros por ID

**Feature**: 030-search-by-id | **Date**: 2026-09-23

Roteiro de validação ponta-a-ponta. Implementação detalhada fica em `tasks.md`/
`contracts/api.md`; cenários funcionais vêm da spec (User Stories 1–3 e edge cases).

## Pré-requisitos

- Toolchain Rust (edition 2021) e Node.js 20+.
- Banco com dados de exemplo: rode a importação do pacote canônico pela interface
  (ou use o banco local já populado). Referências de dados: pesquisadores ~4 mil linhas.

## Validação automatizada (obrigatória — Constituição I/III)

```bash
# Núcleo — roda a suíte inteira, incluindo os testes novos de crud.rs:
cd src-tauri && cargo test

# Front-end — regressão: a suíte existente deve passar SEM alterações:
cd frontend && npm test
```

Cenários que os testes novos de `crud.rs` devem cobrir (red → green):

1. Termo numérico retorna o registro do ID exato como **primeiro** item (`total` coerente).
2. Termo numérico que também ocorre no texto traz **união** sem duplicar o registro do ID.
3. Termo textual clássico (case-insensitive, `100%`/`_` literais) — resultados idênticos aos
   atuais (regressão, FR-005).
4. Entidade sem `search_column` (ex.: `proficiencies`): termo textual ignora filtro;
   termo numérico filtra por ID.
5. Zeros à esquerda (`"007"` → ID 7); termos mistos (`"abc123"`), negativos, decimais e
   overflow → somente busca textual.
6. `total` e paginação (`LIMIT`/`OFFSET`) consistentes com o filtro combinado.
7. Ordenação do usuário (`sort`/`order`, NULLs por último) preservada para os não-matches
   de ID.

## Validação manual (E2E, app rodando)

```bash
npm run tauri dev
```

Login: `admin@admin.com` / `admin123` (seed padrão) ou conta criada na feature 029.

| # | Passo | Resultado esperado |
|---|---|---|
| 1 | Abra **Pesquisadores**, anote o ID de um registro na tabela | — |
| 2 | Digite esse ID no campo "Buscar registro…" | O registro aparece **em primeiro** na lista (US-1, SC-003) |
| 3 | Digite o nome (ou parte) de um pesquisador | Mesmos resultados de antes da feature (US-2, SC-002) |
| 4 | Digite um número que também exista em nomes | Lista traz o registro do ID primeiro + os textuais (FR-004) |
| 5 | Digite um ID inexistente (ex.: `9999999`) | Lista vazia com a mensagem padrão, sem erro (US-1 cenário 3) |
| 6 | Limpe o campo | Listagem integral volta (US-1 cenário 2) |
| 7 | Repita os passos 2 e 5 em **Produções Científicas** e em uma entidade sem busca textual | Comportamento idêntico — a busca por ID funciona em todas (US-3, SC-004) |
| 8 | Digite `abc123` | Somente resultados textuais; sem erro (edge case) |
| 9 | Com a busca por ID ativa, ordene por uma coluna clicando no cabeçalho | Registro do ID segue primeiro; demais seguem a ordenação escolhida (D3) |
| 10 | Navegue entre páginas de resultado com a busca ativa | Paginação e contagem coerentes (FR-008) |

## Critério de aceite final

- `cargo test` e `npm test` 100% verdes, com os cenários 1–7 presentes na suíte.
- Tabela manual acima sem desvios.
- Nenhum arquivo alterado fora de `src-tauri/src/crud.rs` (e seus testes) — verifique com
  `git status`.

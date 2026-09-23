# Data Model: Pesquisa de Registros por ID

**Feature**: 030-search-by-id | **Date**: 2026-09-23

Nenhuma entidade nova, nenhuma coluna nova, nenhuma migração. Este documento registra o
modelo de dados tocado pela feature e a **gramática de classificação do termo de busca**,
que é o único comportamento novo observável.

## Entidades afetadas

### Registro de entidade (todas as tabelas do registry)

| Aspecto | Valor |
|---|---|
| PK | `id INTEGER PRIMARY KEY` (rowid alias) — em todas as tabelas, `001_init.sql` |
| Coluna textual de busca | `search_column` do `registry.rs` (`name`/`title`/`username`/`institution_name`) — presente na maioria; `proficiencies` é a única sem |
| Mudança de schema | **Nenhuma** |
| Mudança de dados | **Nenhuma** (só leitura) |

Entidades (routes): `users`, `researchers`, `students`, `articles`, `research_groups`,
`initiatives`, `advisorships`, `awards`, `campuses`, `organizations`, `proficiencies`,
`languages`, `proficiencies`-familia, `research_productions`, `knowledge_areas` etc. — a
lista canônica é `registry::exported()` e a regra vale para todas, sem exceção por entidade.

## Gramática de classificação do termo de busca

Entrada: `search: Option<String>` (termo livre, já aparado — ver research.md D6).

| Regra | Condição | Efeito |
|---|---|---|
| T1 — vazio | `None` ou vazio após trim | Sem filtro (listagem integral), como hoje |
| T2 — numérico | apenas dígitos ASCII `0-9` **e** parse válido para `i64` | Match por igualdade exata `id = N` **E** match textual `col LIKE '%N%'` (união); registro de ID `N` ordenado primeiro |
| T3 — numérico sem coluna textual | T2 **e** entidade sem `search_column` | Somente match por `id = N` |
| T4 — textual | qualquer outro caso (letras, símbolos, misto, negativo, decimal, overflow) | Somente match textual `col LIKE '%termo%' ESCAPE '\'`, idêntico ao atual |

### Exemplos de classificação

| Termo | Classificação | Resultado esperado |
|---|---|---|
| `"42"` | T2/T3 | Registro de ID 42 (primeiro) + registros com "42" no texto |
| `"007"` | T2 | Registro de ID 7 (parse normaliza) + textuais com "007" |
| `"abc123"` | T4 | Somente textuais |
| `"-3"` | T4 | Somente textuais (símbolo não-dígito) |
| `"12.5"` | T4 | Somente textuais |
| `"99999999999999999999999999"` (overflow) | T4 | Somente textuais |
| `"100%"` | T4 | Somente textuais, `%` literal (comportamento atual preservado) |
| `"  42  "` | T2 (após trim) | Idem `"42"` |

## Regras de validação (do spec → modelo)

- **FR-002** → regra T2: apenas dígitos; `parse::<i64>()` após a verificação de dígitos
  (parse sozinho aceitaria `-3`/`+7`).
- **FR-003** → igualdade exata `id = ?`; sem `CAST(id AS TEXT) LIKE`, sem `LIKE` no id.
- **FR-004** → união via `OR` na mesma query (uma varredura, COUNT consistente).
- **FR-005** → ramo textual do WHERE byte-a-byte o atual (`LIKE ?1 ESCAPE '\'` com wildcards
  escapados por `escape_like`).
- **FR-008** → COUNT e SELECT compartilham o mesmo WHERE; `LIMIT ?/OFFSET ?` intactos.

## Ordenação

- Sem termo ou termo textual: ordenação atual inalterada (padrão do banco ou
  `sort`/`order` do usuário, com `(expr IS NULL)` primeiro para NULLs por último).
- Termo com match por ID: `ORDER BY (id = ?N) DESC, {ordenacao_existente}` — SC-003.
  A expressão `(id = ?N)` vale 1 para o registro do ID e 0 para os demais.

## Transições de estado

Não aplicável — a feature é somente leitura; nenhum estado de entidade muda.

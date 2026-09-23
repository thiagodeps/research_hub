# Research: Pesquisa de Registros por ID

**Feature**: 030-search-by-id | **Date**: 2026-09-23
**Pré-requisito**: não há NEEDS CLARIFICATION na spec — todos os defaults foram assumidos
e documentados em Assumptions. Este documento registra as decisões técnicas correspondentes.

## D1 — Quando o termo é tratado como ID

**Decision**: O termo é candidato a ID somente se for **não-vazio e composto apenas por
dígitos ASCII** (`term.chars().all(|c| c.is_ascii_digit())`), convertido com
`str::parse::<i64>()`. Falha de parse (overflow, ex.: número de 30 dígitos) degrada para
busca puramente textual.

**Rationale**: FR-002 exige "apenas dígitos". `parse::<i64>()` sozinho aceitaria `-3` e
`+7` (FR-002 e edge cases da spec excluem negativos e símbolos) e aceitaria overflow com
erro, por isso a verificação de dígitos precede o parse. Zeros à esquerda ("007") parseiam
naturalmente como 7, como assumido na spec.

**Alternatives considered**:
- `parse::<i64>()` direto — rejeitado: aceitaria negativos/sinal, violando FR-002.
- Regex `^\d+$` — equivalente, mais pesado em dependência/expressão; verificação de
  `chars()` é idiomática e sem custo.
- Tratar overflow clampeando para `i64::MAX` — rejeitado: criar falso match; degradação
  para texto é previsível e inofensiva.

## D2 — Forma do SQL (união ID + texto)

**Decision**: Uma única query com WHERE combinado, parametrizada:

- Termo textual (comportamento atual, intocado):
  `WHERE {col} LIKE ?1 ESCAPE '\'`
- Termo numérico em entidade com coluna pesquisável (FR-004, união):
  `WHERE (id = ?1 OR {col} LIKE ?2 ESCAPE '\')`
- Termo numérico em entidade **sem** coluna pesquisável (FR-006):
  `WHERE id = ?1`
- Sem termo: sem WHERE (como hoje).

O COUNT usa exatamente o mesmo WHERE, mantendo `total` e a paginação consistentes (FR-008).

**Rationale**: Uma query só preserva paginação/ordenação/COUNT atuais sem nova varredura de
tabela. `id = ?` usa a PK (rowid alias) — O(log n). O `LIKE ? ESCAPE '\'` com o termo entre
`%...%` e wildcards escapados permanece byte-a-byte o de hoje (FR-005), inclusive o
case-insensitive padrão do SQLite para ASCII.

**Alternatives considered**:
- Duas queries (uma por ID, outra por texto) com merge em Rust — rejeitado: duplica
  paginação/ordenação e COUNT, mais superfície de erro.
- `FTS5` (full-text search) — rejeitado: cria tabela sombra/índice novo para um requisito
  de igualdade exata em PK; complexity tracking violaria o espírito da Constituição IV/V
  sem ganho.
- `CAST(id AS TEXT) LIKE '%termo%'` para ID parcial — rejeitado: FR-003 exige igualdade
  exata e a spec lista correspondência parcial como fora de escopo; além de invalidar o
  uso da PK.

## D3 — Ordenação: registro por ID primeiro (SC-003)

**Decision**: Quando o termo tem match por ID, prefixa-se a ordenação com relevância:
`ORDER BY (id = ?N) DESC, {ordenacao_existente}`. O restante da lista mantém a ordenação
atual (padrão do banco ou a escolhida pelo usuário via sort/order, incluindo a regra de
NULLs por último).

**Rationale**: SC-003 exige o registro do ID na primeira posição em 100% dos casos — com o
comportamento aditivo (FR-004), o mesmo termo pode trazer matches textuais, e sem o boost
de relevância a posição do registro dependeria da ordem natural.

**Alternatives considered**:
- Ignorar ordenação de relevância e contar com "normalmente é o único resultado" —
  rejeitado: viola SC-003 no cenário da união (FR-004), que é explícito na spec.
- Ordenar tudo por proximidade de relevância textual — rejeitado: sem suporte nativo barato
  no SQLite e fora do escopo (não há requisito de ranking textual).
- Colocar o match de ID primeiro só quando não há ordenação explícita do usuário —
  rejeitado: regra condicional ambígua para o usuário; SC-003 não faz essa ressalva.

## D4 — Entidades sem coluna pesquisável (FR-006)

**Decision**: A busca numérica funciona para **todas** as entidades do registry; para as
sem `search_column` (hoje `proficiencies`), o WHERE é somente `id = ?`. O teste existente
`search_is_ignored_for_entities_without_search_column` é atualizado: termo **textual**
continua ignorado (sem WHERE), termo **numérico** passa a filtrar por ID.

**Rationale**: Todo registro tem ID (PK em todas as tabelas, `001_init.sql`), o que torna a
regra uniforme e resgata páginas com busca inerte, conforme User Story 3.

**Alternatives considered**:
- Replicar o comportamento atual (busca totalmente inerte nessas entidades) — rejeitado:
  contraria US-3 e FR-006 explicitamente.
- Adicionar coluna pesquisável às entidades sem ela — rejeitado: mudança de escopo
  (produto de dados) e de paridade de colunas; não é pedido.

## D5 — Impacto no frontend e no contrato IPC

**Decision**: **Zero mudanças** no frontend e no contrato. `EntityPage.jsx` já envia o
termo livre via `search`, `api.js` já o repassa a `list_entities`, e o comando Rust já o
recebe como `Option<String>`. A classificação numérica é interna do núcleo.

**Rationale**: Constituição VI — regra de negócio em Rust; a fronteira IPC não precisa
saber distinguir ID de texto (e não deve: manter a responsabilidade em um só lugar evita
duplicação). Regressão garantida pelos testes Vitest existentes, que devem passar sem
alteração.

**Alternatives considered**:
- Novo parâmetro `search_id` ou campo de busca separado — rejeitado: FR-007 exige campo
  único; também adicionaria superfície IPC sem necessidade.
- Classificar o termo no JS e mandar `id` separado — rejeitado: duplicaria a regra na
  fronteira (violação direta do Princípio VI).

## D6 — Espaços em branco no termo

**Decision**: O termo é aparado (`trim`) antes da classificação: `"  42  "` é tratado como
`"42"` (ID); termo que vira vazio após o trim é ignorado (sem filtro), como hoje. O LIKE
textual continua usando o termo aparado.

**Rationale**: Colar um ID com espaço acidental é o erro de digitação mais comum; trim é
imperceptível quando desnecessário. O comportamento atual já ignora termo vazio.

**Alternatives considered**:
- Não aparar — rejeitado: `"42 "` (com espaço) cairia só no LIKE e o usuário não
  entenderia por que o ID "não funciona"; a inconsistência seria reportada como bug.
- Aparar também no frontend — rejeitado: regra em dois lugares (Princípio VI).

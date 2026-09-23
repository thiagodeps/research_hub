# Feature Specification: Pesquisa de Registros por ID

**Feature Branch**: `030-search-by-id`

**Created**: 2026-09-23

**Status**: Draft

**Input**: User description: "alem da pesquisa por nome , precisa permitir que faça pesquisa por id"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.

  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Localizar um registro pelo seu ID (Priority: P1)

O Admin está na página de listagem de uma entidade (ex.: Pesquisadores) e sabe o número
identificador (ID) de um registro — por exemplo, porque o anotou anteriormente, recebeu-o de
um colega ou o viu em uma exportação. Ao digitar esse número no campo de busca que já existe
na página, a lista passa a exibir o registro correspondente a esse ID, sem que o Admin precise
navegar entre páginas ou decorar em qual lista o registro está.

**Why this priority**: É o núcleo do pedido. Hoje o campo de busca só compara o termo com a
coluna textual da entidade (nome/título/usuário), então um ID digitado quase nunca encontra o
registro desejado, obrigando o Admin a localizar o registro manualmente.

**Independent Test**: Pode ser testado abrindo qualquer página de listagem, digitando o ID
numérico de um registro existente e verificando que o registro aparece na lista de resultados.

**Acceptance Scenarios**:

1. **Given** que existe um registro com ID 42 em uma entidade, **When** o Admin digita "42" no
   campo de busca da listagem dessa entidade, **Then** o registro de ID 42 aparece nos
   resultados, na primeira posição.
2. **Given** que o Admin digitou um ID no campo de busca, **When** ele limpa o campo, **Then**
   a listagem volta a exibir todos os registros da entidade.
3. **Given** que o Admin digita um ID que não corresponde a nenhum registro da entidade,
   **When** a busca é executada, **Then** a lista fica vazia com a mensagem padrão de
   "nenhum registro encontrado", sem erro.

---

### User Story 2 - A busca textual continua funcionando como hoje (Priority: P2)

O Admin que já usa o campo de busca para pesquisar por nome/título não percebe nenhuma
mudança: digitar um nome continua filtrando pelos registros cujo texto contém o termo, com o
mesmo comportamento de hoje (sem diferenciar maiúsculas de minúsculas, tratando caracteres
especiais como texto comum). Quando o termo digitado é um número, os resultados passam a
incluir tanto os registros que batem pelo ID quanto os que batem pelo texto — ou seja, a
capacidade nova é aditiva, nunca substitui a anterior.

**Why this priority**: Protege a funcionalidade existente: a busca por ID não pode degradar
nem mudar os resultados da busca textual que o Admin já utiliza diariamente.

**Independent Test**: Pode ser testado repetindo buscas textuais conhecidas (por nome e com
caracteres especiais) antes e depois da mudança e comparando que os resultados permanecem os
mesmos, além de um termo numérico que também ocorra em nomes retornar ambos os tipos de match.

**Acceptance Scenarios**:

1. **Given** que existem registros com nomes contendo "Ana", **When** o Admin busca "ana",
   **Then** os mesmos registros de antes são retornados (busca textual sem diferenciar
   maiúsculas de minúsculas).
2. **Given** que existe um registro com ID 7 e outro com "7" no nome, **When** o Admin busca
   "7", **Then** ambos aparecem nos resultados (match por ID e match textual somados na mesma
   lista).
3. **Given** que o Admin busca um termo com caractere especial (ex.: "100%"), **When** a busca
   é executada, **Then** o comportamento permanece o de hoje: o termo é tratado como texto
   literal.

---

### User Story 3 - Pesquisa por ID disponível em todas as entidades (Priority: P3)

Independentemente da entidade em que o Admin esteja — inclusive nas que hoje não têm coluna
textual pesquisável, onde o campo de busca atualmente não tem efeito — digitar um ID numérico
traz o registro correspondente. Todo registro do sistema tem um ID, então a busca por ID
funciona de forma uniforme em todas as páginas de listagem.

**Why this priority**: Dá uniformidade à funcionalidade e resgata páginas em que a busca hoje
é inoperante, mas pode ser entregue depois da história principal sem prejudicá-la.

**Independent Test**: Pode ser testado repetindo a busca por ID em cada página de listagem do
sistema, incluindo as entidades sem coluna pesquisável, verificando que o registro do ID
digitado é retornado em todas elas.

**Acceptance Scenarios**:

1. **Given** uma entidade cujo campo de busca hoje não filtra nada, **When** o Admin digita o
   ID de um registro dessa entidade, **Then** o registro aparece nos resultados.
2. **Given** qualquer página de listagem do sistema, **When** o Admin busca pelo ID de um
   registro existente, **Then** o comportamento é idêntico ao das demais páginas (mesma
   regra, mesma apresentação).

---

### Edge Cases

- O que acontece quando o termo digitado é numérico mas não corresponde a nenhum ID? A busca
  cai no comportamento textual normal (ex.: o número pode ocorrer dentro de um nome ou
  título), podendo resultar em lista vazia.
- O que acontece com termos mistos (letras e números, ex.: "abc123")? Não são tratados como
  ID; seguem apenas a busca textual.
- O que acontece com números com zeros à esquerda (ex.: "007")? São interpretados como o
  número 7 e localizam o registro de ID 7.
- O que acontece com números não inteiros ou negativos (ex.: "12.5", "-3")? Não são tratados
  como ID; seguem apenas a busca textual.
- O que acontece quando o ID existe em outra entidade, mas não na listagem atual? A busca é
  sempre escopada à entidade da página atual: nenhum registro de outra entidade é exibido.
- Como a paginação se comporta? A contagem total e a paginação continuam refletindo
  corretamente a lista de resultados, incluindo os matches por ID.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O sistema MUST permitir que a busca na listagem de qualquer entidade localize
  registros pelo seu identificador numérico único (ID), além da busca textual existente.
- **FR-002**: O sistema MUST tratar um termo de busca como ID somente quando ele for
  completamente numérico (apenas dígitos); termos com letras ou símbolos seguem apenas a
  busca textual.
- **FR-003**: O match por ID MUST ser por igualdade exata: o termo numérico corresponde ao
  registro cujo ID é aquele número (sem correspondência parcial, ex.: "12" não traz o ID 120).
- **FR-004**: Quando o termo numérico corresponder simultaneamente a um ID e a ocorrências
  textuais, o sistema MUST apresentar todos os matches em uma única lista de resultados.
- **FR-005**: A busca textual existente MUST permanecer inalterada (mesmos critérios de
  correspondência e mesmos resultados de hoje), inclusive o tratamento de caracteres
  especiais como texto literal.
- **FR-006**: A busca por ID MUST funcionar em todas as entidades do sistema, incluindo as
  que não possuem coluna textual pesquisável.
- **FR-007**: O Admin MUST continuar usando um único campo de busca, sem precisar escolher
  modo ou campo de filtro.
- **FR-008**: A contagem total e a paginação dos resultados MUST permanecerem corretas quando
  a busca incluir matches por ID.
- **FR-009**: A regra de busca (textual e por ID) MUST ser aplicada pelo núcleo do sistema, e
  a interface apenas coleta o termo digitado e exibe o resultado.

### Key Entities *(include if feature involves data)*

- **Registro (linha de entidade)**: qualquer item gerenciado pelo sistema (pesquisadores,
  alunos, grupos de pesquisa, produções científicas, iniciativas, etc.). Possui um
  identificador numérico único (ID) e uma coluna textual usada pela busca de hoje
  (nome/título/usuário) — esta última podendo não existir em algumas entidades.
- **Listagem de entidade**: página onde o Admin visualiza, pagina, ordena e busca os registros
  de uma entidade; é a superfície onde a busca por ID passa a atuar.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: O Admin localiza qualquer registro conhecido pelo seu ID em menos de 10 segundos,
  direto da página de listagem, sem navegar entre telas.
- **SC-002**: 100% dos cenários de busca textual existentes produzem exatamente os mesmos
  resultados de antes da mudança (zero regressão).
- **SC-003**: Ao buscar pelo ID exato de um registro existente, o registro aparece na primeira
  posição dos resultados em 100% dos casos.
- **SC-004**: A busca por ID apresenta comportamento idêntico em todas as páginas de listagem
  do sistema, incluindo as entidades sem coluna textual pesquisável.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
-->

- "Pesquisa por id" significa busca pelo identificador numérico único exibido na coluna ID das
  listagens, com correspondência exata — não busca por outro campo (e-mail, ano, campus etc.).
- O termo numérico continua também sendo comparado ao texto (comportamento aditivo), então
  buscar "7" traz o registro de ID 7 e qualquer registro com "7" no texto.
- Números com zeros à esquerda (ex.: "007") são interpretados como o número sem os zeros.
- Fora do escopo desta feature: busca por outras colunas, busca por múltiplos IDs de uma vez
  (lista separada por vírgulas), correspondência parcial de ID (ex.: "12" casando com 120) e
  busca global entre todas as entidades.
- A funcionalidade é um aprimoramento em relação ao sistema de referência original, que não
  possuía busca por ID; o comportamento novo deve ficar documentado como diferença esperada.

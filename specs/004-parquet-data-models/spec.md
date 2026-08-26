# Feature Specification: Mapeamento de Tabelas Acadêmicas

**Feature Branch**: `004-parquet-data-models`
**Created**: 2026-08-26
**Status**: Draft

## User Scenarios & Testing

### User Story 1 - Modelagem Completa (Priority: P1)
Como Administrador, quero ter os modelos de dados para todas as tabelas contidas no arquivo `exports_canonical.zip` disponíveis no banco de dados e no código, para que eu possa começar a manipular informações reais de iniciativas, artigos, e grupos de pesquisa.

**Acceptance Scenarios**:
1. **Given** a estrutura do banco gerada pelo SQLAlchemy, **When** o ORM mapear as classes, **Then** teremos modelos como `Article`, `Initiative`, `ResearchGroup`, `Award`, e `Advisorship`.

### User Story 2 - Visualização no Dashboard (Priority: P2)
Como Administrador, quero ter abas e links no menu lateral do painel para navegar por essas novas tabelas dinamicamente, da mesma forma como acesso as Universidades e Pesquisadores.

**Acceptance Scenarios**:
1. **Given** o menu lateral, **When** eu clicar em "Grupos de Pesquisa", **Then** verei a tabela de grupos de pesquisa carregada usando a estrutura reusável do `EntityPage.jsx`.

## Requirements

### Technical Constraints
- **CON-001**: Modelar no arquivo `backend/src/models/orm.py` as entidades prioritárias extraídas do schema do Parquet: `Article`, `Initiative`, `ResearchGroup`, `Advisorship`, e `Award` (podemos expandir iterativamente).
- **CON-002**: Utilizar Tipos nativos como `String`, `Integer` e `Boolean`. 
- **CON-003**: Incluir um item no menu do arquivo `Dashboard.astro` para cada entidade mapeada.
- **CON-004**: Criar uma página `.astro` dentro de `src/pages/dashboard/` para cada entidade mapeada.

### Functional Requirements
- **FR-001**: O administrador deve poder realizar operações CRUD em cada uma dessas abas.
- **FR-002**: Os IDs nas tabelas do ORM devem ser auto incrementados para compatibilidade com o formato dos exports originais.

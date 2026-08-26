# Feature Specification: Ingestão e UI do Restante das Tabelas Parquet

**Feature Branch**: `006-parquet-data-models-remaining`
**Created**: 2026-08-26
**Status**: Draft

## User Scenarios & Testing

### User Story 1 - Modelagem Completa (Priority: P1)
Como Administrador, quero ter os modelos de dados para todas as tabelas RESTANTES contidas no arquivo `exports_canonical.zip` disponíveis no banco de dados, para que o sistema consiga importar o arquivo ZIP em sua totalidade sem perder informações vitais.

**Acceptance Scenarios**:
1. **Given** a estrutura do banco, **When** eu inspecionar a interface, **Then** terei páginas para as entidades: Students, Campuses, Organizations, Fellowships, Proficiencies, Professional Activities, Knowledge Areas, e outras tabelas de domínio.

### User Story 2 - Visualização no Dashboard (Priority: P2)
Como Curador de Dados, quero acessar essas novas tabelas através de um novo agrupamento no menu lateral ou listadas sequencialmente, mantendo a paginação e os recursos de edição.

## Requirements

### Technical Constraints
- **CON-001**: Modelar no arquivo `backend/src/models/orm.py` as entidades faltantes mapeando as colunas como `String` ou `Integer` nullable para garantir resiliência aos dados "sujos".
- **CON-002**: Utilizar a mesma infraestrutura de `BaseRepository` para as novas tabelas (declarando-as no `postgres_adapter.py`).
- **CON-003**: Incluir as páginas genéricas Astro (`EntityPage`) para todas essas tabelas em `frontend/src/pages/dashboard/`.

### Functional Requirements
- **FR-001**: Adicionar os novos itens na barra lateral do painel (`Dashboard.astro`), podendo usar um submenu ou dropdown caso a lista fique longa demais.
- **FR-002**: O `import_zip` no `parquet_service.py` passará a importar mais arquivos automaticamente já que checa a existência das tabelas no ORM.

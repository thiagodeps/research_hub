# Feature Implementation Tasks: Restante dos Parquets

## Phase 1: Mapeamento no ORM (Backend)

**Purpose**: Estruturar as 9 tabelas faltantes.

- [ ] T001 Modificar `backend/src/models/orm.py` e criar as classes do SQLAlchemy para: `Student`, `Campus`, `Organization`, `Fellowship`, `Proficiency`, `ProfessionalActivity`, `KnowledgeArea`, `Language`, `ResearchProduction`.
- [ ] T002 Registrar as novas classes no dicionário `self.models` do `DatabasePostgresAdapter` mapeando seus respectivos nomes no plural (ex: `students`, `campuses`, `organizations`, `fellowships`, `proficiencies`, `professional_activities`, `knowledge_areas`, `languages`, `research_productions`).
- [ ] T003 Adicionar esses plurais e seus mapeamentos `.parquet` (caso diverjam) no `ParquetService._get_table_name_from_filename` em `parquet_service.py`.

---

## Phase 2: Interface Gráfica (Frontend)

**Purpose**: Criar as telas de gerenciamento no painel.

- [ ] T004 Criar as 9 páginas `.astro` em `frontend/src/pages/dashboard/` chamando o componente `<EntityPage>`:
      (`students.astro`, `campuses.astro`, `organizations.astro`, `fellowships.astro`, `proficiencies.astro`, `professional_activities.astro`, `knowledge_areas.astro`, `languages.astro`, `research_productions.astro`)
- [ ] T005 Adicionar estas rotas no menu lateral `frontend/src/layouts/Dashboard.astro` dentro de uma seção "Catálogos Adicionais".
- [ ] T006 Adicionar essas novas tabelas (child_types) no modal genérico `frontend/src/components/LinkModal.jsx` e `frontend/src/services/link_service.py` (se necessário mapear nomes de colunas, faremos dinâmico).

# Feature Implementation Tasks: Parquet Data Models

## Phase 1: Mapeamento no Banco de Dados (Backend)

**Purpose**: Estruturar as tabelas que virão dos arquivos `.parquet`.

- [x] T001 Modificar `backend/src/models/orm.py` e criar as classes do SQLAlchemy para: `Article`, `ResearchGroup`, `Initiative`, `Advisorship` e `Award`.
- [x] T002 Registrar as novas classes no dicionário `self.models` do `DatabasePostgresAdapter` em `backend/src/database/postgres_adapter.py`.
- [x] T003 Atualizar e rodar o script `backend/scripts/seed.py` para forçar a criação destas novas tabelas em disco.

---

## Phase 2: Interface Gráfica (Frontend)

**Purpose**: Criar as telas de gerenciamento no painel de administração.

- [x] T004 Atualizar `frontend/src/layouts/Dashboard.astro` para adicionar as rotas de navegação (ex: `/dashboard/articles`, `/dashboard/groups`) no menu lateral (sidebar).
- [x] T005 Criar página `frontend/src/pages/dashboard/articles.astro` contendo a `EntityPage` com campos: id, title, doi, year.
- [x] T006 Criar página `frontend/src/pages/dashboard/groups.astro` (ResearchGroups) com campos: id, name, description, short_name.
- [x] T007 Criar página `frontend/src/pages/dashboard/initiatives.astro` (Initiatives) com campos: id, name, status, start_date.
- [x] T008 Criar página `frontend/src/pages/dashboard/advisorships.astro` (Advisorships) com campos: id, name, status, start_date.
- [x] T009 Criar página `frontend/src/pages/dashboard/awards.astro` (Awards) com campos: id, title, year.

---

## Dependencies & Execution Order

- A Phase 1 deve ser executada antes, pois o frontend disparará chamadas genéricas `GET /articles`, e se o backend não tiver mapeado a tabela, dará erro.

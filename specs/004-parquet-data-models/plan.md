# Implementation Plan: Parquet Data Models

## 1. Context & Approach

Baseado nos arquivos extraídos de `exports_canonical.zip`, há diversas entidades (arquivos Parquet) representando o ecossistema de produção acadêmica do portal (Artigos, Premiações, Orientações, Grupos de Pesquisa, Iniciativas, etc).
Nesta etapa (etapa de estrutura), criaremos a base ORM (SQLAlchemy) e UI para as entidades vitais do arquivo. 
A importação dos dados em si (fazer o ingestion dos arquivos .parquet para o PostgreSQL) ficará para o futuro, mas a arquitetura e a manipulação manual via tela estarão prontas.

## 2. Component Architecture

### Component 1: Expansão do ORM (`backend/src/models/orm.py`)
Modelos a serem mapeados:
- `Article` (`articles`) -> title, doi, year, type, journal_conference.
- `ResearchGroup` (`research_groups`) -> name, description, short_name, cnpq_url.
- `Initiative` (`initiatives`) -> name, status, description, start_date.
- `Advisorship` (`advisorships`) -> name, status, description, start_date.
- `Award` (`awards`) -> title, year.

*(Como o Pydantic / CRUD genérico pega tudo, basta declararmos no `orm.py` e o adapter vai lidar com o resto)*

### Component 2: Expansão do Database Adapter (`backend/src/database/postgres_adapter.py`)
- Adicionar os novos modelos (Article, ResearchGroup, Initiative, Advisorship, Award) no dicionário `self.models`.

### Component 3: Frontend Menu e Roteamento (`frontend/src/layouts/Dashboard.astro` e `pages/dashboard/`)
- Inserir links na barra lateral do painel de administração.
- Criar os arquivos `.astro` (`articles.astro`, `groups.astro`, `initiatives.astro`, etc) usando o `<EntityPage client:load />`.

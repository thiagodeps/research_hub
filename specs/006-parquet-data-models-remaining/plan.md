# Implementation Plan: Restante das Tabelas Parquet

## 1. Context & Approach

A fim de curar todos os dados do datalake, adicionaremos as entidades remanescentes mais relevantes do `exports_canonical.zip`.
Devido ao alto volume de tabelas, faremos um mapeamento simplificado:
Tabelas a serem adicionadas:
1. `students` -> Students (Alunos)
2. `campuses` -> Campuses (Campus)
3. `organizations` -> Organizations (Organizações)
4. `fellowships` -> Fellowships (Bolsas)
5. `proficiencies` -> Proficiencies (Proficiências)
6. `professional_activities` -> ProfessionalActivities (Atividades Profissionais)
7. `knowledge_areas` -> KnowledgeAreas (Áreas de Conhecimento)
8. `languages` -> Languages (Idiomas)
9. `research_productions` -> ResearchProductions (Produções de Pesquisa)

Isso deixará de fora apenas metadados do pipeline (como logs, ingestion_runs, graphs).

## 2. Component Architecture

### Component 1: Expansão do ORM (`backend/src/models/orm.py`)
Novos modelos mapeados (todas as strings serão nullable, com `id` genérico, e os JSONs ficarão como String):
- `Student`, `Campus`, `Organization`, `Fellowship`, `Proficiency`, `ProfessionalActivity`, `KnowledgeArea`, `Language`, `ResearchProduction`

### Component 2: Adapter e Rotas (`backend/src/database/postgres_adapter.py`)
Registrar no dicionário de classes do `postgres_adapter`. O router genérico (`crud.py`) já cria as rotas automaticamente.
Também registrar no mapping de nomes no `ParquetService` se necessário, porém vamos padronizar o nome no dicionário.

### Component 3: Frontend Menu e Entidades (`frontend/src/layouts/Dashboard.astro` e `pages/dashboard/`)
- Criar os `.astro` wrappers para cada entidade:
  - `students.astro`, `campuses.astro`, `organizations.astro`, `fellowships.astro`, `proficiencies.astro`, `professional_activities.astro`, `knowledge_areas.astro`, `languages.astro`, `research_productions.astro`
- No menu lateral (`Dashboard.astro`), agrupar os links num scroll ou adicionar todos abaixo de uma divisória visual "Mais Dados".

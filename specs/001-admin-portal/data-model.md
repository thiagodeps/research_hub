# Data Model: Portal Admin e Gestão de Pesquisa

Este documento reflete a modelagem lógica derivada do pacote `research_domain`. O backend não deve redefinir as regras de negócio destas entidades, apenas fornecer os adaptadores (repositórios) para persistência (Postgres/Memory) e extensão com atributos operacionais como Soft Delete.

## Entidades de Domínio

### 1. Researcher
- **Descrição**: Pesquisador ou professor.
- **Atributos Principais**: `id`, `name`, `emails` (lista), `resume` (currículo).
- **Relações**: Pode possuir Articles, ResearchProductions, Advisorships.

### 2. University
- **Descrição**: Instituição de ensino.
- **Atributos Principais**: `id`, `name`, `abbreviation` (sigla).
- **Relações**: Possui vários Campi e ResearchGroups.

### 3. Campus
- **Descrição**: Unidade da Universidade.
- **Atributos Principais**: `id`, `name`.
- **Relações**: Vinculado a uma `University` (1:N). Pode possuir vários `ResearchGroup`s.

### 4. ResearchGroup
- **Descrição**: Grupo de Pesquisa.
- **Atributos Principais**: `id`, `name`, `abbreviation`.
- **Relações**: Vinculado a `University` e `Campus`.

### 5. Advisorship
- **Descrição**: Orientação acadêmica.
- **Atributos Principais**: `id`, `title`/`name`, `student_name`, `start_date`, `end_date`, `cancellation_date`.
- **Relações**: Vinculado ao `Researcher` (orientador).

### 6. Article
- **Descrição**: Publicação científica (artigo).
- **Atributos Principais**: `id`, `title`, `year`, `type`, `authors`, `doi`.
- **Relações**: Pode herdar/vincular a uma `Advisorship` ou `Researcher`.

### 7. Demais Entidades Auxiliares
- **KnowledgeArea**: Área de conhecimento.
- **Fellowship**: Bolsa de pesquisa.
- **AcademicEducation**: Formação acadêmica.
- **ResearchProduction**: Produção científica genérica.
- **EducationType** e **ProductionType**: Catálogos/enums auxiliares.

## Entidades Operacionais (Portal Admin)

### Admin User (Seed)
- **Descrição**: Credenciais de login para a plataforma.
- **Atributos**:
  - `id` (UUID)
  - `email` (String, UNIQUE)
  - `password_hash` (String)

## Regras de Estado & Ciclo de Vida

1. **Restrição de Deleção (Cascade Block)**: Nenhuma entidade que atue como "Pai" (ex: `University`) pode ser apagada se possuir entidades "Filhas" (ex: `Campus` ou `ResearchGroup`) ativas vinculadas.
2. **Fusão e Soft Delete**:
   - Ao executar a Fusão entre `Researcher A` e `Researcher B`, um `Researcher C` é gerado.
   - `Researcher A` e `Researcher B` recebem uma marcação de Soft Delete (`is_active = false` ou `merged_into = C_ID`).
   - Todos os relacionamentos (Articles, Advisorships) de A e B são migrados apontando para C.

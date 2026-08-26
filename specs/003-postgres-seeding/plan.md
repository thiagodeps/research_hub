# Implementation Plan: Banco de Dados PostgreSQL e Dados Falsos

## 1. Context & Approach

A infraestrutura atual utiliza um `DatabaseMemoryAdapter` super simplificado que perde todos os dados ao ser reiniciado. A abordagem para implementar o banco de dados oficial baseia-se em introduzir o SQLAlchemy como ORM suportando bancos relacionais padrão (PostgreSQL para produção/desenvolvimento ou SQLite para testes rápidos).

Além do mapeamento ORM, desenvolveremos um script autônomo `seed.py` na raiz do backend que utilizará a biblioteca `Faker` para gerar e popular centenas de registros na base.

## 2. Component Architecture

### Component 1: Engine and Session Config (`backend/src/database/session.py`)
- Configurar `create_engine` e `sessionmaker`.
- Resgatar URI de banco de dados (`DATABASE_URL`).
- Garantir pool de conexões otimizado.

### Component 2: Models (`backend/src/models/orm.py`)
- Mapeamento das tabelas utilizando `declarative_base`.
- Modelos a criar:
  - `Admin` (`admins`)
  - `University` (`universities`)
  - `Researcher` (`researchers`)

### Component 3: Database Adapter (`backend/src/database/postgres_adapter.py`)
- Implementar a interface utilizada por `CrudService` e `BaseRepository`.
- Funções: `get_all`, `get_by_id`, `create`, `update`, `delete`.
- Conversão fluída dos objetos mapeados pelo SQLAlchemy para dict/Pydantic que as rotas esperam.

### Component 4: Database Core Factory (`backend/src/database/core.py`)
- Alterar o factory `get_db()` para devolver o `DatabasePostgresAdapter` se `os.environ.get("STORAGE_TYPE") == "postgres"`.

### Component 5: Seeder Script (`backend/scripts/seed.py`)
- Setup da conexão de DB via SQLAlchemy.
- Deletar todos os dados das tabelas base e resetar chaves.
- Inserir Admin padrão (`admin@admin.com` -> `admin123`).
- Inserir N Universidades usando `Faker` (e.g., Faker().company()).
- Inserir M Pesquisadores usando Faker (e.g., nome, Lattes UUID, ORCID formatado).

## 3. Data Flow & Interfaces

1. `Database Core` resolve dependência baseado em `STORAGE_TYPE`.
2. Repositórios injetam o adaptador de Postgres.
3. Rotas recebem Pydantic na API -> Repository serializa no Model -> SQLAlchemy salva no DB.

## 4. Testing Strategy

- Subir as instâncias com SQLite `sqlite:///./test.db` local na própria memória durante a execução do pytest usando hooks do PyTest.
- Validar as operações do ORM simulando a classe PostgresAdapter.
- Garantir que o `seed.py` funcione rodando-o de ponta a ponta sem erros.

## 5. Security & Error Handling

- Senhas geradas pelo Seeder deverão ser criptografadas utilizando as funções do `src.core.security`.
- O adapter PostgreSQL deve lidar com falhas de rollback.

## 6. Development Phases

**Phase 1: Setup do SQLAlchemy e Modelos**
Criar classes bases ORM.
**Phase 2: Integração e Adapter do Postgres**
Ligar com a lógica do Repository.
**Phase 3: Seeder com Faker**
Desenvolvimento do script de povoamento.

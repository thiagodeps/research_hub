# Feature Implementation Tasks: Banco de Dados PostgreSQL e Dados Falsos

## Phase 1: Setup & Dependências (Shared Infrastructure)

**Purpose**: Instalar bibliotecas base para SQLAlchemy e migração.

- [x] T001 Adicionar dependências `sqlalchemy`, `psycopg2-binary` e `faker` no ambiente virtual (`backend/requirements.txt` se houver, ou instalação direta).

---

## Phase 2: Foundational (Models & ORM)

**Purpose**: Core infrastructure para comunicação com o banco de dados.

- [x] T002 Criar arquivo `backend/src/database/session.py` configurando `create_engine` (com suporte a env `DATABASE_URL` default para sqlite local) e `sessionmaker`.
- [x] T003 Criar arquivo `backend/src/models/orm.py` e definir a base declarativa com os modelos `Admin`, `University` e `Researcher`.
- [x] T004 Criar arquivo `backend/src/database/postgres_adapter.py` com a classe `DatabasePostgresAdapter` que implemente `get_all`, `get`, `save`, e `delete` interagindo com a Session do SQLAlchemy (convertendo ORM objects para dicts).
- [x] T005 Modificar `backend/src/database/core.py` (`get_db`) para retornar `DatabasePostgresAdapter` quando `os.environ["STORAGE_TYPE"] == "postgres"`.

**Checkpoint**: Models mapeados e lógica ORM encapsulada pronta para uso.

---

## Phase 3: Seeder & Testes Isolados (Priority: P1)

**Purpose**: Garantir que as tabelas sejam preenchidas automaticamente.

- [x] T006 Criar script de seeding `backend/scripts/seed.py` que importe o engine, recrie todas as tabelas (`Base.metadata.drop_all` e `create_all`).
- [x] T007 Implementar em `seed.py` a injeção do Admin fixo (`admin@admin.com` -> `admin123`).
- [x] T008 Implementar em `seed.py` o uso do `Faker` para popular 5 Universidades e 20 Pesquisadores de forma randômica.
- [x] T009 Refatorar testes de integração (`test_merge.py`, `test_link.py`) para garantir que os testes limpem e inicializem o banco se executados no modo `postgres`.

**Checkpoint**: At this point, running `python scripts/seed.py` should fully populate the SQLite/Postgres database.

---

## Phase 4: Integração Final (API)

**Purpose**: Substituir o adapter da API pelo novo e garantir que a UI o consuma perfeitamente.

- [x] T010 Subir o FastAPI exportando a variável de ambiente `STORAGE_TYPE=postgres` (e garantir suporte fallback `DATABASE_URL=sqlite:///./test.db`).
- [x] T011 Navegar pelo frontend nas rotas `/dashboard/universities` e `/dashboard/researchers` validando se a massa de dados do seeding é renderizada e as ações CRUD funcionam via adapter novo.

---

## Dependencies & Execution Order

- As Fases 1 e 2 são sequenciais e obrigatórias para termos o banco funcional.
- A Fase 3 depende dos modelos ORM da Fase 2.
- A Fase 4 só valida o resultado de ponta-a-ponta, confirmando a estabilidade da Feature.

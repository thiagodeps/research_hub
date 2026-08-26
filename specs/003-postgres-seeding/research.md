# Technical Research: ORM & Seeding

## 1. Technologies
- **SQLAlchemy 2.0+**: Abordagem moderna para Models e Sessions.
- **Psycopg2 / asyncpg**: Driver para PostgreSQL (Psycopg2 é sincrono, o que bate com as views atuais do FastAPI que não são `async`).
- **Faker**: Lib python ideal para seed.

## 2. Alternatives Considered
- *SQLModel*: É muito bom por integrar Pydantic com SQLAlchemy nativamente, mas para simplificar refatoração optaremos pelo SQLAlchemy puro.
- *Alembic*: Necessário no futuro para migrações contínuas, mas por hora o `.metadata.create_all` basta.

## 3. Findings
O backend atual possui rotas **síncronas** (`def update(...)` invés de `async def`). Portanto, o uso da engine SQLAlchemy `create_engine` sem suporte `async` é o caminho de menor resistência, ou precisaremos refatorar todo o arquivo de rotas para `async`. Focaremos na adoção sincrona.

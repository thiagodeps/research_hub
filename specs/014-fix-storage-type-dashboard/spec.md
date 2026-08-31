# Feature Specification: Fix Empty Dashboard After ZIP Import (014)

## 1. Description
- **Bug:** After importing a ZIP successfully, the dashboard showed no data. Worse, opening any paginated entity list (e.g. `/dashboard/researchers`) crashed the API with `TypeError: DatabaseMemoryAdapter.get_all() got an unexpected keyword argument 'search'`.
- **Root cause:** The backend has two disconnected storage layers. `ParquetService.import_zip` always writes imported rows directly into the SQL database (SQLite/Postgres) via SQLAlchemy. But `get_db()` defaults to `STORAGE_TYPE=memory`, which serves all CRUD/listing routes from an in-process dict that the importer never touches. On top of that, `DatabaseMemoryAdapter.get_all()` only accepted a `table` argument, while `BaseRepository.get_all()` always calls it with `limit`, `offset`, `search`, `sort`, and `order`, and expects an `(items, total)` tuple back — the same interface `DatabasePostgresAdapter.get_all()` already implements.

## 2. Requirements
- `DatabaseMemoryAdapter.get_all()` in `backend/src/database/core.py` must accept the same `limit/offset/search/sort/order` parameters as `DatabasePostgresAdapter.get_all()` and return an `(items, total)` tuple, so listing routes don't crash when the app runs without `STORAGE_TYPE=postgres` set.
- `make run` (`Makefile`) must start the backend with `STORAGE_TYPE=postgres` so the CRUD/listing routes read from the same SQL database the ZIP importer writes to.
- `README.md` must document `STORAGE_TYPE=postgres` as required (not optional) for local development, explaining that it selects the SQL-backed adapter rather than literally requiring a PostgreSQL server.

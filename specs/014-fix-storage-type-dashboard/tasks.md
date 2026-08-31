# Tasks: Fix Empty Dashboard After ZIP Import (014)

- [x] T001 Give `DatabaseMemoryAdapter.get_all()` in `backend/src/database/core.py` the same signature as `DatabasePostgresAdapter.get_all()` (`limit`, `offset`, `search`, `sort`, `order`) and have it return `(items, total)`.
- [x] T002 Update the `run-backend` target in `Makefile` to export `STORAGE_TYPE=postgres` before starting `uvicorn`.
- [x] T003 Update `README.md` to mark `STORAGE_TYPE=postgres` as required and explain why (avoids the disconnected in-memory adapter).

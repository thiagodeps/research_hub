# Tasks: Table Sorting (010)

-[x] T001 Update `postgres_adapter.py` `get_all` to support `sort` and `order`, using `func.length()` for heuristic size sorting on string arrays, or just column sorting.
-[x] T002 Plumb `sort` and `order` through `repositories.py` and `crud_service.py`.
-[x] T003 Update `crud.py` `get_all` endpoint to accept `sort` and `order`.
-[x] T004 Add `sortCol`, `sortOrder` states to `EntityPage.jsx` and append them to API requests.
-[x] T005 Update `EntityTable.jsx` to render clickable column headers with visual sorting indicators.

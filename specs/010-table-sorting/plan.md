# Implementation Plan: Table Sorting (010)

## 1. Backend Changes
- In `backend/src/api/crud.py`, add `sort: Optional[str] = None, order: Optional[str] = "asc"` to the `get_all` route.
- In `backend/src/services/crud_service.py`, pass `sort` and `order` down.
- In `backend/src/database/repositories.py`, pass `sort` and `order` down to adapter.
- In `backend/src/database/postgres_adapter.py`, inside `get_all()`:
  - If `sort` is provided, verify `hasattr(model, sort)`.
  - To support "number of items" for JSON, we can use SQLAlchemy's `func.length(getattr(model, sort))`. Since all our lists are stringified JSON `[{...}, {...}]`, the string length correlates extremely well with the number of items. 
  - Apply `order_by(sort_attr.asc())` or `desc()`.

## 2. Frontend Changes
- In `EntityPage.jsx`:
  - Add state `[sortCol, setSortCol] = useState(null)` and `[sortOrder, setSortOrder] = useState('asc')`.
  - Pass `sortCol`, `sortOrder`, and a `onSort={handleSort}` callback to `EntityTable`.
  - Update `loadData()` to append `&sort=${sortCol}&order=${sortOrder}` if `sortCol` is truthy.
- In `EntityTable.jsx`:
  - Update `<thead>` to loop through `columns`.
  - Make each `<th>` a clickable flex container.
  - If `sortCol === col`, render an arrow (↑ for asc, ↓ for desc).
  - When clicked, call `onSort(col)`.


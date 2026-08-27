# Implementation Plan: Search & Deep Linking (008)

## 1. Backend Search Implementation
- In `backend/src/api/crud.py`, update `@router.get("/")` to accept `search: Optional[str] = None`.
- Pass `search` to `service.get_all(..., search=search)`.
- In `postgres_adapter.py` (`get_all`), if `search` is provided:
  - Find the column to search on (check if `name`, `title`, or `username` exists in the model).
  - Apply `query = query.filter(SearchColumn.ilike(f"%{search}%"))`.

## 2. Frontend Search Implementation
- In `EntityPage.jsx`, add `[search, setSearch] = useState("")`.
- Add an `<input type="text">` above the table.
- When `search` changes, reset `page` to `0` and call `loadData()`.
- Update the fetch URL to include `&search=${search}`.

## 3. Frontend Deep Linking
- In `EntityForm.jsx`, where `type === 'json_readonly'`, we currently map over `val` and render `<li>{JSON.stringify(item)}</li>`.
- We will improve this:
  - Check if `item` has an `id` and a `name` or `title`.
  - Determine the path base based on the `field.name` (e.g. `groups` for `research_groups`).
  - Render an `<a>` tag pointing to `/dashboard/${pathBase}?openId=${item.id}`.
- In `EntityPage.jsx`, inside `useEffect` (or on mount), check `new URLSearchParams(window.location.search).get('openId')`.
  - If present, fetch that specific item: `await apiFetch('/{entity}/{id}')`.
  - Set it as `editingItem` to pop open the modal!
  - Remove `openId` from URL using `history.replaceState` so it doesn't reopen if they refresh.

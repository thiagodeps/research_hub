# Tasks: Search & Deep Linking (008)

-[x] T001 Update `postgres_adapter.py` `get_all` method to support `search` filtering by `name`, `title`, or `username`.
-[x] T002 Update `crud.py` router `get_all` endpoint to accept the `search` query string.
-[x] T003 Update `EntityPage.jsx` to include a search input and pass the `search` param in `loadData`.
-[x] T004 Update `EntityPage.jsx` to read `openId` from URL on mount, fetch the entity, and open the edit modal.
-[x] T005 Update `EntityForm.jsx` to render `json_readonly` items as clickable `<a href>` links mapped to their respective pages.

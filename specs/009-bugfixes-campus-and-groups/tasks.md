# Tasks: Bugfixes (009)

- [x] T001 Remove explicit `groups` mapping from `parquet_service.py` to allow `research_groups` to sync with ORM metadata.
- [x] T002 Update `EntityTable.jsx` to render `parsed.name` for plain JSON objects instead of falling back to string truncation.
- [x] T003 Update `EntityForm.jsx` to wrap plain parsed JSON objects in an Array so `.map()` works natively in the form links UI.
- [x] T004 Run a background script to manually hydrate the `research_groups` table with the missing data from `exports_canonical.zip` without dropping the database.

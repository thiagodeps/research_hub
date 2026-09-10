# Tasks: Bugfixes — Record Creation, Link Errors, Tab Numbering (025)

- [x] T001 Fix `EntityPage.handleSave` in `frontend/src/components/EntityPage.jsx` to route create vs. update based on `editingItem`'s own id, and PUT to `editingItem.id` rather than `payload.id`.
- [x] T002 Make the `id` field in `frontend/src/components/EntityForm.jsx` read-only/non-editable when creating a new record (no `initialData.id`), so ids are never hand-assigned against the ZIP numbering.
- [x] T003 Surface JSON parse failures in `EntityForm.jsx`'s `json_readonly` renderer instead of silently swallowing them.
- [x] T004 Add sequential numbering to the sidebar table links in `frontend/src/layouts/Dashboard.astro`.

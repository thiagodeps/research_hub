# Feature Specification: Bugfixes — Record Creation, Link Errors, Tab Numbering (025)

## 1. Description
- **Bug 1 (create routing):** `EntityPage.handleSave` decides POST-vs-PUT by comparing
  `editingItem?.id === payload.id`. Every entity form renders an editable `id`
  input, so on a brand-new record (`editingItem = {}`) the user normally leaves
  it blank, and `payload.id` is `undefined` too — `undefined === undefined` is
  `true`, so the save is routed as a `PUT /{entity}/undefined`. That path
  matches no route in `frontend/src/services/api.js` (the `id` segment must be
  `\d+`), so the request is rejected before it ever reaches the Rust backend.
  The only way a record is ever created today is if the user manually types a
  numeric id that happens not to collide — which requires guessing the next id
  used by the imported ZIP, the exact friction the user reported. The Rust
  side (`crud::create`, `src-tauri/src/crud.rs`) already omits an empty/absent
  `id` and lets SQLite assign the next rowid after the highest id currently in
  the table (i.e. it already continues the ZIP's own numbering) — the backend
  needs no change.
- **Bug 2 (silent link-error swallow):** `EntityForm.jsx`'s `json_readonly`
  renderer (used for relationship columns such as `campus`, `research_groups`)
  wraps `JSON.parse` in `try { … } catch (e) {}` — an empty catch. When the
  column holds corrupted or unparseable content, the field silently renders
  "Nenhum vínculo." exactly as it would for a record with no links at all,
  giving the user no indication anything is wrong. `EntityTable.jsx`'s own
  `formatCellValue` already reports `'Erro'` on the same failure, so the table
  and the detail form disagree, and the form — the more detailed view — is the
  one that stays silent.
- **Bug 3 (UX):** the dashboard's sidebar table tabs (`Dashboard.astro`) are an
  unnumbered list, making it harder to navigate/refer to a specific table
  section.

## 2. Requirements
- `EntityPage.handleSave` must decide create-vs-update from whether
  `editingItem` represents an existing record (has a non-null `id`), not from
  comparing it against the submitted payload's `id`. Editing an existing
  record must keep using its real id for the `PUT` URL, never the form's
  (possibly edited) `id` field.
- The `id` input in `EntityForm` must not be an editable field when creating a
  new record — creation always lets the backend assign the next id, which
  already follows the ZIP-imported numbering. It may still display the id
  (read-only) when editing an existing record.
- `EntityForm`'s `json_readonly` renderer must surface a parse failure to the
  user (visible error text in that field) instead of silently rendering an
  empty/no-links state.
- The dashboard sidebar (`Dashboard.astro`) must show a sequential number
  before each table's nav entry (both the primary and "Catálogos Adicionais"
  groups), so a table can be referred to by its position.

# Feature Specification: Search & Deep Linking (008)

## 1. Description
Users need the ability to search for specific records in the large canonical tables rather than just paging through them. 
Additionally, users need to seamlessly navigate between related entities. When viewing a large entity (e.g., Student), they should be able to click on one of its relations (e.g., a specific Article or Initiative) and be redirected to that entity's detailed view.

## 2. Requirements

### 2.1 Search Bar
- **Backend:** `CrudRouter` and `DatabasePostgresAdapter` must accept an optional `search` query parameter. It should perform an ILIKE (case-insensitive) search across primary descriptive columns (like `name` or `title`).
- **Frontend:** Add a search input bar above the `EntityTable` in `EntityPage.jsx`. It should debounce user input and trigger an API fetch with the `search` parameter, resetting pagination to page 0.

### 2.2 Deep Linking
- **Backend:** Ensure the GET `/{id}` endpoint works for all entities.
- **Frontend:** 
  - In `EntityForm.jsx`, format `json_readonly` array items as HTML links `<a>` instead of plain text `<li>`.
  - The link URL should follow a mapping pattern (e.g., `research_groups` -> `/dashboard/groups?openId=123`).
  - In `EntityPage.jsx`, if the URL contains `?openId=XYZ` on load, the page should automatically fetch that specific entity and open the "Visualizar/Editar" modal for it.

## 3. Acceptance Criteria
- [ ] Typing in the search bar filters the table results dynamically.
- [ ] Relationship JSON items in the details modal render as clickable blue links.
- [ ] Clicking a relation link redirects to the target entity's dashboard page and automatically opens its details modal.

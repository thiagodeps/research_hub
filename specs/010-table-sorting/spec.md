# Feature Specification: Table Sorting (010)

## 1. Description
Users need the ability to sort table data by clicking on the column headers in the dashboard. They should be able to sort alphabetically (for text like names) or by size/quantity (for JSON relationship columns like `initiatives`), toggling between Ascending and Descending order.

## 2. Requirements

### 2.1 Backend
- **crud.py:** The `get_all` endpoint must accept two new optional query parameters: `sort` (the column name) and `order` (`asc` or `desc`).
- **postgres_adapter.py:** The `get_all` method must apply an `ORDER BY` clause to the SQLAlchemy query if `sort` is provided. If the column represents a JSON array (or just any column generically), it should sort properly. To handle "number of initiatives", we can apply a `func.length()` sort on string columns if a special flag is passed, or just sort generically.

### 2.2 Frontend
- **EntityPage.jsx:** Manage `sortCol` and `sortOrder` states and append them to the `apiFetch` URL.
- **EntityTable.jsx:** Make the `<th>` headers clickable. When a header is clicked:
  - If it's already the `sortCol`, toggle `sortOrder` between `asc` and `desc`.
  - If it's a new column, set it to `sortCol` and `asc`.
  - Display a small arrow (↑/↓) next to the active column to indicate sort direction.

## 3. Acceptance Criteria
- [ ] Clicking on "name" sorts the rows alphabetically.
- [ ] Clicking again reverses the order.
- [ ] Clicking on a relationship column like "initiatives" sorts the rows (ideally by the amount of data/length).
- [ ] The API processes `sort` and `order` parameters safely, ignoring invalid column names.

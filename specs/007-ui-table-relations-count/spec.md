# Feature Specification: UI Table Relations Count (007)

## 1. Description
The main entities tables (EntityTable.jsx) currently display raw stringified JSON arrays for relationship columns (such as `initiatives`, `research_groups`, `students`). This causes the tables to be visually cluttered, slow to render, and ugly when the arrays are empty (displaying `[]`). 

We need to format these specific relationship columns in the React Table component to display only the count of items (e.g., "3 vínculos", "0 vínculos" or just the number), while maintaining the full raw JSON view when the user clicks "Visualizar" to open the detailed EntityPage/EntityForm.

## 2. Requirements
- The `EntityTable` component must detect if a column represents a relationship (or a JSON array).
- If it is a relationship array, parse the JSON and display the length of the array in the table cell.
- If it's an empty array or null, display a fallback like "0" or "-" to keep it clean.
- The modal/page view (`EntityForm`) must continue to display the full interactive or read-only JSON representation for editing/viewing.

## 3. Acceptance Criteria
- [ ] Columns like `initiatives`, `students`, `knowledge_areas` on the dashboard tables show the count of items instead of `[{...}, {...}]`.
- [ ] Clicking to view an entity still shows the full JSON list in the detail form.
- [ ] The table layout looks cleaner and does not stretch or break due to massive JSON strings.


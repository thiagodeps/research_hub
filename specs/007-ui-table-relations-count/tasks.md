# Tasks: UI Table Relations Count (007)

-[x] T001 Update `EntityPage.jsx` to pass the `fields` array down to `EntityTable` as a prop.
-[x] T002 Update `EntityTable.jsx` to accept the `fields` prop.
-[x] T003 In `EntityTable.jsx`, create a helper function `formatCellValue(value, col, fields)` that checks if the field type is `json_readonly`.
-[x] T004 In `formatCellValue`, if `json_readonly`, safely `JSON.parse` the string and return the length (e.g., `3 itens`). If empty/null/error, return `0 itens`.
-[x] T005 Update the JSX in `EntityTable` to use `formatCellValue` instead of rendering `item[col]` directly.

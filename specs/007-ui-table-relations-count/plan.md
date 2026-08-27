# Implementation Plan: UI Table Relations Count (007)

## 1. Context & Architecture
We have an Astro/React hybrid frontend. The dashboard lists entities using `frontend/src/components/EntityTable.jsx`. The detailed view/edit uses `frontend/src/components/EntityForm.jsx` or similar, wrapped by `EntityPage.jsx`.

Currently, `EntityTable.jsx` receives `data` from the API, and iterates over the `columns` prop to render `item[col]`. For JSON strings, it literally renders the raw string.

## 2. Technical Approach
1. In `EntityTable.jsx`, we need to parse or detect if a value is a JSON array string (e.g. starts with `[` and ends with `]`).
2. Alternatively, we can pass down the `fields` schema (which we already pass to `EntityPage.jsx`) into `EntityTable.jsx` so the table knows the `type` of the column.
3. Currently `EntityPage.jsx` does: `<EntityTable data={data} columns={columns} onEdit={...} />`. We can also pass `fields={fields}` down to `EntityTable`.
4. In `EntityTable.jsx`, find the field definition for the column. If `field.type === 'json_readonly'`, we format the output as `X vínculos`.
5. Implement the formatting logic inside `EntityTable`: 
   - Parse the JSON string.
   - Return `${parsed.length} vínculos` or `0 vínculos`.

## 3. Risks & Edge Cases
- Invalid JSON strings could cause `JSON.parse` to throw errors and crash the frontend. We must wrap the parsing in a `try/catch`.
- Null or empty strings should safely default to `0`.

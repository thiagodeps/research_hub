# Implementation Plan: Inline JSON Rendering (013)

## 1. Frontend Changes
- Edit `frontend/src/components/EntityForm.jsx`.
- Find the `items.map()` block inside the `json_readonly` field handler.
- Add an `if (item.id)` conditional.
- If true, return the `<a href...>` link.
- If false, return a `<li className="text-slate-600">` containing an `Object.entries(item).map()` loop to print out the keys and values cleanly.

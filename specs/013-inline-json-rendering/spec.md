# Feature Specification: Inline JSON Rendering for Metadata (013)

## 1. Description
The user reported an Astro error when clicking on `academic_education` links inside a Researcher's entity form. Analysis of the database and source ZIP reveals that `academic_education` (and `role_evidence`) are inline JSON metadata fields, not external table relationships. They lack an `id` field and do not have a corresponding dashboard route. The system was erroneously trying to render them as deep links (e.g. `ID: undefined`).

## 2. Requirements
- Modify `EntityForm.jsx` to dynamically inspect JSON objects in `json_readonly` fields.
- If an object contains an `id` property, it is a relationship and should render as a deep link (existing behavior).
- If an object does NOT contain an `id` property, it is inline metadata and should render as a readable key-value list (e.g. formatting `degree: Doutorado`, `institution: UFBA`, etc.).

## 3. Acceptance Criteria
- [ ] Clicking on a Researcher correctly parses and displays `academic_education` as bulleted lists of degrees/institutions without broken `undefined` links.
- [ ] Columns that actually have IDs (like `initiatives`) continue to render as functional deep links.

# Feature Specification: Remove Universities Tab (012)

## 1. Description
The user has noted that the `Universities` tab is redundant because the platform already has a `Campuses` tab which contains the relevant institutional data. Furthermore, the `universities` table in the database contains 0 records from the canonical parquet exports. Therefore, the tab and its associated frontend page should be removed to declutter the UI.

## 2. Requirements
- Remove the "Universities" link from the sidebar navigation in `Dashboard.astro`.
- Delete `frontend/src/pages/dashboard/universities.astro`.

## 3. Acceptance Criteria
- [ ] The "Universities" link is no longer visible in the Dashboard sidebar.
- [ ] The `universities.astro` file is deleted.

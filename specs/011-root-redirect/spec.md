# Feature Specification: Root URL Redirect (011)

## 1. Description
Currently, visiting the root URL (`http://localhost:4321/`) results in a 404 error because `frontend/src/pages/index.astro` does not exist. The user has to manually type `/login` to access the application. We need to create a landing mechanism at the root path.

## 2. Requirements
- Create `frontend/src/pages/index.astro`.
- When a user navigates to the root path `/`, the application should run a lightweight script to check if the user is authenticated (e.g., checking `localStorage` for a token).
- If authenticated, redirect the user immediately to `/dashboard`.
- If not authenticated, redirect the user immediately to `/login`.

## 3. Acceptance Criteria
- [ ] Navigating to `http://localhost:4321/` automatically resolves to `/login` for logged-out users.
- [ ] Navigating to `http://localhost:4321/` automatically resolves to `/dashboard` for logged-in users.

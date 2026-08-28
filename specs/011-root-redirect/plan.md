# Implementation Plan: Root URL Redirect (011)

## 1. Frontend Changes
- Create `frontend/src/pages/index.astro`.
- Insert a basic HTML shell.
- Add an inline `<script>` that reads `localStorage.getItem('token')`.
- Use `window.location.replace('/dashboard')` if token exists, else `window.location.replace('/login')`.
- Add a fallback loading spinner or text while the JavaScript executes (which should be nearly instantaneous).

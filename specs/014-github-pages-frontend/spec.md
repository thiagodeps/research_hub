# Feature Specification: GitHub Pages Deployment Integration (014)

## 1. Description
The user requested that the program run on GitHub Pages. GitHub Pages is a static hosting service, meaning it can only host the Astro Frontend (HTML/JS/CSS). It cannot host the Python/FastAPI Backend or the PostgreSQL/SQLite database.
Therefore, this feature adapts the Astro Frontend to be deployable to GitHub Pages, while preserving the API base URL architecture so the backend can be hosted on a separate cloud provider (e.g., Render/Railway) and linked via environment variables.

## 2. Requirements
- **astro.config.mjs:** Must be updated with `site` (https://thiagodeps.github.io) and `base` (/research_hub/) properties to ensure static assets (JS/CSS) resolve correctly on GitHub Pages.
- **GitHub Actions:** A deploy workflow `.github/workflows/deploy.yml` must be created to automatically build and publish the Astro site to the `gh-pages` branch on every push to `main`.
- **API URL Configuration:** Ensure `PUBLIC_API_URL` is clearly configurable for when the backend is deployed.

## 3. Acceptance Criteria
- [ ] `astro.config.mjs` contains `site` and `base` matching the repo URL.
- [ ] A functional GitHub Actions workflow exists for Astro deployment.
- [ ] All codebase changes are pushed to GitHub.

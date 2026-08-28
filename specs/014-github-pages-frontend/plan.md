# Implementation Plan: GitHub Pages Deployment Integration (014)

## 1. Config Updates
- Modify `frontend/astro.config.mjs` to include `site: 'https://thiagodeps.github.io'` and `base: '/research_hub'`.
- Modify `index.astro` script to account for the `base` path (e.g., redirecting to `import.meta.env.BASE_URL + '/login'`). Wait, Astro handles `base` implicitly for `<a href>` but for `window.location.replace` we need to use the base path. I'll hardcode `/research_hub` or use a standard approach. Let's make it robust by letting Astro inject the base URL.

## 2. GitHub Actions
- Create `.github/workflows/deploy.yml`.
- Add standard Astro GitHub pages deployment steps (checkout, setup-node, install dependencies, build, upload-pages-artifact, deploy-pages).

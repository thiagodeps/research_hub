# Release Notes (Patch Notes) 📜

Complete history of versions, enhancements, bug fixes, and new features in **Research Hub**.

---

## v1.3.4 (Current — GitHub Sync & Nested Archives)
*Source Branch: `034-github-sync`*

### GitHub Synchronization (SEP-034)
- **Remote Export Download**: Direct import of canonical packages from GitHub URLs (public or private repositories with a configured Personal Access Token). Supports direct raw file URLs (`raw.githubusercontent.com`) and GitHub release assets.
- **Atomic Export Upload**: Send the Horizon canonical package directly to GitHub branches via the Git Data API (creates atomic commits directly without needing to clone the repository).
- **Secure Token Management**: Local storage of GitHub Personal Access Token (PAT) in a dedicated file with strict POSIX `0600` permissions. Interactive modal with token validation, scope checking, token masking, and zero-trace removal.
- **Privacy Protection Policy (PII)**: Strict block on sending SRC project data to remote repositories, ensuring that participant CPFs and emails are never exposed to cloud repositories.
- **Operational Limits and Safety**: Supports package transfers of up to 100 MB with pre-validation of file size and automated safety snapshots prior to any database replacement.
- **Leak Audit**: 100% of error flows audited — tokens and personal data never leak into logs, error messages, or telemetry.

### Nested Canonical Archives
- Seamless ingestion of nested ZIP files and directory structures inside the root canonical package.

---

## v1.3.3 (SRC Curation & Multi-Project Selector)
*Source Branch: `033-src-data-tab`*

### Multi-Project Selector
- New post-login home page at `/projects` allowing seamless switching between **Horizon** (15 canonical tables) and **SRC** (Extension & Teaching).
- Strict isolation of database schemas, tables, forms, and safety snapshots between both projects.

### Extension and Teaching Domain (SRC)
- Dedicated `src_acoes` database table and relational migration `002_src_tables.sql`.
- Import and export of the consolidated SRC JSON file (`src_consolidado.json`), preserving object key order and structure.
- Specialized visual editor for nested **Participations** within actions (Target Audience and Execution Team).
- Textual linking to umbrella programs and deletion protection against removing programs with dependent child actions.

---

## v1.3.0 (New JSON-Only Canonical Export & Campus Tab Fix)
*Source Branch: `032-canonical-export-json`*

### JSON-Only Package Ingestion (SEP-032)
- Comprehensive adaptation of import and export workflows to the new DataLake standard (`{table}_canonical.json` files at the root of the ZIP, discarding Parquet files).
- Backward compatibility preserved for legacy archives containing Parquet files.
- JSON type inference and fidelity module for flawless round-trip conversion (preserving original booleans, integers, floats, nulls, and relation lists).

### Campus Tab Fix
- Elimination of the duplicate phantom nested "Campus (Links)" field inherited from upstream DataLake bug.

---

## v1.2.0 (Curation Workflow Stability & Deduplication)
*Source Branches: `030-search-by-id` and `031-bugfixes-curation-workflow`*

### Search by Identifier (SEP-030)
- Search input with direct numeric ID search support across all Horizon entities.

### Critical Curation Workflow Fixes (SEP-031)
- **B-01**: Fixed stale state in edit form when switching between different records.
- **B-02**: Automatic pagination repositioning to the last valid page upon deleting the last record on a page.
- **B-03**: Automatic clearing of merge selections after deletions, searches, or page changes.
- **B-04**: Enhanced merge with support for resolving by "title" (articles, productions, awards) in addition to "name".
- **B-04b**: Transactional union of relationship lists during record merges, preventing relationship loss.
- **B-05**: Elimination of race conditions on debounced asynchronous search queries.
- **B-06**: Canonical lowercase username normalization for active sessions.
- **B-07**: Session authentication verification prior to opening system file dialogs.
- **B-08**: Atomic ZIP package validation upon import, rejecting packages without canonical tables without clearing existing data.

---

## v1.1.0 – v1.1.3 (Authentication, User Registration & Accessibility)
*Source Branch: `029-user-registration`*

### User Registration (SEP-029)
- New `/register` page for creating administrator accounts.
- Secure password hashing using `bcrypt` within the Rust core.

### Accessibility and Quality
- Form accessibility fixes (`htmlFor` association of labels with input elements).
- Comprehensive unit test suite with JSDOM environment and form cleanup.

---

## v1.0.9 (Direct Page Navigation)
*Source Branch: `028-table-page-nav`*

### Page Navigation
- Interactive numeric page input in the `EntityPage` component for direct jumping to any table page with automatic limit clamping.

---

## v1.0.0 – v1.0.7 (Rust + Tauri 2.0 Desktop Migration & Python Decommission)
*Source Branches: `rust-tauri-migration` (SEP-015 to SEP-026)*

### Complete Architectural Rewrite
- Complete decommission of legacy Python FastAPI backend and PostgreSQL database.
- Modern desktop architecture with Rust + Tauri 2.0 and embedded single-file SQLite database.
- Eliminated background servers, open network ports, and multi-process overhead.
- Astro + React + Tailwind frontend statically compiled and embedded into the final binary.
- Multiplatform automated CI/CD release pipelines on GitHub Actions (Linux `.deb`, `.rpm`, `.AppImage`; Windows `.exe`, `.msi`; macOS `.dmg`, `.app`).
- Containerized build support via Docker with Ubuntu 24.04 glibc base.

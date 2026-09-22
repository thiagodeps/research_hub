# Research: Account Creation and Password Hashing

## Context

ResearchHub is a local desktop application built on Tauri 2.0 with a Rust core and an Astro/React frontend embedded in the binary. Currently, authentication is restricted to a single pre-seeded administrator credential (`admin@admin.com` / `admin123`). This feature adds an account creation interface and a secure password hashing mechanism allowing users to register new accounts with email and password, and subsequently authenticate into the workspace from the initial login screen.

## Decisions

### Decision 1: Password Hashing Algorithm and Salt Management

- **Decision**: Utilize the `bcrypt` crate with cost factor 12 (matching `bcrypt::DEFAULT_COST` already in use in `db.rs` and `auth.rs`).
- **Rationale**:
  - `bcrypt` is already declared in `src-tauri/Cargo.toml` and verified in `src-tauri/src/auth.rs`.
  - Bcrypt automatically generates cryptographically random 128-bit salts and bundles the salt, work factor, and hash in standard modular crypt format (`$2b$12$...`).
  - Work factor 12 provides robust defense against offline brute-force attacks while completing in ~50ms on desktop CPUs, imperceptible to the user.
  - Plaintext passwords will never be persisted or logged at any point in memory or storage.
- **Alternatives considered**:
  - `Argon2id`: State-of-the-art memory-hard algorithm, but requires adding a new dependency (`argon2`) and adjusting the existing verification code in `auth.rs`. Unnecessary complexity when bcrypt is already vetted in the codebase.
  - `PBKDF2`: Standardized but lacks memory-hardness; no advantage over bcrypt.
  - `SHA-256` / `MD5`: Fast hashing algorithms vulnerable to modern GPU rainbow tables; strictly rejected.

### Decision 2: Account Persistence and Storage Model

- **Decision**: Persist registered accounts directly in the SQLite `admins` table (`id INTEGER PRIMARY KEY, username TEXT UNIQUE, hashed_password TEXT`), storing the normalized email (lowercased and trimmed) in `username` and the salted bcrypt hash in `hashed_password`.
- **Rationale**:
  - Conforms to Constitution Principle IV: single Admin profile for all local data curation, with no multi-role approval tiers.
  - The existing `verify_credentials(&conn, username, password)` in `src-tauri/src/auth.rs` already validates against `admins WHERE username = ?1`.
  - Reusing `admins` guarantees that newly created accounts can log in immediately through the existing `login` command without requiring schema migrations or complex multi-table joins.
  - SQLite enforces unique constraint on `username`, providing database-level protection against race conditions and duplicate registrations.
- **Alternatives considered**:
  - Creating a separate `users` table: Adds redundant tables and requires branching the login verification queries or creating complex inheritance schemes.
  - Modifying `admins` table structure (e.g. adding `role` or `name`): Unnecessary for the current scope and adds migration risks.

### Decision 3: IPC Boundary and Business Logic Placement

- **Decision**: Implement a dedicated Tauri command `register(state: State<'_, AppState>, email: String, password: String, password_confirm: String) -> Result<(), AppError>` in `src-tauri/src/commands.rs`. Wire this command through `frontend/src/services/api.js` via `apiFetch('/auth/register', { method: 'POST', body: ... })`.
- **Rationale**:
  - Conforms strictly to Constitution Principle VI: all business logic (email validation, password length checks, matching password verification, duplicate checking, and bcrypt hashing) resides exclusively in the Rust backend.
  - Conforms to the repository guideline: all JS→Rust communication goes through `frontend/src/services/api.js`.
  - React components only manage UI input state and visual error/success banners, remaining decoupled from Tauri IPC primitives.
- **Alternatives considered**:
  - Directly calling `invoke('register', ...)` from React components: Violates the architecture guideline restricting IPC calls to `api.js`.
  - Validating and hashing passwords in JavaScript: Strictly forbidden by Constitution Principle VI.

### Decision 4: User Experience and Navigation Flow

- **Decision**:
  - Create a new Astro page `frontend/src/pages/register.astro` using `SplitScreen.astro` and `frontend/src/components/RegisterForm.jsx`.
  - Add a link on `LoginForm.jsx` ("Não tem uma conta? Cadastre-se") leading to `/register`.
  - Add a link on `RegisterForm.jsx` ("Já tem uma conta? Entrar") leading to `/login`.
  - Upon successful registration, the user is redirected to `/login?registered=true`, where `LoginForm.jsx` reads the query parameter and displays an informative success banner ("Conta criada com sucesso! Entre com seu email e senha.").
- **Rationale**:
  - Satisfies the clarified requirement (Session 2026-09-22: Option A).
  - Maintains 100% visual consistency with the existing Figma split-screen design branding.
  - Provides immediate feedback while allowing the user to verify login with their new credentials.
- **Alternatives considered**:
  - Modal registration dialog on login screen: Clutters the split-screen design and departs from Figma templates.
  - Automatic login redirect directly to `/dashboard`: Rejected during clarification to honor user prompt ("para logar na pagina inicial").

### Decision 5: Coexistence of Seeded Admin

- **Decision**: Retain the existing `DEFAULT_ADMIN_USERNAME` (`admin@admin.com` / `admin123`) seeded in `db::seed_admin`.
- **Rationale**:
  - Satisfies clarified Decision 2 (Session 2026-09-22: Option A).
  - Ensures full backward compatibility with existing tests in `auth.rs`, `db.rs`, and future E2E test suites.
  - Prevents accidental lockout in offline desktop environments.
- **Alternatives considered**:
  - Deleting or disabling `admin@admin.com` upon first user registration: Rejected during clarification.

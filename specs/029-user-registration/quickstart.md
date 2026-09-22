# Quickstart: Testing Account Creation and Password Hashing

This document outlines how to test and verify the account creation and password hashing feature.

---

## 1. Automated Testing

### Backend Rust Tests (`src-tauri`)

Run domain and IPC command tests:

```bash
cargo test --package research_hub_lib auth::tests
```

**Key test cases covered**:
- `register_creates_account_with_bcrypt_hash`: Verifies that a valid registration inserts an account into the SQLite `admins` table with a valid `$2b$12$...` hash.
- `register_normalizes_email_to_lowercase_and_trims`: Ensures emails like `" Prof@Ifes.edu.br "` are normalized to `"prof@ifes.edu.br"`.
- `register_rejects_duplicate_email`: Ensures trying to register an existing email returns `AppError::Validation("Este email já está cadastrado.")`.
- `register_rejects_password_mismatch`: Ensures non-matching confirmation returns `AppError::Validation`.
- `register_rejects_short_password`: Ensures passwords < 8 characters return `AppError::Validation`.
- `register_rejects_invalid_email_format`: Ensures emails without `@` or domain return `AppError::Validation`.
- `newly_registered_account_can_login`: Registers an account and immediately checks `verify_credentials`, asserting successful authentication.

### Frontend Component Tests (`frontend`)

```bash
npm test
```

**Key test cases covered**:
- `LoginForm.test.jsx`: Verifies the link to `/register` is present, and verifies that the `?registered=true` query parameter displays the success banner.
- `RegisterForm.test.jsx`: Verifies form rendering, input validation (matching passwords, minimum length), error message rendering upon API failure, and navigation to `/login?registered=true` upon successful registration.
- `api.test.js`: Verifies that `POST /auth/register` maps to `invoke('register', { email, password, passwordConfirm })`.

---

## 2. Manual Verification Flow

1. **Launch Application**:
   ```bash
   npm run tauri dev
   ```
2. **Access Registration Screen**:
   - On the initial login screen, click the link: **"Não tem uma conta? Cadastre-se"**.
   - Verify the URL changes to `/register` and the registration form is rendered with the split-screen branding.
3. **Test Validation Feedback**:
   - Enter mismatched passwords -> verify error message: *"A confirmação de senha não coincide com a senha digitada."*
   - Enter password with 5 characters -> verify error message: *"A senha deve conter no mínimo 8 caracteres."*
   - Enter an invalid email (e.g. `invalid-email`) -> verify error message: *"Formato de email inválido."*
4. **Successful Registration**:
   - Enter a valid email (`novo.professor@ifes.edu.br`) and matching password (`SenhaForte@2026`).
   - Click **"Criar Conta"**.
   - Verify immediate redirection to `/login?registered=true`.
   - Verify the green notification banner: *"Conta criada com sucesso! Faça login com suas credenciais."*
5. **Authenticate with New Account**:
   - Enter `novo.professor@ifes.edu.br` and `SenhaForte@2026` on the login screen.
   - Click **"Entrar"**.
   - Verify successful authentication and redirection to the `/dashboard`.
6. **Verify Database Security (Zero Plaintext)**:
   - Inspect the SQLite database in the app data directory:
     ```bash
     sqlite3 ~/.local/share/research_hub/hub.db "SELECT id, username, hashed_password FROM admins;"
     ```
   - Confirm that `hashed_password` starts with `$2b$12$` and the plaintext password `SenhaForte@2026` is nowhere in the database.

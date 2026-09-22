# IPC & API Contracts: Account Creation and Registration

## 1. Tauri IPC Command Contract

The Rust backend exposes a new asynchronous Tauri command registered in `tauri::generate_handler!`.

### Command: `register`

- **Signature**:
  ```rust
  #[tauri::command(async)]
  pub fn register(
      state: State<'_, AppState>,
      email: String,
      password: String,
      password_confirm: String,
  ) -> Result<(), AppError>
  ```
- **Arguments (IPC payload)**:
  ```json
  {
    "email": "professor@ifes.edu.br",
    "password": "MinhaSenhaSegura123",
    "passwordConfirm": "MinhaSenhaSegura123"
  }
  ```
- **Success Response**: `null` (Rust `Ok(())` deserialized as `null` in JavaScript)
- **Failure Response**:
  Serialized `AppError` object:
  ```json
  {
    "kind": "validation",
    "message": "Este email já está cadastrado."
  }
  ```
- **Possible Error Messages**:
  - `"Formato de email inválido."`
  - `"A senha deve conter no mínimo 8 caracteres."`
  - `"A confirmação de senha não coincide com a senha digitada."`
  - `"Este email já está cadastrado."`

---

## 2. Frontend IPC Bridge Contract (`frontend/src/services/api.js`)

All React components communicate through `apiFetch`. A new route is added to the internal `ROUTES` table:

```javascript
[/^\/auth\/register$/, 'POST', (_m, b) =>
  invoke('register', {
    email: b.email,
    password: b.password,
    passwordConfirm: b.password_confirm,
  })],
```

### Usage in Components

```javascript
await apiFetch('/auth/register', {
  method: 'POST',
  body: JSON.stringify({
    email: email.trim(),
    password,
    password_confirm: passwordConfirm,
  }),
});
```

---

## 3. Navigation & Query Parameter Contract

- **Registration Page**: `/register`
- **Login Redirection on Success**:
  - Target: `/login?registered=true`
  - When `registered=true` is present in `window.location.search`, `LoginForm.jsx` displays:
    ```jsx
    <div className="p-3 mb-4 text-sm text-green-800 bg-green-100 rounded-md">
      Conta criada com sucesso! Faça login com suas credenciais.
    </div>
    ```
- **Navigation Links**:
  - `LoginForm.jsx`: Link to `/register` (`"Não tem uma conta? Cadastre-se"`)
  - `RegisterForm.jsx`: Link to `/login` (`"Já tem uma conta? Entrar"`)

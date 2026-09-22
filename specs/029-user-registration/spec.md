# Feature Specification: Account Creation and Password Hashing

**Feature Branch**: `029-user-registration`  
**Created**: 2026-09-22  
**Status**: Draft  
**Input**: User description: "esse e o programa research_hub , que ja esta sendo trabalhado com speckit, so que em outro pc meu  , preciso agora de um sistema de hash para amazenar as senhas e uma pagina que permita criação de conta para logar na pagina inicial com email e senha"

## Clarifications

### Session 2026-09-22
- Q: Como deve ser a transição após o usuário preencher o formulário de cadastro e a conta ser criada com sucesso? → A: Redirecionar para a tela inicial de Login com notificação de sucesso para que o usuário informe email e senha na página inicial.
- Q: Como o administrador padrão semeado (admin@admin.com) deve ser tratado após a criação de novas contas? → A: Manter o admin padrão ativo coexistindo normalmente com as novas contas criadas.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create a new account with email and password (Priority: P1)

As a user of ResearchHub, I want to create an account by submitting my email address and a secure password on a dedicated registration page, so that my credentials are saved securely and I can use them to access the application.

**Why this priority**: Currently, the system only supports a single pre-seeded administrator credential. Enabling users to create their own accounts with secure password hashing is the primary capability required to personalize and protect access to the workspace.

**Independent Test**: Can be fully tested by opening the account creation page, filling in a new email and password, submitting the registration, verifying that credentials are securely persisted in storage in hashed form, and verifying redirection to the login view with confirmation feedback.

**Acceptance Scenarios**:

1. **Given** a user on the account creation page, **When** they submit a valid email and matching passwords that satisfy security criteria, **Then** the account is successfully created with a cryptographically hashed password, and the user is redirected to the login page with a success message.
2. **Given** an existing account registered with "prof@ifes.edu.br", **When** a user attempts to create another account with "prof@ifes.edu.br", **Then** the submission is rejected with an error message stating that the email is already in use.
3. **Given** a user filling in the registration form, **When** the password confirmation does not match the chosen password, **Then** the registration is blocked with a validation error indicating that passwords do not match.
4. **Given** a password shorter than 8 characters, **When** submitting the registration form, **Then** the system rejects the submission with a clear password complexity requirement error.

---

### User Story 2 - Authenticate on initial login screen with new account (Priority: P2)

As a registered user, I want to log in on the initial login screen using my registered email and password, so that I can securely unlock my curation session and access the data management features.

**Why this priority**: Creating an account is only valuable if the user can subsequently authenticate on the existing initial login page with those credentials.

**Independent Test**: Can be fully tested by registering a new account, navigating to the initial login screen, entering the newly registered email and password, and verifying that the application successfully grants access to the dashboard.

**Acceptance Scenarios**:

1. **Given** a newly registered account with valid credentials, **When** the user enters the registered email and password on the initial login screen and submits, **Then** authentication succeeds, a local session is established, and the user is redirected to the dashboard.
2. **Given** a registered account, **When** the user enters the correct email but an incorrect password, **Then** the system rejects the login attempt with an invalid credentials error without disclosing internal details.

---

### User Story 3 - Navigation between Login and Registration pages (Priority: P3)

As a user on the authentication screens, I want clear visual links to navigate between the login screen and the account creation screen, so that I can easily switch between logging in and creating a new account.

**Why this priority**: Users need an intuitive way to access the registration screen from the default initial landing page without having to know or manually type URL paths.

**Independent Test**: Can be fully tested by clicking the "Criar conta" link on the login screen to navigate to the registration screen, and clicking "Já tem uma conta? Entrar" on the registration screen to navigate back to the login screen.

**Acceptance Scenarios**:

1. **Given** the login screen, **When** the user clicks the link to create a new account, **Then** they are taken to the account creation page.
2. **Given** the account creation page, **When** the user clicks the link to return to login, **Then** they are taken back to the login screen.

---

### Edge Cases

- **Case-insensitive email matching**: Emails entered with uppercase or mixed case (e.g. `User@Example.COM`) must be normalized (lowercased and trimmed) to prevent duplicate accounts with differing capitalization.
- **Leading and trailing whitespace**: Whitespace around email addresses or passwords must not cause accidental validation failures or storage anomalies; emails must be trimmed automatically.
- **Pre-existing seeded admin coexistence**: Existing default admin credentials must continue to function normally, allowing existing tests and workflows to operate alongside newly created accounts.
- **Offline registration**: Account creation and password hashing must execute entirely locally on the desktop without network dependency or remote server requests.
- **Weak or blank passwords**: Passwords consisting solely of whitespace or fewer than 8 characters must be rejected before any hashing or database write occurs.

## Requirements *(mandatory)*

### Technical & Architectural Constraints

- **CON-001**: A aplicação DEVE ser um app desktop de processo único em Tauri 2.0, com núcleo em Rust e front-end em Astro/React/Tailwind compilado estaticamente e embarcado no binário, seguindo o protótipo Figma. Funciona integralmente sem rede.
- **CON-002**: A persistência DEVE usar SQLite em arquivo único, embarcado (`rusqlite`, feature `bundled`), no diretório de dados da aplicação. É proibido qualquer motor que exija servidor ou instalação separada.
- **CON-003**: Toda regra de negócio DEVE residir no processo Rust. É proibido SQL, acesso a arquivos ou duplicação de regra no JavaScript; a comunicação se dá por IPC do Tauri, concentrada em um único módulo do front-end (`frontend/src/services/api.js`).
- **CON-004**: Todas as entidades do domínio exigem CRUD completo operado unicamente por perfil de Admin (sem fluxo de aprovação). Todos os usuários cadastrados operam com privilégios de Admin local.
- **CON-005**: Paridade funcional precede melhoria. Correções de comportamento preexistente DEVEM ser declaradas como escopo explícito desta spec, nunca aplicadas em silêncio.
- **CON-006**: Alvos de release são Linux e Windows. macOS, assinatura de código, auto-update e bancos não-SQLite estão fora de escopo por decisão de arquitetura.

### Functional Requirements

- **FR-001**: System MUST provide an account creation interface accessible via link from the initial login screen.
- **FR-002**: System MUST validate email format and reject invalid or malformed email addresses.
- **FR-003**: System MUST require a password confirmation field during registration and reject submissions if the password and confirmation do not match.
- **FR-004**: System MUST enforce a minimum password length of 8 characters and reject empty or whitespace-only passwords.
- **FR-005**: System MUST cryptographically hash all account passwords using a secure, salted one-way hashing function before saving to persistent storage; plaintext passwords MUST NEVER be stored, persisted, or logged.
- **FR-006**: System MUST ensure email uniqueness and reject registration if an account with the specified email (case-insensitive) already exists.
- **FR-007**: System MUST provide clear, localized feedback on registration failure (e.g. duplicate email, invalid format, mismatched passwords, insufficient password length).
- **FR-008**: System MUST redirect the user to the initial login screen upon successful registration, displaying a success notification.
- **FR-009**: System MUST allow users to authenticate on the initial login screen using credentials registered through the account creation page.
- **FR-010**: All credential validation, uniqueness checks, and password hashing MUST be executed within the backend process, invoked from the frontend through the centralized API communication bridge.

### Key Entities *(include if feature involves data)*

- **Admin Account**: Represents an authorized user profile in the local application. Key attributes include unique email identifier, cryptographically salted and hashed password, and local administrative access.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can complete account registration in under 30 seconds via the registration interface.
- **SC-002**: 100% of registered passwords in persistent storage are stored as cryptographic hashes; 0% plaintext passwords exist in storage.
- **SC-003**: 100% of registration attempts with duplicate emails, invalid formats, or mismatched passwords are rejected with clear, user-facing error messages.
- **SC-004**: Users are able to log in on the initial login screen on their first attempt immediately following successful account registration.
- **SC-005**: Account creation and subsequent login function 100% offline without requiring any external network or internet connection.

## Assumptions

- Single Admin role: In accordance with Constitution Principle IV, all registered accounts have local administrative privileges with no multi-tier role hierarchy or approval workflows.
- Existing seeded administrator (`admin@admin.com`) remains functional alongside newly registered accounts.
- Because this is an offline desktop application, email verification links or password reset emails are not applicable; authentication is strictly local.
- Existing styling and split-screen layout conventions used on the login screen are reused for visual consistency on the registration screen.

# Feature Specification: Local Session Authentication

**Feature Branch**: `021-local-session-auth`
**Created**: 2026-09-02
**Reference**: `docs/estudo-migracao-rust-tauri.md` §AD-05, §10.3.5

## User Scenarios & Testing

### User Story 1 - Operations require a real session (Priority: P1)

Data operations are refused until the curator has signed in.

**Why this priority**: Authentication is currently decorative. `security.py` issues a JWT, but no endpoint decodes it — the `Authorization` header the components send is ignored, and any caller can read or delete every table without credentials. Porting that as-is would carry a security hole into the new system for no reason.

**Acceptance Scenarios**:

1. **Given** no session, **When** any data command is called, **Then** it is refused with an unauthenticated error.
2. **Given** valid credentials, **When** signing in, **Then** a session opens and commands succeed.
3. **Given** an open session, **When** signing out, **Then** commands are refused again.
4. **Given** invalid credentials, **When** signing in, **Then** the error does not reveal whether the username or the password was wrong.

### Edge Cases

- Application restarted → session does not persist; sign-in is required again.
- Sign-out while an import is running → the running operation finishes; the next one is refused.
- `login` itself must never be gated, or no one could ever sign in.

## Requirements

- **FR-001**: Every data command MUST verify an open session before doing any work.
- **FR-002**: `login` MUST NOT require a session and MUST open one on success.
- **FR-003**: Credentials MUST be verified with bcrypt against the `admins` table.
- **FR-004**: A failed sign-in MUST NOT reveal which field was wrong, and MUST take comparable time for unknown users. *(Declared improvement.)*
- **FR-005**: `logout` MUST close the session in the Rust process, not only clear browser storage.
- **FR-006**: `session_status` MUST report whether a session is open.
- **FR-007**: Sessions MUST live in process memory only and MUST NOT survive a restart.
- **FR-008**: No JWT is issued or validated; the token the interface stores is a UI flag with no cryptographic meaning.

## Success Criteria

- **SC-001**: Every data command refuses to run without a session.
- **SC-002**: Signing in with the seeded admin succeeds; wrong password fails.
- **SC-003**: Signing out closes the session in the backend process.
- **SC-004**: Restarting the application requires signing in again.

## Assumptions

- Single Admin profile, as the Constitution requires.
- Local desktop use means no channel to protect, so JWT would add ceremony without security.
- Default credentials stay as they are; changing them is out of scope.

## Out of Scope

- Password changes, multiple users, roles, lockout policies.

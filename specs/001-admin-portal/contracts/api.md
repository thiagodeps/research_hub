# REST API Contracts

A API FastAPI fornecerá os seguintes grupos de endpoints consumidos pelo frontend Astro.
Todos os endpoints (exceto `/auth/login`) exigem header `Authorization: Bearer <jwt_token>`.

## 1. Autenticação

### `POST /api/v1/auth/login`
- **Request**: `{ "email": "admin@example.com", "password": "..." }`
- **Response (200)**: `{ "access_token": "ey...", "token_type": "bearer" }`
- **Response (401)**: `{ "detail": "Credenciais inválidas" }`

## 2. Padrão CRUD (Para todas as 12 entidades do Domínio)

Usaremos `University` como exemplo. O mesmo contrato se aplica substituindo `/universities` por `/researchers`, `/campuses`, etc.

### `GET /api/v1/universities`
- **Query Params**: `page`, `limit`, `search` (opcional).
- **Response (200)**:
  ```json
  {
    "items": [
      { "id": 1, "name": "IFES", "abbreviation": "IFES" }
    ],
    "total": 1
  }
  ```

### `GET /api/v1/universities/{id}`
- **Response (200)**: Dados detalhados da entidade.
- **Response (404)**: Entidade não encontrada.

### `POST /api/v1/universities`
- **Request**: Payload completo da entidade.
- **Response (201)**: Entidade criada com `id`.

### `PUT /api/v1/universities/{id}`
- **Request**: Campos a atualizar.
- **Response (200)**: Entidade atualizada.

### `DELETE /api/v1/universities/{id}`
- **Response (204)**: Apagado com sucesso.
- **Response (409)**: Conflito - Bloqueio de deleção por possuir dependentes (Edge case implementado).

## 3. Operações Especiais (Admin)

### `POST /api/v1/merge/{entity_type}`
- **Description**: Realiza a fusão de dois registros.
- **Request**:
  ```json
  {
    "source_ids": [1, 2],
    "resolved_data": {
      "name": "Nome Consolidado",
      "emails": ["email1@test.com", "email2@test.com"]
    }
  }
  ```
- **Response (200)**: O novo registro criado, e o status dos anteriores migrados para `Soft Delete`.

### `POST /api/v1/link`
- **Description**: Realiza a Herança / Associação de registros distintos.
- **Request**:
  ```json
  {
    "parent_type": "advisorship",
    "parent_id": 10,
    "child_type": "article",
    "child_id": 55
  }
  ```
- **Response (200)**: Sucesso na associação.

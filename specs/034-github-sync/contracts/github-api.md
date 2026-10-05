# Contrato: Interações com a API do GitHub

**Feature**: `034-github-sync` | **Date**: 2026-09-29

Endpoints usados pelo `sync_github.rs`. Base URL injetável
(`https://api.github.com` em produção; wiremock nos testes — R8). Autenticação
quando há token: `Authorization: Bearer {token}`; sempre
`X-GitHub-Api-Version: 2022-11-28` e `User-Agent: research-hub`.

## Download (US1)

### URL de arquivo (raw ou blob)

1. `GET /repos/{owner}/{repo}/contents/{path}?ref={ref}`
   - `Accept: application/vnd.github.raw` → corpo = bytes do arquivo.
   - Resposta 200 traz `Content-Length`; stream para arquivo temporário em
     chunks (R1), emitindo `sync://progress`.
   - `404` → `not_found`; `401/403` sem token → `github_api` com mensagem
     "URL exige acesso autenticado; configure o token" (US1-3).

### URL de release asset

1. `GET /repos/{owner}/{repo}/releases/tags/{tag}` → localizar asset pelo nome.
2. `GET /repos/{owner}/{repo}/releases/assets/{id}` com
   `Accept: application/octet-stream` → bytes (mesmo tratamento de streaming).

## Upload (US2) — Git Data API, ordem fixa

1. **Checagem prévia** (`github_check_destination`):
   - `GET /repos/{owner}/{repo}/git/ref/heads/{branch}` → ponta atual
     (`object.sha`). 404 → `not_found` ("branch não existe; crie-a no GitHub").
   - `GET /repos/{owner}/{repo}/contents/{path}?ref={branch}` → `file_sha` se
     existir (para confirmação de sobrescrita).
2. **Envio** (`github_upload`), após política + export + limite:
   1. `POST /repos/{o}/{r}/git/blobs` — `{ content: base64(zip), encoding: "base64" }` → `blob_sha`.
   2. `GET .../git/ref/heads/{branch}` → `base_sha` (re-lido no momento do commit).
   3. `POST .../git/trees` — `{ base_tree: base_sha, tree: [{ path, mode: "100644", type: "blob", sha: blob_sha }] }` → `tree_sha`.
   4. `POST .../git/commits` — `{ message: "Export canônico ResearchHub ({data})", tree: tree_sha, parents: [base_sha] }` → `commit_sha`.
   5. `PATCH .../git/refs/heads/{branch}` — `{ sha: commit_sha, force: false }`.
      - `422` (ponta mudou desde a leitura) → reexecutar 2–5 uma vez; se
        persistir, `github_api` com "o repositório mudou durante o envio;
        tente novamente" (FR-007, nada gravado pelo app).
      - `403` → sem permissão de escrita; mensagem explícita de que nada foi
        gravado (US2-3).

## Token (US3)

- `GET /user` → valida o token; header de resposta `X-OAuth-Scopes` informa os
  escopos (exibidos sem o valor do token).

## Limites respeitados (FR-010)

- Tamanho: blob de até 100 MB (limite da Git Data API ≥ 100 MB; app impõe
  100 MB — Q3). Checagem ANTES do `POST /git/blobs` e durante o download.
- Taxa: operações são iniciadas pelo usuário (sem polling em background);
  `403` com header `X-RateLimit-Remaining: 0` → `github_api` com "limite de
  requisições do GitHub atingido; tente mais tarde".

## O que NÃO é usado (escopo)

- Criação de refs/branches (proibido — Q1), releases, Git LFS, clonagem git,
  webhooks, GitHub Apps (OAuth app flow) — apenas PAT do curador (R4).

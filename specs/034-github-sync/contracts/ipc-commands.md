# Contrato IPC: Comandos de sincronização GitHub

**Feature**: `034-github-sync` | **Date**: 2026-09-29

Comandos Tauri novos, todos `async` (rede), registrados em
`src-tauri/src/lib.rs` no `invoke_handler`. Convenções existentes valem:
parâmetros em snake_case (o Tauri mapeia camelCase do JS), erros como wire
`{ kind, message }` via `AppError`, e `api.js` é o único ponto de chamada no
front-end.

## Configuração e token

### `github_get_config` → `Record`

- **Params**: `project: "horizon" | "src"`
- **Retorna**: `{ repo, branch, path, last_source_url, has_token, token_hint }`
  (`token_hint` = últimos 4 caracteres ou `null`; o valor completo nunca
  atravessa IPC de volta — FR-006).
- **Erros**: `sync_config` (projeto desconhecido).

### `github_set_config` → `null`

- **Params**: `project`, `repo`, `branch`, `path`
- **Validação** (domínio puro): formato `owner/name`, branch/path não vazios,
  sem `..`, `.zip` no path do Horizon. Escrita atômica do arquivo (R10).
- **Erros**: `sync_config`, `sync_policy` (n/a — destino é permitido para os
  dois projetos; o bloqueio é só de envio).

### `github_save_token` → `null`

- **Params**: `token: string`
- **Efeito**: grava no `sync_config.json` com permissão restrita (R4). Nunca
  loga o valor.
- **Erros**: `sync_config` (token vazio).

### `github_test_token` → `Record`

- **Efeito**: `GET /user` (e leitura do header `X-OAuth-Scopes`).
- **Retorna**: `{ login, scopes, valid: true }`.
- **Erros**: `network`, `github_api` (401 → mensagem "token inválido ou
  revogado"), `sync_config` (sem token salvo).

### `github_remove_token` → `null`

- **Efeito**: remove a chave `token` do arquivo; inspeção posterior confirma
  vestígio zero (SC-006).

## Download (US1 — Horizon e SRC)

### `github_download` → `Record` (mesma forma do import manual correspondente)

- **Params**: `project`, `url: string`
- **Fluxo**: normaliza URL (R2) → baixa com streaming para temporário no
  app-data (aborta > 100 MB com `too_large`) → valida formato do projeto
  (zip canônico no Horizon; JSON consolidado no SRC) → chama o import manual
  existente (snapshot incluso) → remove temporário → retorna contagens.
- **Erros**: `validation` (URL/host estranho, formato não bate com o
  projeto), `network` (DNS/timeout/conexão), `github_api` (404 = arquivo não
  existe na ref; 401 sem token em repo privado → mensagem aponta para o
  TokenDialog), `too_large`, `file`, e os erros herdados do import
  (`validation`/`file` com as mesmas mensagens do manual).
- **Garantia**: em qualquer erro, base local intacta (FR-007, SC-002).

## Upload (US2 — só Horizon)

### `github_check_destination` → `Record`

- **Params**: `project`
- **Retorna**: `{ repo, branch, path, branch_exists, file_exists, file_sha }`
  (lê ref + contents; usado pela UI para confirmar sobrescrita — US2-2).
- **Erros**: `network`, `github_api`, `not_found` (branch inexistente), `sync_config` (destino não configurado).

### `github_upload` → `Record`

- **Params**: `project`, `confirm_overwrite: bool`
- **Fluxo**: política (R4/data-model regra 1: `project == "src"` → `sync_policy`)
  → gera o zip canônico **agora** (Q2 — mesmo fluxo do export manual) →
  verifica tamanho (≤ 100 MB, senão `too_large` antes de transferir) → lê ref
  da branch (**404 → `not_found` "crie a branch no GitHub"; nunca cria** — Q1)
  → Git Data API: blob → tree → commit → PATCH ref (R3) → retorna
  `{ commit_sha, html_url, replaced: bool }`.
- **Guarda de sobrescrita**: se o arquivo de destino existe com SHA diferente
  do último envio conhecido e `confirm_overwrite == false`, retorna erro
  `conflict` com mensagem pedindo confirmação explícita (US2-2).
- **Erros**: `sync_policy`, `sync_config`, `too_large`, `not_found`,
  `network`, `github_api` (403 = sem permissão de escrita, mensagem "nada foi
  gravado"; 422 na ref = ponta mudou → re-ler e informar para tentar de
  novo), `conflict`, `zip`.

## Kinds de erro novos em `error.rs`

| Kind | Variante | Uso |
|------|----------|-----|
| `network` | `Network(String)` | DNS, timeout, conexão recusada — "sem acesso a github.com; verifique a conexão" |
| `github_api` | `GithubApi { status, hint }` | Qualquer resposta de erro do GitHub com `hint` acionável |
| `too_large` | `TooLarge { limit_bytes }` | Limite de 100 MB (antes ou durante a transferência) |
| `sync_config` | `SyncConfig(String)` | Config ausente/inválida, projeto desconhecido, token vazio |
| `sync_policy` | `SyncPolicy(String)` | Recusa de envio do SRC (FR-005) |
| `conflict` | *(existente)* | Sobrescrita não confirmada |

Reuso: `not_found`, `validation`, `file`, `zip` com semântica atual. Nenhum
`kind` novo contém o token em `message` — coberto por teste de inspeção
(SC-005).

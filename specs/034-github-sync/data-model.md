# Data Model: Sincronização de exports com repositório GitHub

**Feature**: `034-github-sync` | **Date**: 2026-09-29

Nenhuma migração de banco: o SQLite permanece inalterado. A persistência nova
é um único arquivo JSON no app-data; as demais entidades abaixo são transientes
ou de domínio puro em `sync_domain.rs`.

## Entidades

### SyncConfigFile (persistida — `app-data/sync_config.json`)

| Campo | Tipo | Regras |
|-------|------|--------|
| `version` | integer | Sempre `1`; leitura tolera ausência (R10) |
| `token` | string \| null | Token pessoal GitHub; `null`/ausente = sem token. Nunca logado, nunca retornado completo |
| `projects.horizon` | SyncProjectConfig | Destino/envio do Horizon |
| `projects.src` | SyncProjectConfig | Destino do SRC (presente por desenho; envio bloqueado por política FR-005) |

### SyncProjectConfig

| Campo | Tipo | Regras de validação (`sync_domain.rs`) |
|-------|------|----------------------------------------|
| `repo` | string | Formato `owner/name`; sem espaços; `owner` e `name` não vazios |
| `branch` | string | Não vazia; sem espaços; não pode começar com `-` |
| `path` | string | Não vazio; sem `..`; terminado em `.zip` para o Horizon (export) |
| `last_source_url` | string \| null | Última URL usada em download, para preencher o campo na UI |

**Unicidade/identidade**: um registro por projeto (chave `horizon`/`src`) — escrita substitui o objeto inteiro do projeto (CRUD direto, Princípio IV).

### AccessToken (transiente de uso — só em memória durante operações)

| Propriedade | Regra |
|-------------|-------|
| Presença | Opcional (FR-006); exigido para repo privado ou envio |
| Escopo exigido | Leitura: `repo` (ou `public_repo`); envio: `repo` com permissão de escrita |
| Exposição | UI mostra apenas indicador de presença + últimos 4 caracteres; proibido em qualquer mensagem/log (FR-006, SC-005) |

### SyncOperation (transiente — eventos `sync://progress`)

| Campo | Valores |
|-------|---------|
| `operation` | `download` \| `upload` |
| `project` | `horizon` \| `src` |
| `phase` | `started` → `transferring` → `importing` (download) \| `committing` (upload) → `done` \| `failed` |
| `bytes_done` / `bytes_total` | Progresso de transferência; `bytes_total` pode ser null antes do `Content-Length` |

**Transições**: toda operação termina em `done` ou `failed` — sem estado parcial na base local (FR-007). Falha após transferência (ex.: import recusa arquivo) ainda é `failed`, e a base permanece como estava (import é tudo-ou-nada já garantido).

## Regras de domínio (puras, em `sync_domain.rs`)

1. **Política de envio (FR-005)**: `project == "src"` + operação de envio → erro `sync_policy` SEMPRE, mesmo com destino privado. Função pura testável sem rede.
2. **Limite de tamanho (Q3)**: arquivos > 100 MB → `too_large` antes de iniciar transferência; durante o download, ultrapassar o limite aborta com `too_large`.
3. **Normalização de URL (R2)**: aceita `raw.githubusercontent.com/{o}/{r}/{ref}/{path}`, `github.com/{o}/{r}/blob/{ref}/{path}`, `github.com/{o}/{r}/releases/download/{tag}/{asset}` → vira (endpoint, owner, repo, ref/asset, path). URL de outro host → `validation`.
4. **Branch existente (Q1)**: envio exige ref existente; 404 na leitura da ref → `not_found` com mensagem orientando criar a branch no GitHub. Nunca `POST /git/refs`.
5. **Export no ato (Q2)**: o envio chama o export do Horizon vigente e envia esse buffer — não existe opção "reenviar último arquivo".
6. **Sobrescrita consciente (US2-2)**: antes do commit, o destino atual é lido (`github_check_destination`); se existe arquivo com conteúdo diferente, a UI exige confirmação explícita.

## Relacionamentos

- `SyncProjectConfig` → usado por download (origem sugerida) e upload (destino obrigatório).
- `SyncOperation` → referencia `project` + configuração vigente no instante do início (snapshot em memória; mudanças de config não afetam operação em curso).
- Import/export manuais → intocados; download reusa os mesmos comandos de import por baixo (R7).

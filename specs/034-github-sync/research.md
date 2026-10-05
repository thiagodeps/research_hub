# Research: Sincronização de exports com repositório GitHub

**Feature**: `034-github-sync` | **Date**: 2026-09-29

Decisões técnicas para os pontos em aberto do Technical Context e para as
integrações externas. Cada decisão traz racional e alternativas consideradas.

## R1 — Cliente HTTP no núcleo Rust

**Decision**: `reqwest` com `default-features = false` + `features = ["rustls-tls", "stream"]`, comandos Tauri `async fn` rodando no runtime tokio do próprio Tauri (`tauri::async_runtime`).

**Rationale**: TLS via `rustls` elimina a dependência de OpenSSL — essencial para compilar os dois alvos de release (Linux e Windows, Princípio III/Technical Stack) sem DLLs de sistema. A feature `stream` permite baixar o corpo em chunks (100 MB sem ficar na memória, FR-008/SC-001). Comandos async são nativos no Tauri 2 e não bloqueiam a thread pool para transferências longas.

**Alternatives considered**:
- `ureq` (bloqueante, rustls por padrão) — mais simples, porém travaria uma thread por transferência de até 100 MB e complica progresso/cancelamento.
- Shell out para `curl`/`git` — quebra a autocontidão (depende de binários do SO) e a auditabilidade da lógica em Rust (Princípio VI).
- `reqwest` com native-tls — reproduz o problema de OpenSSL no Windows/CI.

## R2 — Mecanismo de download (origem dos bytes)

**Decision**: normalizar qualquer URL aceita para a **GitHub Contents API** com `Accept: application/vnd.github.raw` (`GET /repos/{owner}/{repo}/contents/{path}?ref={branch}`); URLs de release asset usam a **Release Assets API** (`GET .../releases/assets/{id}` com `Accept: application/octet-stream`). Autenticação opcional via header `Authorization: Bearer {token}`.

**Rationale**: uma única via autenticada cobre repositórios públicos e privados (o host `raw.githubusercontent.com` trata mal tokens em alguns cenários privados), e o endpoint informa tamanho (`Content-Length`/campo `size`) — permitindo checar o limite de 100 MB antes/de durante a transferência (FR-010, Q3).

**Alternatives considered**:
- GET direto em `raw.githubusercontent.com` — falha de forma inconsistente para repos privados com token.
- Clonar o repositório via `git2` — pesado, exige `libgit2`, retorna histórico desnecessário.
- GraphQL — desnecessário para download de arquivo único.

## R3 — Mecanismo de envio (destino dos bytes)

**Decision**: fluxo **Git Data API** em quatro passos — `POST /repos/{o}/{r}/git/blobs` (conteúdo do export), `GET /repos/{o}/{r}/git/ref/heads/{branch}` (SHA da ponta; **404 → erro tipado de branch inexistente, nunca cria ref** — Q1), `POST /repos/{o}/{r}/git/trees` (árvore nova com o blob no caminho informado, base = ponta atual), `POST /repos/{o}/{r}/git/commits` + `PATCH /repos/{o}/{r}/git/refs/heads/{branch}` (commit e avanço da ref).

**Rationale**: a Contents API (PUT em `/contents/{path}`) é mais simples mas limita o arquivo a ~1 MB; o requisito Q3 é **100 MB**, então o fluxo Git Data API é obrigatório e serve uniformemente a qualquer tamanho. Ele devolve o SHA do commit (confirmação da US2) e detecta concorrência: se a ref mudou entre a leitura e o `PATCH`, o GitHub recusa o avanço (`422`) → re-ler a ponta e reavaliar (FR-007).

**Alternatives considered**:
- Contents API para tudo — inviável acima de ~1 MB (contradiz Q3=B).
- Release assets como destino — fora do escopo (Assumption: commit direto na branch; sem releases).
- Git LFS — fora do escopo declarado na spec.

## R4 — Armazenamento do token e da configuração por projeto

**Decision**: arquivo JSON `sync_config.json` no diretório app-data, criado com permissão restrita ao usuário (0600 em Unix; ACL de perfil em Windows), contendo `{ "token": string|null, "projects": { "horizon": {...}, "src": {...} } }`. Remover o token = apagar a chave (ou o arquivo) — verificável por inspeção (SC-006).

**Rationale**: mantém o mesmo modelo de persistência local do aplicativo (arquivos no app-data, sem servidor), evita colocar credenciais dentro do SQLite que já contém os dados curados, e torna o "vestígio zero" trivialmente auditável. O token nunca é logado nem retornado completo aos comandos de leitura (só um indicador de presença/últimos 4 caracteres na UI).

**Alternatives considered**:
- Crate `keyring` (SO keychain) — segurança marginalmente melhor, mas adiciona deps específicas por plataforma e conflita com o espírito autocontido (Princípio V) sem ameaça real nova: quem lê o arquivo já lê o SQLite com os mesmos dados.
- Tabela em SQLite — mistura credencial com dados curados; backup/export do banco carregaria o token junto.

## R5 — Erros tipados de sincronização

**Decision**: estender `AppError` (thiserror) com variantes novas e `kind`s no wire `{ kind, message }`: `Network` → `network`, `GithubApi { status, hint }` → `github_api`, `TooLarge { limit_bytes }` → `too_large`, `SyncConfig` → `sync_config`, `SyncPolicy` → `sync_policy` (recusa do envio do SRC, FR-005). Reuso de `NotFound` para URL/branch/caminho inexistente e `File` para falha de I/O local. Nenhum desses erros inclui o token (FR-006).

**Rationale**: o padrão `{ kind, message }` já é o contrato de erro do app (error.rs); mensagens acionáveis ("o que houve + o que fazer") são exigidas pelo FR-007 e testáveis por `kind` nos testes de integração.

**Alternatives considered**: uma única variante `Sync(String)` — mais curta, mas impede o front-end de diferenciar offline de permissão de limite, exatamente o que o FR-007 pede para exibir.

## R6 — Progresso para o front-end (FR-008)

**Decision**: eventos Tauri `sync://progress` emitidos pelo `AppHandle` com payload `{ operation: "download"|"upload", project, phase, bytes_done, bytes_total, detail? }`, fases `started → transferring → importing|committing → done|failed`. O `api.js` expõe um wrapper de assinatura (`onSyncProgress`).

**Rationale**: streaming com progresso exige push do núcleo; eventos são o mecanismo nativo do Tauri 2 e não criam polling. Fases explícitas deixam a UI distinguir "baixando" de "importando" (o import reusa os fluxos manuais).

**Alternatives considered**: comando de polling `github_status` — mais código, latência de UI, sem benefício.

## R7 — Reuso do import existente (zero regressão)

**Decision**: `github_download` grava os bytes em arquivo temporário no app-data e chama os imports existentes (`import_canonical_zip` / `import_src_json`) — a lógica de validação, snapshot e substituição permanece a MESMA função já testada (FR-003, FR-001). O import atual é extraído para aceitar caminho de arquivo (já é o formato atual) — nenhuma alteração semântica.

**Rationale**: garante SC-004 (regressão zero) por construção: o download não reimplementa validação, só muda a origem dos bytes, como a spec exige ("só muda a origem").

**Alternatives considered**: importar de bytes em memória — duplicaria o caminho de validação ou forçaria refatoração arriscada do import já estável.

## R8 — Estratégia de testes sem rede real

**Decision**: dev-dependency `wiremock` sobe um servidor HTTP local nos testes de integração (`src-tauri/tests/sync_github.rs`); os testes apontam a base URL do cliente para o mock (a base URL é injetável em `sync_github.rs`, default `https://api.github.com`). Cenários: download público, com token, 404, corpo maior que o limite, upload feliz (blob/tree/commit/ref), branch 404, sem permissão (403), conflito de ref (422). Componentes React testados com Vitest (mockando `api.js`).

**Rationale**: Princípio III proíbe depender de rede externa para a suíte; a injeção de base URL isola a mesma lógica que vai a produção.

**Alternatives considered**: `httpmock` — equivalente, mas `wiremock` é async-native e casa com reqwest/tokio; testar contra o GitHub real — proibido em CI (flaky, credenciais).

## R9 — UI: onde os controles moram

**Decision**: nenhum endpoint novo de página (ROUTES permanece 21). Na área Horizon: o controle de export ganha "Enviar para o GitHub" e o de import ganha "Baixar do GitHub (URL)". Na área SRC: apenas o download por URL (o envio nem é renderizado — FR-005: "a operação não é oferecida"). Um `TokenDialog` compartilhado acessível das duas áreas.

**Rationale**: menor churn visual e de rotas; a separação por projeto (FR-004) fica explícita na composição dos componentes, e o SRC demonstra o desenho "política desabilitada, estrutura presente".

**Alternatives considered**: página `/sync` nova — inflaria ROUTES e afastaria o controle do contexto onde o export já acontece.

## R10 — Versionamento de formato do arquivo de configuração

**Decision**: `sync_config.json` ganha campo `"version": 1`; leitura tolera ausência (assume 1) e campos extras são preservados (sem rewrite agressivo). Escrita sempre completa e atômica (escreve `.tmp` + rename) para não corromper a config se o app morrer no meio.

**Rationale**: o usuário previu mudança de política do SRC (FR-005 é reversível); versionar o arquivo evita migração ad hoc depois, e a escrita atômica protege o token de truncamento.

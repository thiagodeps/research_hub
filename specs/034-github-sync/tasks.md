---

description: "Task list for SEP-034 GitHub sync"
---

# Tasks: Sincronização de exports com repositório GitHub

**Input**: Design documents from `/specs/034-github-sync/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: OBRIGATÓRIOS — Constituição I (TDD Estrito): cada task de implementação é precedida (ou acompanhada) da task de teste correspondente, que DEVE falhar antes (red) e passar depois (green).

**Organization**: Tasks agrupadas por user story (US1 download P1, US2 envio P2, US3 token P3) para implementação e validação independentes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Pode rodar em paralelo (arquivos diferentes, sem dependência de task incompleta)
- **[Story]**: User story dona da task (US1, US2, US3)
- Caminhos exatos em todas as descriptions

## Path Conventions

Aplicação desktop Tauri: núcleo em `src-tauri/src/`, testes de integração em `src-tauri/tests/`, front-end em `frontend/src/`, testes front em `frontend/tests/unit/`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Dependências e esqueleto dos módulos novos

- [X] T001 Adicionar dependências em src-tauri/Cargo.toml: `reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "stream", "json"] }` em [dependencies]; `wiremock = "0.6"` e `tempfile = "3"` em [dev-dependencies] (comentário de justificativa no estilo do arquivo — ver plan.md R1); `cargo check` verde sem OpenSSL de sistema
- [X] T002 Criar esqueletos `src-tauri/src/sync_domain.rs` e `src-tauri/src/sync_github.rs` (doc comments apontando para research.md R1–R10) e registrá-los em `src-tauri/src/lib.rs` (`mod sync_domain; mod sync_github;` junto dos demais módulos); `cargo test` permanece verde (130 testes da 033 intactos — SC-004)

**Checkpoint**: compila com as deps novas, zero comportamento novo.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Erros tipados, persistência da configuração e comandos de config — bloqueiam TODAS as user stories

**⚠️ CRITICAL**: Nenhuma story começa antes deste phase completo

- [X] T003 [P] TDD: escrever testes unitários (red) para os kinds novos em src-tauri/src/error.rs — `Network→"network"`, `GithubApi{status,hint}→"github_api"`, `TooLarge{limit_bytes}→"too_large"`, `SyncConfig→"sync_config"`, `SyncPolicy→"sync_policy"`, cada um com mensagem acionável (FR-007) e SEM possibilidade de conter o token (kind() não recebe dados sensíveis); depois implementar as variantes thiserror e o mapeamento em `kind()`
- [X] T004 [P] TDD: escrever testes (red) em src-tauri/src/sync_domain.rs (usando `tempfile`) para `sync_config.json` conforme data-model.md — schema `{version:1, token, projects:{horizon,src}}`; validações: `repo` no formato `owner/name` sem espaços, `branch` não vazia sem espaços, `path` não vazio sem `..` e terminado em `.zip` para horizon, `last_source_url` opcional; escrita atômica (`.tmp` + rename) e permissão restrita 0600 em Unix (R4/R10); depois implementar `SyncConfigFile` com load/save
- [X] T005 TDD: testes de integração (red) para os comandos de configuração em src-tauri/tests/sync_github.rs — `github_get_config` retorna `{repo,branch,path,last_source_url,has_token,token_hint}` onde `token_hint` são os ÚLTIMOS 4 caracteres ou null (nunca o valor completo — FR-006), projeto desconhecido → `sync_config`; `github_set_config` valida as regras do T004 e rejeita com `sync_config`; depois implementar ambos em src-tauri/src/commands.rs e registrar em `src-tauri/src/lib.rs` (invoke_handler)

**Checkpoint**: config CRUD por projeto funcionando por IPC; token ainda não tem UI nem uso em rede.

---

## Phase 3: User Story 1 — Baixar export do GitHub e importar (Priority: P1) 🎯 MVP

**Goal**: Curador informa a URL de um export no GitHub (Horizon zip ou consolidado SRC, público ou privado) e a base do projeto é carregada com as MESMAS garantias do import manual — snapshot, validação, tudo-ou-nada.

**Independent Test**: Com um export válido hospedado (mock wiremock nos testes; GitHub real no quickstart), importar só pela URL — sem diálogo de arquivo — e conferir contagens idênticas ao import manual do mesmo arquivo.

### Tests for User Story 1 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T006 [P] [US1] TDD: testes de normalização de URL (red) em src-tauri/src/sync_domain.rs — `raw.githubusercontent.com/{o}/{r}/{ref}/{path}`, `github.com/{o}/{r}/blob/{ref}/{path}` e `github.com/{o}/{r}/releases/download/{tag}/{asset}` viram `(endpoint, owner, repo, ref/asset, path)` conforme contracts/github-api.md; host estranho ou URL malformada → `validation`; depois implementar `normalize_url`
- [X] T007 [US1] TDD: testes wiremock (red) em src-tauri/tests/sync_github.rs para o cliente de download — 200 com `Accept: application/vnd.github.raw` faz streaming para arquivo temporário em chunks emitindo eventos `sync://progress` (fases started→transferring, bytes_done/bytes_total); 404 → `not_found`; 401 sem token → `github_api` com mensagem "URL exige acesso autenticado; configure o token" (US1-3); corpo que estoura 100 MB durante o stream → `too_large` e temporário removido (Q3); depois implementar `download_to_temp` em src-tauri/src/sync_github.rs com base URL injetável (default `https://api.github.com` — R8)
- [X] T008 [US1] TDD: testes de integração (red) do comando `github_download` em src-tauri/tests/sync_github.rs — sucesso público retorna as MESMAS contagens do `import_canonical_zip`/`import_src_json` manual do mesmo arquivo (SC-001); com token pré-gravado no config (seed via T004) baixa de repo privado (US1-2); zip informado na área src → `validation` (FR-009); QUALQUER falha deixa a base intacta (contagens antes/depois — SC-002) e o temporário removido; depois implementar `github_download` em src-tauri/src/commands.rs reusando os imports existentes por caminho de arquivo (R7 — zero refatoração do import) e registrar em lib.rs

### Implementation for User Story 1

- [X] T009 [P] [US1] Adicionar rota `POST /github/download` no array ROUTES de frontend/src/services/api.js ANTES dos padrões genéricos `/:entity` + testes unitários (primeiro red) em frontend/tests/unit/api.test.js (seção nova `/github/*`)
- [X] T010 [US1] TDD: testes Vitest (red) em frontend/tests/unit/SyncPanel.test.jsx e depois implementar src-tauri→frontend/src/components/SyncPanel.jsx (seção de download): input de URL pré-preenchido com `last_source_url` (GET `/github/config`), progresso via `onSyncProgress` (contracts/frontend-routes.md), erro tipado exibido por `kind` (network/not_found/github_api/too_large/validation), sucesso exibe contagens finais; prop `allowUpload` (default false) já presente mas sem botão nesta story
- [X] T011 [US1] Compor `SyncPanel` (allowUpload=false) nas áreas Horizon e SRC — frontend/src/layouts/Dashboard.astro e frontend/src/layouts/SrcDashboard.astro — de modo que o SRC ofereça download por URL e NENHUM envio (FR-005: "a operação não é oferecida"); confirmar que nenhuma página Astro nova foi criada e o teste de contagem de ROUTES em src-tauri/src/lib.rs segue passando (21 rotas)

**Checkpoint**: US1 funcional e testável isoladamente — download público/privado → import com contagens corretas; MVP demonstrável (parar e validar aqui).

---

## Phase 4: User Story 2 — Publicar o export do Horizon no GitHub (Priority: P2)

**Goal**: Curador configura destino (repo/branch/caminho) por projeto, clica enviar e o zip canônico do Horizon é gerado NO ATO e gravado como commit na branch — com aviso genérico e sobrescrita sempre explícita. SRC recusado por política.

**Independent Test**: Com destino válido (mock wiremock; GitHub real no quickstart), enviar e conferir que o arquivo na branch é byte a byte igual ao export local; tentar enviar pelo SRC e ver a recusa `sync_policy`.

### Tests for User Story 2 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T012 [P] [US2] TDD: testes (red) da política pura em src-tauri/src/sync_domain.rs — `check_upload_allowed("src")` → `Err(SyncPolicy)` SEMPRE (mesmo com destino privado; FR-005), `check_upload_allowed("horizon")` → `Ok(())`; depois implementar
- [X] T013 [US2] TDD: testes wiremock (red) do fluxo Git Data API em src-tauri/tests/sync_github.rs seguindo contracts/github-api.md — caminho feliz `POST git/blobs` → `GET git/ref/heads/{branch}` → `POST git/trees` → `POST git/commits` → `PATCH git/refs/heads/{branch}` com assert de ORDEM das chamadas e de que NUNCA há `POST /git/refs` (Q1: nunca cria branch); ref 404 → `not_found` "branch não existe; crie-a no GitHub"; PATCH 403 → `github_api` "sem permissão de escrita; nada foi gravado"; PATCH 422 → reexecuta bloco uma vez e, persistindo, `github_api`; arquivo > 100 MB → `too_large` ANTES de qualquer chamada a blobs (Q3); eventos de fase `committing`
- [X] T014 [US2] TDD: testes de integração (red) de `github_check_destination` em src-tauri/tests/sync_github.rs — retorna `{repo,branch,path,branch_exists,file_exists,file_sha}` com flags false quando ausentes (não erro), repo 404 → `not_found` "repositório não encontrado ou sem acesso", sem destino configurado → `sync_config`; depois implementar em src-tauri/src/commands.rs (contracts/ipc-commands.md — versão corrigida)
- [X] T015 [US2] TDD: testes de integração (red) de `github_upload` em src-tauri/tests/sync_github.rs — `project=src` → `sync_policy` sem NENHUMA chamada HTTP; horizon feliz retorna `{commit_sha, html_url, replaced}` e os bytes enviados são IDÊNTICOS aos do export manual do mesmo estado (gerar no ato via fluxo de export existente — Q2/SC-003); destino existente com `file_sha` diferente e `confirm_overwrite=false` → `conflict` (US2-2); depois implementar em src-tauri/src/commands.rs e registrar em lib.rs

### Implementation for User Story 2

- [X] T016 [P] [US2] Adicionar rotas `GET /github/check` e `POST /github/upload` em frontend/src/services/api.js (antes dos genéricos) + testes unitários (red primeiro) em frontend/tests/unit/api.test.js
- [X] T017 [US2] TDD: testes Vitest (red) em frontend/tests/unit/SyncPanel.test.jsx e depois implementar a seção de ENVIO em frontend/src/components/SyncPanel.jsx (renderiza só com `allowUpload=true`): formulário de destino (repo/branch/path → POST `/github/config`) com validação `owner/name`, aviso genérico de dados pessoais exibido antes de enviar (FR-005 — Option B), confirmação explícita de sobrescrita quando `github_check` traz `file_exists` com conteúdo divergente, erro `conflict`/`not_found`/`sync_policy`/`too_large`/`github_api` mapeados por kind, sucesso exibe link do commit; manter o SRC sem este bloco (US2-4)

**Checkpoint**: US1 + US2 independentes e funcionais — ciclo completo de backup do Horizon.

---

## Phase 5: User Story 3 — Gerenciar o token de acesso localmente (Priority: P3)

**Goal**: Curador salva, testa e remove o token; nada dele aparece em tela (só `token_hint`), mensagens ou arquivos após remoção.

**Independent Test**: Salvar token → download privado passa a funcionar; testar → confirma login/escopos sem mostrar o valor; remover → acesso privado volta a falhar com a mensagem adequada e `sync_config.json` fica sem vestígio.

### Tests for User Story 3 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T018 [US3] TDD: testes wiremock (red) em src-tauri/tests/sync_github.rs para `github_save_token`, `github_remove_token` e `github_test_token` — save grava no config com permissão restrita; remove apaga a chave e inspeção do arquivo confirma vestígio ZERO (SC-006); test consulta `GET /user` e retorna `{login, scopes, valid:true}` sem ecoar o token; 401 → `github_api` "token inválido ou revogado"; sem token salvo → `sync_config`; depois implementar os três em src-tauri/src/commands.rs e registrar em lib.rs

### Implementation for User Story 3

- [X] T019 [P] [US3] Adicionar rotas `POST /github/token`, `DELETE /github/token` e `POST /github/token/test` em frontend/src/services/api.js (antes dos genéricos) + testes unitários (red primeiro) em frontend/tests/unit/api.test.js
- [X] T020 [US3] TDD: testes Vitest (red) em frontend/tests/unit/TokenDialog.test.jsx e depois implementar frontend/src/components/TokenDialog.jsx — input de senha para colar o token, "testar" mostra login/escopos vindos do backend, "remover" com confirmação; em NENHUM estado o valor completo aparece (só `token_hint` de `github_get_config`); estados de erro tipados exibidos
- [X] T021 [US3] Ligar `TokenDialog` ao `SyncPanel` nas duas áreas (botão "Token de acesso…" em frontend/src/components/SyncPanel.jsx abrindo o diálogo); fluxo visível: sem token → erro de acesso autenticado aponta para o diálogo (US1-3/US3 independent test)

**Checkpoint**: as três stories funcionam independentemente.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Auditoria de segurança, regressão e validação final

- [X] T022 [P] [US3] TDD: teste de auditoria de vazamento (red) em src-tauri/tests/sync_github.rs — iterar por todos os erros produzidos nos cenários das stories (network/github_api/too_large/sync_config/sync_policy/not_found/conflict) e afirmar que NENHUM contém o valor do token nem CPF/e-mail/nome de participante (SC-005); depois corrigir qualquer vazamento encontrado
- [X] T023 Rodar a verificação completa de regressão: `cargo test` (todos verdes, incluindo os 130 pré-existentes — SC-004), `npm test`, `npm run build` (0 erros, 21 páginas) e teste de contagem de ROUTES em src-tauri/src/lib.rs
- [X] T024 Executar em desktop os 12 cenários manuais de specs/034-github-sync/quickstart.md (GitHub real + offline) com atenção ao SC-001 (50 MB < 60 s) e às verificações de segurança da seção 3; registrar os resultados no próprio quickstart.md; atualizar frontend/tests/e2e/app.e2e.js somente se a navegação tiver mudado (esperado: não — 0 páginas novas)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (T001–T002)**: imediato; bloqueia tudo
- **Foundational (T003–T005)**: depende do Setup; BLOQUEIA todas as stories
- **US1 (T006–T011)**: depende da Foundational — sem dependência de outras stories
- **US2 (T012–T017)**: depende da Foundational; integra com o `SyncPanel` da US1 (T010/T011) na UI, mas a lógica (T012–T015) é independente
- **US3 (T018–T021)**: depende da Foundational (T004 fornece o armazenamento); a UI integra com SyncPanel (T010)
- **Polish (T022–T024)**: depois das stories desejadas

### User Story Dependencies

- **US1 (P1)**: totalmente independente após Foundational (token pré-gravado por seed de teste — T008)
- **US2 (P2)**: lógica própria; só a composição de UI reaproveita o SyncPanel
- **US3 (P3)**: comandos próprios; o USO do token pelo download/envio já foi testado nas US1/US2 via seed — aqui é a gestão

### Within Each User Story

- Testes (red) ANTES da implementação (Constituição I)
- Domínio puro (`sync_domain.rs`) antes do cliente HTTP (`sync_github.rs`) antes dos comandos (`commands.rs`) antes das rotas (`api.js`) antes dos componentes

### Parallel Opportunities

- Foundational: T003 ∥ T004 (arquivos diferentes)
- US1: T006 ∥ T009; US2: T012 ∥ T016; US3: T019 ∥ T020 (após seus respectivos testes de backend)
- Stories diferentes podem ser implementadas em paralelo após a Foundational (equipe separada), respeitando o lock do SyncPanel (T010 → T017 → T021)

---

## Parallel Example: User Story 1

```bash
# Testes de domínio e rota front-end em paralelo (arquivos distintos):
Task: "T006 [P] [US1] normalização de URL em src-tauri/src/sync_domain.rs"
Task: "T009 [P] [US1] rota POST /github/download em frontend/src/services/api.js"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 Setup (T001–T002)
2. Phase 2 Foundational (T003–T005) — CRITICAL
3. Phase 3 US1 (T006–T011)
4. **STOP and VALIDATE**: download+import pela URL funcionando com contagens idênticas (quickstart cenários 1–5)
5. MVP entrega valor sozinho (restaurar base sem pen drive)

### Incremental Delivery

1. Setup + Foundational → fundação pronta
2. US1 → validar → MVP
3. US2 → validar → backup completo do Horizon (envio segue bloqueado no SRC por política — FR-005)
4. US3 → validar → gestão de token completa
5. Polish → auditoria + regressão + quickstart manual

### Notes

- Toda task segue red→green: a task de teste lista os cenários e a de implementação referencia o contrato exato
- Commit após cada task ou grupo lógico (instrução fixa do usuário)
- Nenhuma task depende de rede real — wiremock injetável (R8); GitHub real só no quickstart (T024)

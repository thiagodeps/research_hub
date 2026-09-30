# Implementation Plan: Sincronização de exports com repositório GitHub

**Branch**: `034-github-sync` | **Date**: 2026-09-29 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/034-github-sync/spec.md`

## Summary

Adicionar sincronização de exports com o GitHub ao núcleo Rust: **download**
de um export hospedado em repositório (público ou privado) seguido do import
já existente (Horizon zip ou consolidado SRC, com as mesmas validações e
snapshot), e **envio** do zip canônico do Horizon como commit direto na
branch informada (por projeto; SRC bloqueado por política — FR-005). Um token
de acesso opcional, salvo apenas localmente, cobre repositórios privados e o
envio. Todo o acesso à rede vive no processo Rust; o front-end apenas invoca
comandos via `api.js` e exibe progresso.

## Technical Context

**Language/Version**: Rust 1.77.2 (edition 2021) no núcleo Tauri 2.11.3; JavaScript (Astro + React) no front-end estático embarcado.

**Primary Dependencies**: Existentes — `rusqlite` (bundled), `serde`/`serde_json` (preserve_order), `thiserror`, `zip`, `bytes`. **Novas** — `reqwest` (TLS via `rustls`, streaming de corpo; sem OpenSSL para compilar em Linux e Windows) e, em dev-dependencies, `wiremock` (servidor HTTP de mentira para testes de integração sem rede real).

**Storage**: SQLite em arquivo único (inalterado — nenhuma migração nova). A configuração de sincronização (token + destino por projeto) vai em um **arquivo JSON** no diretório app-data (`sync_config.json`), com permissão restrita ao usuário (0600 em Unix / ACL de perfil em Windows).

**Testing**: `cargo test` (unitários em `sync_domain.rs` sem Tauri + integração contra `wiremock`), `npm test` (Vitest para os novos componentes), smoke e2e com WebdriverIO/tauri-driver. Validação com GitHub real fica no quickstart manual (nenhum teste automatizado depende de rede externa — Princípio III).

**Target Platform**: Linux e Windows (desktop, Tauri 2.0).

**Project Type**: desktop-app (processo único; IPC do Tauri; sem camada HTTP local entre front-end e núcleo).

**Performance Goals**: download de 50 MB + import concluem em < 60 s (SC-001); corpo baixado em streaming (sem carregar 100 MB em memória); upload suporta até 100 MB (SC-007).

**Constraints**: atomicidade tudo-ou-nada na base local (FR-007); token nunca em mensagens/logs/diagnósticos (FR-006, SC-005); branch de destino deve existir — nunca criada automaticamente (Q1); export gerado no ato do envio (Q2); limite de 100 MB verificado antes de transferir (Q3); offline, apenas os recursos de sincronização ficam indisponíveis (Princípio V).

**Scale/Scope**: curador único; 2 projetos (Horizon, SRC); ~8 comandos IPC novos; 0 páginas Astro novas (UI embutida nas áreas existentes); 0 migrações de banco.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Status | Evidência / Justificativa |
|-----------|--------|---------------------------|
| I. TDD Obrigatório | ✅ PASS | Ordem de tasks será teste-primeiro: parsing de URL/payload e política no domínio puro, cliente HTTP contra `wiremock`, componentes com Vitest. |
| II. Paridade Funcional | ✅ PASS | A sincronização transporta os MESMOS bytes dos fluxos manuais; nenhum formato, validação ou saída do ciclo importar→editar→exportar muda (FR-001, SC-004). O oráculo Python não é afetado. |
| III. Testes Rigorosos (Rust e Front-end) | ✅ PASS | Domínio síncrono testado sem Tauri; integração HTTP contra servidor mock local (nunca rede real em CI); Vitest para a camada de comunicação e componentes; smoke e2e estendido. |
| IV. CRUD Completo e Operações Diretas | ✅ PASS | A entidade "Configuração de sincronização" tem CRUD completo: ler/salvar/remover destino por projeto, salvar/testar/remover token — sem fluxo de aprovação. |
| V. Aplicação Desktop Local e Autocontida | ✅ PASS (com nota) | A proibição é de camada HTTP **entre front-end e lógica de negócio** — mantida: só IPC. As chamadas HTTPS para github.com partem do núcleo Rust, iniciadas explicitamente pelo usuário, e são opcionais: sem rede, o aplicativo funciona integralmente (Princípio V) e a sincronização falha com erro tipado (FR-008). Nada do Escopo Diferido é reintroduzido (sem auto-update, sem novo motor de dados). |
| VI. Lógica de Negócio em Rust | ✅ PASS | Parsing de URL, política de privacidade, fluxo de commit Git Data API, validação de limites e leitura/escrita do arquivo de config: tudo em Rust. Front-end só renderiza e invoca via `api.js` (ponto único de acoplamento). |

**Re-check pós-Phase 1**: os contratos em `contracts/` mantêm toda a lógica no núcleo e a fronteira IPC concentrada em `api.js` — gates seguem PASS sem violações a justificar (Complexity Tracking vazio).

## Project Structure

### Documentation (this feature)

```text
specs/034-github-sync/
├── plan.md              # Este arquivo
├── research.md          # Fase 0 — decisões técnicas
├── data-model.md        # Fase 1 — entidades e regras
├── quickstart.md        # Fase 1 — roteiro de validação
├── contracts/
│   ├── ipc-commands.md  # Comandos Tauri novos (payloads/erros)
│   ├── github-api.md    # Interações com a API do GitHub (endpoints/ordem)
│   └── frontend-routes.md # Rotas api.js e eventos de progresso
└── tasks.md             # Fase 2 (/speckit.tasks — não gerado aqui)
```

### Source Code (repository root)

```text
src-tauri/src/
├── sync_domain.rs       # NOVO — lógica pura testável sem Tauri: parsing de URL
│                        #   do GitHub, política de privacidade (FR-005), limites
│                        #   de tamanho, validação de config, schema do JSON local
├── sync_github.rs       # NOVO — cliente GitHub (reqwest): download (Contents API
│                        #   raw / release asset) e upload (Git Data API), eventos
│                        #   de progresso
├── commands.rs          # + ~8 comandos github_* (async)
├── error.rs             # + variantes tipadas de rede/sincronização
├── lib.rs               # registro de módulos + invoke_handler
└── (import.rs / src_domain.rs / crud.rs / registry.rs — inalterados)

src-tauri/tests/
├── sync_github.rs       # NOVO — integração contra wiremock (download, upload,
│                        #   branch inexistente, 404, sem permissão, >100 MB)
└── fixtures/            # (inalterado)

frontend/src/
├── components/
│   ├── SyncPanel.jsx    # NOVO — download por URL + enviar para o GitHub
│   └── TokenDialog.jsx  # NOVO — salvar/testar/remover token (nunca ecoa o valor)
├── services/api.js      # + rotas /github/* (antes dos padrões genéricos)
└── (páginas/layouts existentes recebem os componentes — 0 páginas novas)
```

**Structure Decision**: dois módulos Rust novos separam lógica pura (`sync_domain.rs`, testável sem rede nem Tauri, espelhando o padrão de `src_domain.rs`) do cliente HTTP (`sync_github.rs`); comandos ficam em `commands.rs` como nos padrões `src_*`; a UI é composta dentro das áreas Horizon/SRC existentes para não inflar ROUTES (permanece 21).

## Complexity Tracking

> Nenhuma violação de constituição a justificar — tabela vazia.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |

# Contrato Front-end: rotas `api.js` e eventos de progresso

**Feature**: `034-github-sync` | **Date**: 2026-09-29

## Rotas novas em `frontend/src/services/api.js`

Registradas no array `ROUTES` **antes** dos padrões genéricos `/:entity`
(mesma regra da seção `/src` da SEP-033). `apiFetch` continua o único ponto
de chamada ao núcleo.

| Padrão | Verbo | Comando Tauri |
|--------|-------|---------------|
| `/github/config` | GET (`?project=`) | `github_get_config` |
| `/github/config` | POST | `github_set_config` |
| `/github/token` | POST (`{ token }`) | `github_save_token` |
| `/github/token/test` | POST | `github_test_token` |
| `/github/token` | DELETE | `github_remove_token` |
| `/github/download` | POST (`{ project, url }`) | `github_download` |
| `/github/check` | GET (`?project=`) | `github_check_destination` |
| `/github/upload` | POST (`{ project, confirmOverwrite }`) | `github_upload` |

Sem novas páginas Astro: **ROUTES de página permanecem 21** (teste de contagem
em `lib.rs` inalterado). Os componentes novos são compostos nas áreas
existentes (R9).

## Evento de progresso

- Canal: `sync://progress` (emitido pelo núcleo via `AppHandle.emit`).
- Payload: `{ operation, project, phase, bytes_done, bytes_total, detail? }`
  (ver data-model).
- `api.js` expõe `onSyncProgress(cb)` encapsulando `listen()` de
  `@tauri-apps/api/event` — o resto do front-end nunca usa a API de eventos
  diretamente (fronteira única, Princípio VI).

## Componentes novos (Vitest)

| Componente | Responsabilidade | Cenários testados |
|------------|------------------|-------------------|
| `SyncPanel.jsx` | Download por URL + (só Horizon) botão "Enviar para o GitHub" com aviso genérico (FR-005) e confirmação de sobrescrita; desabilitado sem rede configurável | sucesso, erro tipado exibido, SRC sem botão de envio, sobrescrita exige confirmação |
| `TokenDialog.jsx` | Salvar/testar/remover token; nunca ecoa o valor (só `token_hint`) | salvar, testar (ok/401), remover, máscara do valor |

## Regressão garantida

- Nenhuma rota existente muda de posição semântica: `/github/*` entra como
  bloco novo antes dos genéricos; testes unitários existentes de `api.js`
  continuam verdes (SC-004).

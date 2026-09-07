# Implementation Plan: Shell Desktop Tauri 2.0

**Branch**: `015-tauri-shell-bootstrap` | **Date**: 2026-09-02 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/015-tauri-shell-bootstrap/spec.md`

## Summary

Transformar o ResearchHub de aplicação web de dois processos em aplicação desktop de processo único, criando o andaime Tauri 2.0 que carrega o front-end Astro já existente. Nenhuma lógica de negócio é introduzida.

O trabalho é dominado por uma pergunta de risco — se as rotas do Astro resolvem sob o protocolo de asset do Tauri (risco 10.6 do estudo) — que foi respondida empiricamente antes de qualquer decisão de implementação. **A resposta eliminou a maior parte do trabalho previsto.**

## Technical Context

**Language/Version**: Rust 1.98.0 stable (MSRV declarado: 1.77.2, exigido pelo Tauri 2.0)
**Primary Dependencies**: `tauri` 2.11.3, `tauri-plugin-log` 2, `tauri-cli` 2.11.4
**Storage**: N/A nesta feature (SQLite chega na SEP-016)
**Testing**: `cargo test` com `tauri::test::mock_builder`; Vitest permanece intocado
**Target Platform**: Linux e Windows (macOS é escopo diferido — Constitution §Escopo Diferido)
**Project Type**: Aplicação desktop (núcleo Rust + front-end web embarcado)
**Performance Goals**: Janela aberta em menos de 3 s (SC-006)
**Constraints**: Zero componentes React modificados (SC-005); nenhuma porta de rede aberta; funciona offline
**Scale/Scope**: 18 rotas de navegação, 1 janela, 0 comandos IPC

## Constitution Check

*GATE: verificado antes da Fase 0 e revalidado após a implementação.*

- [x] **I — TDD Adherence**: o spike de roteamento foi escrito como verificação executável antes de qualquer decisão de configuração, e foi convertido em três testes `cargo test` que hoje guardam o comportamento.
- [x] **II — Functional Parity**: nenhuma funcionalidade nova. A única mudança de comportamento é a correção da sidebar (FR-011), declarada como escopo explícito na spec.
- [x] **III — Test Strategy**: `cargo test` cobre as 18 rotas automaticamente. E2E com WebdriverIO chega na SEP-022, conforme planejado; nesta feature nada é alvo de release.
- [x] **IV — Operations**: não aplicável (sem CRUD nesta feature).
- [x] **V — Desktop Architecture**: processo único, Astro embarcado, sem camada HTTP, sem porta de rede, funciona offline.
- [x] **VI — Business Logic in Rust**: nenhuma regra de negócio existe ainda; nenhum SQL ou I/O de arquivo foi adicionado ao JavaScript.
- [x] **Deferred Scope**: alvos de bundle limitados a `deb`, `rpm`, `appimage`, `nsis`. Sem macOS, sem assinatura, sem updater.

## Project Structure

### Documentation (this feature)

```text
specs/015-tauri-shell-bootstrap/
├── spec.md              # Especificação
├── plan.md              # Este arquivo
└── tasks.md             # Saída de /speckit.tasks — ainda não criado
```

### Source Code (repository root)

```text
src-tauri/                    # NOVO — processo Rust
├── Cargo.toml                # crate research-hub; feature custom-protocol
├── build.rs
├── tauri.conf.json           # identidade, janela, CSP, alvos de bundle
├── capabilities/default.json # apenas core:default (decisão Q1)
├── icons/                    # padrão do Tauri; branding fica para a SEP-023
└── src/
    ├── main.rs               # chama research_hub_lib::run()
    └── lib.rs                # builder + ROUTES + testes de roteamento

frontend/                     # PRESERVADO
├── src/components/           # 7 componentes React — INTOCADOS
├── src/pages/                # 18 páginas — INTOCADAS
├── src/layouts/Dashboard.astro  # ÚNICA alteração: correção da sidebar
└── src/services/api.js       # intocado (shim chega na SEP-017)

backend/                      # CONGELADO — oráculo de paridade até a SEP-024
```

**Structure Decision**: o front-end permanece no lugar e é consumido por `frontendDist: "../frontend/dist"`. Não há movimentação de arquivos. O `src-tauri/` nasce como diretório irmão de `frontend/` e `backend/`, seguindo a convenção do Tauri.

## Fase 0 — Spike de Roteamento (concluída)

### A pergunta

O Astro gera rotas como diretórios com `index.html` (`dist/dashboard/groups/index.html`), enquanto o menu lateral aponta para `/dashboard/groups`, sem barra final e sem extensão. Um servidor HTTP resolve isso; o protocolo de asset do Tauri poderia não resolver.

### Como foi respondida

Em vez de clicar 16 itens de menu e julgar pelo resultado visual, o spike consultou **o mesmo resolvedor que o protocolo usa** (`AppHandle::asset_resolver()`), testando cada rota em quatro formas: nua, com barra final, com `/index.html` explícito e com sufixo `.html`. O método importa: um teste visual diria *se* funciona; este diz *quais formas de URL* funcionam, que é o que determina a estratégia.

O spike exigiu build em modo de produção — em modo dev o `generate_context!` não embute os assets, apontando para o servidor de desenvolvimento.

### Resultado

```
RESUMO (18 rotas):
  href nu       (/dashboard/groups)   -> 18/18 resolvem
  href c/ barra (/dashboard/groups/)  -> 18/18 resolvem
```

**As quatro formas resolvem, para todas as 18 rotas.** O resolvedor do Tauri normaliza o caminho e acrescenta `index.html` quando necessário.

### Decisão

**Nenhuma mudança de roteamento.** Os hrefs do front-end ficam exatamente como estão.

As três mitigações previstas no estudo (risco 10.6) foram todas descartadas, e é importante registrar por quê — todas eram desnecessárias, e duas seriam ativamente prejudiciais:

| Mitigação prevista | Por que foi descartada |
|---|---|
| `trailingSlash: 'always'` + barra nos hrefs | Desnecessária. Exigiria alterar `LoginForm.jsx` e `EntityForm.jsx`, **violando FR-012 e SC-005** |
| `build.format: 'file'` + `.html` nos hrefs | Desnecessária. Mesmo problema: tocaria dois componentes React |
| Handler de protocolo customizado em Rust | Desnecessária. Seria código nosso reimplementando o que o framework já faz |

Vale destacar o achado que só apareceu ao inventariar os links: **duas das três estratégias previstas eram incompatíveis com a própria spec.** `LoginForm.jsx` faz `window.location.href = '/dashboard'` e `EntityForm.jsx` monta `` `/dashboard/${route}?openId=${item.id}` `` — ambos componentes React, que FR-012 proíbe alterar. Se o spike tivesse sido pulado e uma das mitigações adotada por precaução, a feature teria quebrado o próprio critério de aceite.

### Regressão

O spike virou teste permanente em `src-tauri/src/lib.rs`:

- `todas_as_rotas_do_menu_resolvem_com_o_href_atual` — as 18 rotas, com o href exato do front-end
- `rotas_com_query_string_resolvem` — deep link `?openId=N`
- `contagem_de_rotas_bate_com_o_menu_lateral` — denuncia página nova não registrada

Para que `cargo test` exercite o mesmo caminho do binário de produção, a feature `custom-protocol` é habilitada via `[dev-dependencies]`. Sem isso os testes rodariam contra um contexto sem assets e passariam ou falhariam por motivo errado.

## Fase 1 — Configuração

| Item | Valor | Origem |
|---|---|---|
| `identifier` | `br.edu.ifes.researchhub` | substitui o `com.tauri.dev` do template |
| Janela | 1440×900, mínimo 1024×700, `dragDropEnabled: true` | FR-003 |
| CSP | `default-src 'self'` + `img-src 'self' asset: data:` + `style-src 'self' 'unsafe-inline'` | FR-005 |
| Capabilities | apenas `core:default` | FR-004 / decisão Q1 |
| Alvos de bundle | `deb`, `rpm`, `appimage`, `nsis` | Q2 |
| Crate | `research-hub`, lib `research_hub_lib` | — |

`style-src 'unsafe-inline'` é exigido pelos estilos inline que o Astro e o Tailwind emitem. É a única concessão da política.

### Armadilha encontrada

O `beforeBuildCommand` gerado pelo template usava `npm --prefix ../frontend`, e o Tauri executa esse comando **a partir da raiz do projeto**, não de `src-tauri/`. O caminho resolvia para fora do repositório e o build falhava. Corrigido para `npm --prefix frontend`. Vale para `beforeDevCommand` também.

## Fase 2 — Correção da Sidebar (FR-011)

`Dashboard.astro` misturava `<a>` soltos com `<li>` fora de qualquer `<ul>`, e a partir de "Grupos de Pesquisa" usava classes de tema claro (`text-slate-700 hover:bg-slate-100`) sobre o painel `bg-slate-900` — 13 dos 16 itens ficavam praticamente invisíveis.

Reescrito como `<ul>` com 16 `<li>`, classes uniformes (`text-slate-200 hover:bg-slate-800 hover:text-white`) e o divisor de seção como item próprio. Os hrefs não mudaram — o spike confirmou que não precisam.

## Complexity Tracking

Nenhuma violação da Constitution a justificar. A feature reduziu complexidade em relação ao previsto: três mitigações de roteamento foram descartadas e nenhum código de contorno foi escrito.

## Estado de Verificação

| Critério | Estado |
|---|---|
| SC-002 — 18/18 rotas resolvem no binário compilado | ✅ automatizado em `cargo test` |
| SC-005 — zero componentes React modificados | ✅ `git diff --stat frontend/src/components/` vazio |
| SC-001 / SC-003 / SC-004 / SC-006 / SC-007 | ⏳ pendente de verificação de execução |

## Próximo Passo

`/speckit.tasks` para gerar `tasks.md`, ou seguir direto para a SEP-016 (núcleo de persistência).

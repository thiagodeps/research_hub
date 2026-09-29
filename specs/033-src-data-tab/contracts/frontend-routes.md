# Contrato: Rotas do front-end (`api.js`) e navegação

**Regra do repositório**: `frontend/src/services/api.js` é o único ponto de acoplamento JS→Rust. Nenhum componente chama `invoke` diretamente (DataControlCenter hoje é a exceção histórica de drag-drop; componentes novos seguem `api.js`).

## Novas rotas em `api.js` (mapeadas para os comandos de `contracts/ipc-commands.md`)

| Método+Rota | Comando | Usado por |
|---|---|---|
| `POST /src/import` | `import_src_json` | SrcDataControl (o path vem do evento de diálogo; ver nota) |
| `POST /src/export` | `export_src_json` | SrcDataControl |
| `GET /src/meta` · `PUT /src/meta` | `src_get_meta` / `src_update_meta` | Cabeçalho da área SRC |
| `GET /src/acoes` (`?search=&sort=&order=&limit=&offset=`) | `src_list_acoes` | EntityTable da página Ações |
| `POST /src/acoes` | `src_create_acao` | EntityForm |
| `GET|PUT|DELETE /src/acoes/:id` | `src_get_acao` / `src_update_acao` / `src_delete_acao` | EntityForm / EntityTable |
| `GET /src/acoes/:id/participacoes` | `src_list_participacoes` | Editor de participações |
| `POST /src/acoes/:id/participacoes` | `src_create_participacao` | Editor de participações |
| `PUT|DELETE /src/participacoes/:id` | `src_update_participacao` / `src_delete_participacao` | Editor de participações |

**Padrão de ordem**: rotas mais específicas antes das genéricas (mesma precaução do `/link` atual). As rotas `/src/...` devem preceder o catch-all `/:entity` ou ser distingüíveis por prefixo — decisão de implementação anotada para o `/speckit.tasks`.

**Import por drag-drop**: o componente SRC pode reutilizar o padrão do `DataControlCenter` (drop → `runImport(file)`), chamando o comando dedicado por `apiFetch`.

## Páginas (Astro) e navegação

| Rota | Arquivo | Papel |
|---|---|---|
| `/` | `frontend/src/pages/index.astro` | **MODIFICAR**: autenticado → `/projects` (antes: `/dashboard`) |
| `/projects` | `frontend/src/pages/projects.astro` | **NOVA**: página principal pós-login — cartões Horizon e SRC (FR-001/002) |
| `/dashboard` (+15 entidades) | existentes | **INVÁRIAS** — área Horizon (FR-005) |
| `/src` | `frontend/src/pages/src/index.astro` | **NOVA**: área SRC — resumo da base (campus, totais), import/export do JSON, link para Ações |
| `/src/acoes` | `frontend/src/pages/src/acoes.astro` | **NOVA**: tabela de ações (busca/ordenação/paginação) + CRUD + editor de participações |

**Layouts**: novo `SrcDashboard.astro` (menu lateral: Resumo, Ações, Importar/Exportar, "Trocar projeto" → `/projects`). O `Dashboard.astro` existente ganha apenas o atalho "Trocar projeto". Nenhum item de menu Horizon é removido.

## Teste de rotas (`lib.rs::ROUTES`)

Fonte da verdade de navegação (hoje 18 rotas; `/register` tem teste próprio e fica fora do array). Atualização obrigatória junto com as páginas novas:

```text
"/" , "/login", "/dashboard", 15 entidades,        ← 18 (invárias)
"/projects", "/src", "/src/acoes",                  ← 3 novas
total: 21
```

O teste `contagem_de_rotas_bate_com_o_menu_lateral` e o de resolução de rotas cobrem a mudança (TDD: testes primeiro).

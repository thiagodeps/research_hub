# Contract: IPC — Listagem de Entidades (`list_entities`)

**Feature**: 030-search-by-id | **Date**: 2026-09-23

A superfície externa deste projeto é a fronteira IPC Tauri entre o front-end React e o
núcleo Rust (Constituição V/VI — não há HTTP). Este contrato descreve o único comando
tocado pela feature. **A assinatura não muda**; o que muda é a semântica do parâmetro
`search`.

## Comando Tauri: `list_entities`

**Arquivo**: `src-tauri/src/commands.rs` (assinatura intacta)

### Parâmetros (invocação)

| Nome | Tipo | Obrigatório | Descrição |
|---|---|---|---|
| `entity` | `string` | sim | Route do registry (ex.: `researchers`) |
| `limit` | `number \| null` | não | Tamanho da página (1..1000, default 100) |
| `offset` | `number \| null` | não | Deslocamento (≥ 0) |
| `search` | `string \| null` | não | **Termo livre** — ver semântica nova abaixo |
| `sort` | `string \| null` | não | Coluna válida do registry (inválida é ignorada) |
| `order` | `string \| null` | não | `asc` (default) ou `desc` |

### Retorno

```jsonc
// crud::Page — forma inalterada
{
  "items": [ { "id": 42, "name": "…", "…": "…" } ],  // colunas do registry
  "total": 7                                          // matches do WHERE completo
}
```

### Semântica de `search` (MUDANÇA desta feature)

| Termo | Comportamento |
|---|---|
| `null` / `""` / só espaços | Sem filtro — listagem integral (inalterado) |
| Somente dígitos (após trim) | União: `id = N` **ou** `{search_column} LIKE '%N%'`; registro `id = N` ordenado primeiro. Entidade sem `search_column`: somente `id = N` |
| Qualquer outro | `{search_column} LIKE '%termo%' ESCAPE '\'` — case-insensitive (ASCII), wildcards `%`/`_` literais — **inalterado** |
| Entidade sem `search_column` + termo textual | Filtro ignorado — listagem integral (inalterado) |

### Erros

| Caso | Resultado |
|---|---|
| `entity` fora do registry | `AppError::UnknownEntity` (inalterado — nunca monta SQL) |
| Termo numérico com overflow (> i64) | Tratado como termo textual (não é erro) |
| Falha de banco | `AppError` padrão serializado (inalterado) |

## Ponte front-end: `apiFetch('GET /{entity}?search=…')`

**Arquivo**: `frontend/src/services/api.js` (intacto)

- Rota `GET /{entity}` repassa `search`, `sort`, `order`, `limit`, `offset` como hoje
  (`q.get('search')` → `invoke('list_entities', { search })`).
- Nenhum tratamento especial de números no JS: a classificação é regra de negócio e
  permanece no núcleo Rust (Constituição VI).

## Compatibilidade

- Consumidores atuais (`EntityPage.jsx` e testes Vitest de `api.js`): **nenhuma mudança
  exigida**; resultados textuais existentes permanecem idênticos (FR-005, SC-002).
- O contrato é aditivo: casos que antes devolviam X continuam devolvendo X; termos
  puramente numéricos podem devolver X + o registro do ID.

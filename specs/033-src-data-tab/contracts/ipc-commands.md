# Contrato: Comandos IPC (núcleo Rust ⇄ front-end)

**Princípio VI (Constituição)**: a fronteira IPC é auditável em um só lugar. Comandos do Horizon ficam **invários**; todos os comandos abaixo são novos e dedicados ao domínio SRC. Erros atravessam a fronteira como `{ kind, message }` serializado (padrão `error.rs`) — nunca pânico.

## Comandos novos

### `import_src_json`

```text
Parâmetros : path: string          (caminho escolhido no diálogo aberto pelo Rust)
Eventos    : "src-import://progress"  →  { stage: string, detail?: string }
Retorno    : {
             total_acoes: number,
             total_participacoes: number,
             acoes_com_participacoes: number,
             replaced: boolean,          // false = base anterior vazia
             snapshot: string | null,    // caminho do snapshot criado (padrão import.rs)
           }
Erros      : Validation (arquivo não-JSON / estrutura incompatível / acao_id duplicado — base intacta)
             Io (leitura/escrita), Snapshot
Pré-condição: sessão aberta (require_session)
Efeitos    : valida → snapshot (se havia base) → substitui src_meta/src_acoes/src_participacoes → informa (FR-006..009)
```

### `export_src_json`

```text
Parâmetros : path: string          (destino do diálogo salvar, aberto pelo Rust)
Retorno    : { total_acoes: number, total_participacoes: number, path: string }
Erros      : Io, Validation (base vazia → confirmada como erro claro? Não: base vazia exporta arquivo com "acoes": [] — decidido: export de base vazia produz arquivo vazio válido)
Pré-condição: sessão aberta
Efeitos    : remonta o consolidado (contrato src-consolidated-json.md) e escreve com indent=2 (C9)
```

### CRUD de ações (dedicados — sincronizam projeção ↔ raw_json)

```text
src_list_acoes    (limit: number|null, offset: number|null,
                   search: string|null, sort: string|null, order: "asc"|"desc"|null)
                  → { total: number, rows: [{ id, acao_id, processo, titulo, natureza,
                                              tipo, coordenador, acao_vinculante,
                                              total_participacoes }] }
src_get_acao      (id: number) → { ação (todas as projeções + metadados) + resumo
                                 + participantes agregados }
src_create_acao   (payload: { acao_id, processo?, titulo?, natureza?, tipo?,
                              coordenador?, acao_vinculante?, campus? })
                  → { id }         // raw_json construído com as chaves canônicas
src_update_acao   (id: number, payload: idem create, campos parciais)
                  → null           // grava projeção E raw_json (chave de rótulo)
src_delete_acao   (id: number) → null
                  // Se outra ação referencia esta como "Ação vinculante": erro
                  // Conflict com a lista de filhas — o front confirma e chama
                  // src_delete_acao com force: true (desvincula com aviso, FR-013)
Erros      : Validation (acao_id vazio/duplicado), NotFound, Conflict
```

### CRUD de participações (aninhadas à ação)

```text
src_list_participacoes   (acaoId: number)
                         → { rows: [{ id, ord, tipo, atividade_num, atividade_id,
                                      atividade, nome }] }   // ordenado por ord
src_create_participacao  (acaoId: number, payload: { tipo, atividade_num?,
                         atividade_id?, atividade?, Nome?, CPF?, "E-mail"?, Situação?,
                         Função?, Vínculo?, extras?: Record<string,string> })
                         → { id }      // ord = max(ord)+1; raw_json com chaves canônicas
src_update_participacao  (id: number, payload: campos parciais) → null
src_delete_participacao  (id: number) → null
Erros      : Validation (tipo vazio), NotFound
```

Nota: os campos de pessoa preservam as chaves de rótulo do arquivo (`Nome`, `CPF`, `E-mail`, `Situação`, `Função`, `Vínculo`); `extras` carrega chaves adicionais (contrato C5). Metadados do consolidado: `src_get_meta()` → `{ campus }` / `src_update_meta(campus)`.

## Comandos do Horizon — INVÁRIOS

`login`, `register`, `logout`, `session_status`, `list_entities`, `get_entity`, `create_entity`, `update_entity`, `delete_entity`, `import_canonical_zip`, `export_canonical_zip`, `merge_entities`, `link_entities` — nenhuma assinatura, evento ou erro muda (FR-005).

## Registro (lib.rs)

Todos os comandos novos entram em `invoke_handler` + `commands.rs` (ou módulo `src_domain.rs` novo); `require_session` em todos.

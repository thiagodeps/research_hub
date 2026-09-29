# Data Model: Área de Dados do Projeto SRC com Seleção de Projeto

**Branch**: `033-src-data-tab` | **Data**: 2026-09-29 | **Spec**: [spec.md](./spec.md) | **Research**: [research.md](./research.md)

Visão geral dos dois domínios de dados no mesmo arquivo SQLite (Constituição V):

```text
┌─────────────────────────────────────────────┐
│ research_hub.sqlite (arquivo único)         │
│                                             │
│  DOMÍNIO HORIZON (intocado nesta feature)   │
│  ├── admins                                 │
│  └── researchers, students, articles, ...   │
│                                             │
│  DOMÍNIO SRC (novo — migration 002)         │
│  ├── src_meta          (metadados do arquivo)│
│  ├── src_acoes         (ações)              │
│  └── src_participacoes (pessoas por ação)   │
└─────────────────────────────────────────────┘
```

A separação de projetos (FR-004) é estrutural: tabelas prefixadas `src_`, seção própria no `registry.rs`, comandos IPC dedicados e rotas de front-end distintas. Nenhuma query do Horizon toca tabelas `src_*` e vice-versa.

## Entidades

### Projeto (contexto de trabalho)

Estado de sessão, não tabela. Valores: `horizon` | `src`. Reside na sessão do front-end (a página ativa determina qual conjunto de comandos/rotas é usado); o back-end não mantém "projeto atual" — cada comando é inequivocamente de um domínio, o que elimina vazamento por estado compartilhado.

### src_meta — metadados do consolidado

Um único registro (rowid 1), reescrito a cada importação.

| Campo | Tipo | Origem no JSON | Regras |
|---|---|---|---|
| `campus` | TEXT NULL | `campus` (raiz) | Exibido como metadado; editável |
| `imported_at` | TEXT NOT NULL | — | Timestamp do snapshot de importação (auditoria) |

Os contadores da raiz (`total_acoes`, `acoes_com_participacoes`, `total_atividades`, `total_publico_alvo`, `total_equipe`) **não são armazenados**: são derivados das linhas e recomputados no export (research R4). `campus` é o único dado de cabeçalho não derivável.

### src_acoes — ações de extensão/ensino

| Campo | Tipo | Origem no JSON | Regras |
|---|---|---|---|
| `id` | INTEGER PK AUTOINCREMENT | — | Identificador interno (nunca exportado) |
| `acao_id` | TEXT NOT NULL | `acao_id` | Único entre ações (validação na importação: duplicado aborta com mensagem clara); preservado na edição |
| `raw_json` | TEXT NOT NULL | objeto inteiro da ação | Objeto original íntegro — chaves de rótulo preservadas, campos desconhecidos incluídos (research R3). Nunca exportado como coluna; é o que vira o objeto no export |
| `processo` | TEXT NULL | `"Processo nº"` | Projeção: busca/ordenação/listagem |
| `titulo` | TEXT NULL | `"Título ação"` | Projeção; coluna de busca (search_column) |
| `natureza` | TEXT NULL | `"Natureza"` | Projeção |
| `tipo` | TEXT NULL | `"Tipo ação"` | Projeção |
| `coordenador` | TEXT NULL | `"Coordenador(a)"` | Projeção |
| `acao_vinculante` | TEXT NULL | `"Ação vinculante"` | Vínculo textual ao programa guarda-chuva; sem FK (referência por rótulo na fonte) |
| `campus` | TEXT NULL | `"Campus"` / `campus` | Projeção informativa |
| `total_participacoes` | INTEGER NOT NULL DEFAULT 0 | `total_participacoes` | Derivado; mantido em sincronia com as linhas filhas |

**Sincronização (regra Rust, Princípio VI)**: toda criação/edição de ação grava a projeção **e** o campo correspondente dentro de `raw_json` (chave de rótulo conhecida). Campos de rótulo fora das projeções continuam em `raw_json` e sobrevivem a qualquer round-trip. Campos desconhecidos nunca são descartados.

**Vínculo programa→filhas**: consultado por igualdade textual (`acao_vinculante` da filha ↔ processo/título do pai). Exclusão de ação referenciada como vinculante por outra dispara o aviso obrigatório (FR-013): o aplicativo impede ou desvincula com confirmação — nunca referência órfã silenciosa.

### src_participacoes — pessoas aninhadas à ação

| Campo | Tipo | Origem no JSON (por entrada) | Regras |
|---|---|---|---|
| `id` | INTEGER PK AUTOINCREMENT | — | Interno |
| `acao_row_id` | INTEGER NOT NULL FK→src_acoes(id) | — | `ON DELETE CASCADE` (excluir ação leva suas participações, com confirmação prévia) |
| `ord` | INTEGER NOT NULL | índice na lista | Preserva a ordem original; reordenável em edição |
| `tipo` | TEXT NOT NULL | `tipo` | Valores válidos: `Público-alvo`, `Equipe de execução` (import valida; outro valor entra como veio — o arquivo é o oráculo — e é sinalizado) |
| `atividade_num` | TEXT NULL | `atividade_num` | Contexto da atividade de origem |
| `atividade_id` | TEXT NULL | `atividade_id` | Preservado (research R2/Assumption) |
| `atividade` | TEXT NULL | `atividade` | Nome da atividade |
| `nome` | TEXT NULL | `"Nome"` | Projeção para busca/listagem |
| `raw_json` | TEXT NOT NULL | objeto inteiro da entrada | Mesma política da ação: chaves de rótulo preservadas (CPF, E-mail, Situação, Função, Vínculo, ...) |

**Contém PII localmente** — mesma natureza do arquivo de origem; o aplicativo é offline, perfil único de admin e nada publica (spec, Assumptions).

### Horizon (contexto — intocado)

`admins` + as 15 tabelas canônicas definidas em `registry.rs::ENTITIES`. Nenhuma alteração de esquema, comando ou comportamento (FR-005; migration 002 adiciona apenas tabelas `src_*`).

## Validações (importação — todas atômicas)

1. Arquivo é JSON válido → senão aborta com mensagem clara, base intacta (FR-007).
2. Raiz é objeto com `acoes` sendo lista → senão aborta ("não parece um consolidado do SRC").
3. Cada item de `acoes` é objeto com `acao_id` não vazio → senão aborta indicando a posição do problema.
4. `participacoes`, quando presente, é lista → senão aborta.
5. `acao_id` duplicado → aborta (a substituição inteira é desfeita antes de qualquer escrita).
6. `acoes: []` (base vazia) → importação **válida**: base SRC fica vazia e o resultado informa isso (edge case da spec).
7. Só depois de toda a validação: snapshot (se havia base) + substituição (FR-006).

## Exportação (remontagem)

- Raiz: `campus` (de `src_meta`), contadores recomputados (R4), `acoes` em ordem estável (ordenação por `id` interno = ordem do arquivo importado; novas ações entram ao final).
- Cada ação: emitida a partir de `raw_json` (ordem de chaves preservada), com `total_participacoes` e `participacoes` atualizados; participações em ordem de `ord`.
- Participações: emitidas a partir de `raw_json`; entradas criadas na curadoria recebem as chaves canônicas do formato (`atividade_num`, `atividade_id`, `atividade`, `tipo` + campos da pessoa conforme o formulário).
- Serialização idêntica à do `src-etl-consolidate`: `indent=2`, sem escape de não-ASCII, chaves na ordem original (`preserve_order`).

## Transições de estado

| Estado | Evento | Resultado |
|---|---|---|
| Base SRC vazia (pós-migration) | Import válido | Base substituída; contagem informada |
| Base carregada | Import válido | Snapshot `src-<timestamp>` + substituição (nunca mescla) |
| Base carregada | Import inválido | Abort; base e snapshot anteriores intactos |
| Base carregada | CRUD de ações/participações | Linhas + `raw_json` + projeções sincronizadas; contadores derivados atualizados |
| Base carregada | Export | Arquivo fiel ao conteúdo corrente; base inalterada |
| Sessão | Seleção/troca de projeto | Front-end troca de área; nenhum dado do outro projeto é renderizado (FR-004) |

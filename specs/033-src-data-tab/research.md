# Research: Área de Dados do Projeto SRC com Seleção de Projeto

**Branch**: `033-src-data-tab` | **Data**: 2026-09-29 | **Spec**: [spec.md](./spec.md)

## R1 — Como separar os dados dos dois projetos no mesmo armazenamento

**Decision**: tabelas dedicadas com prefixo `src_` (`src_meta`, `src_acoes`, `src_participacoes`) no mesmo arquivo SQLite, com metadados das entidades SRC em uma seção própria do `registry.rs`. Nenhuma tabela do Horizon é tocada; nenhum comando genérico de CRUD do Horizon enxerga rotas SRC.

**Rationale**: a Constituição (Princípio V) exige SQLite em arquivo único — um banco por projeto fragmentaria o estado, os snapshots e a experiência. Tabelas dedicadas tornam a mistura impossível no nível do esquema (FR-004/SC-005 sem depender de disciplina de código) e preservam o Horizon intocado (FR-005). O `registry.rs` é, por decisão de arquitetura do repositório (AGENTS.md), o único lugar para metadados de entidade — a seção SRC vive lá, não em um registro paralelo.

**Alternatives considered**:
- *Coluna `project` em tabelas compartilhadas*: obrigaria todas as queries do Horizon a filtrar por projeto; risco permanente de vazamento entre bases; contamina o registry existente.
- *Um arquivo de banco por projeto*: quebra o modelo de estado atual (`AppState` guarda uma conexão), duplica a rotina de snapshots e cria dois "aplicativos" num binário só.

## R2 — Formato real do consolidado do SRC (evidência de primeira mão)

**Decision**: tratar como contrato de entrada/saída o formato produzido por `src-etl-consolidate` (`/home/thiagodeps/src/src/src_etl/etl/consolidar.py`), não o diagrama ER do README.

**Rationale**: a leitura do código-fonte do SRC_ETL mostra que o consolidado NÃO é "ações com atividades aninhadas e contagens":

```json
{
  "campus": "Serra",
  "total_acoes": N, "acoes_com_participacoes": N,
  "total_atividades": N, "total_publico_alvo": N, "total_equipe": N,
  "acoes": [
    { "acao_id": "...", "Processo nº": "...", "Título ação": "...", ...
      "total_participacoes": K,
      "participacoes": [
        { "atividade_num": "...", "atividade_id": "...", "atividade": "...",
          "tipo": "Público-alvo" | "Equipe de execução",
          "Nome": "...", "CPF": "...", "E-mail": "...", "Situação": "...", ... }
      ] }
  ]
}
```

- As `participacoes` são **achatadas: uma entrada por pessoa**, com o contexto da atividade de origem (`atividade_num`, `atividade_id`, `atividade`) e o campo `tipo` ("Público-alvo" ou "Equipe de execução"). Atividades não existem como objetos no consolidado.
- As chaves dos campos vêm dos rótulos da página do SRC (`"Processo nº"`, `"Título ação"`, `"Nome"`, `"CPF"`...), não de snake_case — o `Acao` Pydantic tem `extra: "allow"` e o consolidado embute o dict bruto raspado (`dict(a)`).
- **O consolidado contém PII** (CPF/e-mail dos alunos) e o próprio projeto SRC o marca como local ("Mantenha local — não commitar nem publicar"). As contagens sem PII são característica do painel público, não do arquivo que o Research Hub editará.

**Consequência (correção declarada na spec)**: a spec descrevia "público-alvo como contagens, sem dados pessoais" — correção aplicada: a entidade editável aninhada é a **Participação** (pessoa com tipo e contexto de atividade); o dado permanece **local**, coerente com o Princípio V (app desktop, offline, perfil único de admin). Correção registrada aqui e no plano — não é mudança silenciosa.

**Alternatives considered**: seguir o diagrama ER do README — rejeitado: o arquivo real é o oráculo do round-trip (SC-003) e é o que o curador efetivamente importará.

## R3 — Estratégia de armazenamento das ações: raw + projeções

**Decision**: cada ação é uma linha em `src_acoes` com (a) `raw_json` TEXT contendo o objeto original **íntegro** (chaves de rótulo preservadas, inclusive campos desconhecidos) e (b) colunas de projeção normalizadas para listagem/busca/ordenação/edição (`acao_id`, `processo`, `titulo`, `natureza`, `tipo`, `coordenador`). Cada participação é uma linha em `src_participacoes` com o mesmo padrão (`raw_json` próprio + projeções `tipo`, `atividade_num`, `atividade`, `nome`).

**Rationale**: garante SC-003 por construção — uma importação sem edição exporta os mesmos objetos, chave a chave, porque os objetos originais é que são emitidos de volta (serde_json com `preserve_order` mantém a ordem das chaves; `ensure_ascii=false` do Python equivale à serialização padrão do serde_json, que não escapa não-ASCII). Também absorve campos desconhecidos (`extra: "allow"` do Pydantic) sem mapeamento exaustivo. As projeções dão busca/ordenação/paginação em SQL e colunas estáveis para a UI. Toda escrita (create/update/delete de ação ou participação) sincroniza projeção e `raw_json` no lado Rust — regra de negócio no núcleo (Princípio VI).

**Alternatives considered**:
- *Decomposição total em colunas*: exige mapa completo rótulo↔coluna e quebra ao primeiro campo novo do scraper; o round-trip precisaria reconstruir ordem de chaves de memória.
- *JSON blob único por ação sem linhas de participação*: inviabiliza busca/paginação/CRUD direto das participações (Princípio IV).

## R4 — Contadores do cabeçalho no export

**Decision**: no export, os contadores derivados são **recomputados** das linhas: `total_acoes` (count de ações), `acoes_com_participacoes` (ações com ≥1 participação), `total_participacoes` por ação (count de linhas), `total_publico_alvo`/`total_equipe` (count por `tipo`), `total_atividades` (distintos `atividade_num` entre participações). `campus` vem verbatim de `src_meta` (importado), editável como metadado.

**Rationale**: contadores derivados que não acompanham as edições ficariam mentirosos depois da curadoria. Numa importação sem edição a recomputação reproduz os valores originais — exceto o caso (raro) de atividade coletada com zero pessoas, que some do flatten e reduziria `total_atividades` no export; diferença esperada **declarada** aqui (Princípio II: nada silencioso).

**Alternatives considered**: guardar o cabeçalho verbatim e nunca atualizar — rejeitado: exporta números inconsistentes com o conteúdo editado.

## R5 — Navegação e seleção de projeto

**Decision**: nova página `/projects` como principal pós-login (dois cartões: Horizon e SRC). A raiz `/` passa a redirecionar o autenticado para `/projects` (hoje vai direto a `/dashboard`). `/dashboard` permanece exatamente como é (Horizon); nova área `/src` com layout próprio (menu lateral com Ações + controle de import/export do JSON + atalho "Trocar projeto" de volta a `/projects`). O layout do Horizon também recebe o atalho "Trocar projeto".

**Rationale**: atende FR-001/FR-002/FR-003 sem alterar nenhum comportamento do Horizon. O teste `ROUTES` em `lib.rs` (fonte da verdade de navegação, hoje 18 rotas) é atualizado junto — padrão já estabelecido pelo repositório.

**Alternatives considered**: seletor dentro da dashboard — rejeitado: a spec exige uma nova página principal de seleção, e a base não deve carregar antes da escolha (FR-002).

## R6 — Comandos IPC: dedicados para SRC, genéricos preservados

**Decision**: os comandos genéricos (`list/get/create/update/delete_entity`) continuam servindo **apenas** o Horizon. O SRC recebe comandos dedicados que encapsulam a sincronização projeção↔`raw_json` e o escopo de projeto: CRUD de ações, CRUD de participações por ação, `import_src_json` e `export_src_json`. `api.js` segue sendo o único ponto de acoplamento e ganha as rotas correspondentes.

**Rationale**: toda escrita SRC precisa tocar `raw_json` — regra que o CRUD genérico não pode conhecer. Manter os comandos do Horizon intocados elimina qualquer risco de regressão na base existente (FR-005) e mantém a fronteira auditável em `api.js` (Princípio VI).

**Alternatives considered**: estender `crud.rs` genérico com hooks de sincronização — rejeitado: acoplaria a regra SRC ao caminho do Horizon exatamente quando a feature exige separá-los.

## R7 — Import/export do JSON do SRC

**Decision**: `import_src_json` valida a estrutura (objeto raiz com `acoes` como lista; cada item com `acao_id`; `participacoes` lista quando presente), cria snapshot do banco (mesma rotina `rusqlite::backup` do import Horizon, com prefixo `src-`), substitui (TRUNCATE + recarga, nunca mescla) e emite progresso por evento (`src-import://progress`) — mesmo ritual do import canônico (FR-006..009). `export_src_json` abre diálogo de salvar, remonta o arquivo e escreve com `indent=2`. Diálogos de arquivo continuam abrindo no Rust (padrão SEP-018 — o front-end nunca toca o sistema de arquivos).

**Rationale**: espelha o fluxo de confiança já validado pelo usuário no Horizon (snapshot, substituição, progresso, mensagem clara de falha com base intacta). Validação estrutural rejeita JSON de outra origem (edge case da spec) sem apagar nada.

**Alternatives considered**: apontar um caminho fixo de arquivo — rejeitado: o arquivo consolidado vive em outra pasta/projeto e o usuário escolhe onde salvar a devolução.

## R8 — Estratégia de testes

**Decision**: fixtures do consolidado em `src-tauri/tests/fixtures/` (3 ações: uma com participações, uma sem, uma com campo de rótulo extra/ausente) cobrindo round-trip byte-objetual, validação/abort, snapshot e CRUD com sincronização de projeções — tudo em `cargo test` sem Tauri (padrão dos módulos vizinhos). No front-end, Vitest para `/projects` (roteamento e cartões), rotas novas do `api.js` e editor de participações; um smoke e2e (`frontend/tests/e2e`, `tauri-driver`) para o fluxo login → seleção de projeto → área SRC. Reavaliação: as contagens do export são asserção explícita do teste de round-trip (R4).

**Rationale**: Princípios I e III — todo comportamento novo nasce com teste; a paridade SRC (round-trip) é verificável mecanicamente contra fixture.

**Alternatives considered**: gerar fixtures com o `src-etl` real nos testes — rejeitado: dependência de rede/Playwright em teste unitário viola o isolamento exigido.

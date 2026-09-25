# Research: Adaptação ao Novo Export Canonical (Somente JSON + Aba Campus)

**Branch**: `032-canonical-export-json` | **Date**: 2026-09-25

Não há itens NEEDS CLARIFICATION: o Technical Context ficou completo com a stack vigente do repositório e as decisões abaixo foram tomadas a partir da análise do pacote novo (`exports_canonical.zip`, 395 entradas, sem `.parquet`) e do código atual (`src-tauri/src/import.rs`, `export.rs`, `parquet_io.rs`, `registry.rs`).

## R1. Como localizar a tabela canônica de cada entidade no pacote

**Decision**: para cada tabela gerenciada do `registry::exported()`, procurar **nesta ordem**: (1) `{tabela}_canonical.json` na raiz; (2) `parquet/{tabela}_canonical.parquet` (fallback legado). Se nenhuma das duas existir, abortar todo o import com mensagem nomeando a tabela ausente — **antes** do snapshot e de qualquer `DELETE`.

**Rationale**: o pacote novo só tem JSON na raiz (verificado: 26 `{tabela}_canonical.json`, zero parquets); o pacote antigo tem os dois (idênticos entre si), então a precedência do JSON não muda nada para pacotes antigos e mantém FR-006. A verificação prévia (FR-014) é o que impede o apagão silencioso: hoje o import faz `DELETE` de todas as tabelas antes de saber o que o pacote carrega; com um pacote JSON-only o loop atual de entradas `.parquet` carregaria zero tabelas.

**Alternatives considered**:
- *Loop pelas entradas do ZIP (como hoje, filtrando `.parquet`)* — rejeitado: no pacote novo carregaria 0 tabelas; a direção correta é iterar pelas entidades do registry, não pelas entradas.
- *Fazer o import tolerante a tabela ausente (carrega o resto, sinaliza no resumo)* — rejeitado pelo usuário na clarificação Q1: risco de perda silenciosa de dados curados.
- *Manter só JSON e rejeitar pacotes legados com erro* — rejeitado: descartaria a capacidade de importar pacotes antigos ainda existentes em máquinas de usuários; o fallback custa pouco porque `parquet_io.rs` já existe e está testado.

## R2. Fidelidade de tipos sem o schema do parquet

**Decision**: a inferência de tipos passa a ler o **JSON canônico da tabela no `original.zip`** no momento do export: para cada coluna, o tipo do primeiro valor não-null do arquivo original (número → inteiro ou decimal; booleano; texto; lista/objeto) determina como o texto armazenado no SQLite é convertido de volta. Coluna sem tipo conhecido (tabela nunca importada, coluna nova curada no app) sai como texto — o mesmo caminho degradado de hoje.

**Rationale**: substitui diretamente o mecanismo da SEP-019 (que lia o schema arrow do `parquet/{tabela}_canonical.parquet` do original — arquivo que não existe mais) mantendo o mesmo princípio: o pacote original é a única fonte de verdade dos tipos, e o banco continua todo-TEXT. JSON não declara schema, mas pandas/como o pacote é gerado, colunas são homogêneas — primeiro não-null é um inferidor suficiente e barato. Números inteiros que chegaram como `23` voltam como `23`; `23.0` volta como `23.0` (o texto armazenado preserva a forma); `true` volta booleano; `null` volta null; vínculos voltam como estrutura real.

**Alternatives considered**:
- *Tabela de metadados de tipos capturada no import (nova migration `column_types`)* — rejeitada: maquinaria nova (migration + escrita + teste) para guardar informação que o `original.zip` já guarda; o export já abre o original para preservar entradas.
- *Inferir tipo por valor individual no export (linha a linha)* — rejeitada: colunas heterogêneas na saída quebram consumidores downstream; inferência por coluna é o comportamento que os consumidores já esperam (schema única por coluna, como parquet).
- *Guardar tudo como JSON nativo em colunas SQLite JSON* — rejeitada: violaria a arquitetura todo-TEXT vigente e exigiria reescrever CRUD/busca/ordenação.

## R3. Destino da coluna `campus` da tabela `campuses`

**Decision**: remover `campus` dos `columns` da `EntityDef` de campuses em `registry.rs` e remover o `formField`/coluna da página `campuses.astro` (listagem `id` + `name`). **Sem migration destrutiva**: a coluna física `campus TEXT` permanece em `001_init.sql` dormente (nunca mais lida nem escrita: import insere só colunas do registry; export seleciona só colunas do registry).

**Rationale**: `ALTER TABLE ... DROP COLUMN` no SQLite exige rebuild e uma migration `002` só para eliminar um valor que já fica invisível; o próximo import substitui a base inteira e zera os restos. O teste `dashboard_pages_only_declare_columns_that_exist` do registry guarda a sincronia página↔registry. As colunas `campus` de **outras** entidades (researchers, students, articles etc.) são vínculos legítimos e permanecem intocadas — o "Campus (Vínculos)" só é fantasma na aba Campuses.

**Alternatives considered**:
- *Migration 002 com `DROP COLUMN`* — rejeitada: risco/deploy desnecessários para dado já morto; nenhuma regra lê essa coluna após a mudança do registry.
- *Manter a coluna no registry mas nunca populá-la* — rejeitada: a listagem/formulário continuariam oferecendo o campo fantasma (fere FR-007) e o export voltaria a emitir o aninhamento.

## R4. Pacotes legados no export (parquets antigos no `original.zip`)

**Decision**: no export, os arquivos `parquet/{tabela}_canonical.parquet` do original **não são mais gerados e não são preservados** — são descartados como resíduo do formato antigo. O conjunto "preservado byte a byte" passa a ser: toda entrada do original que não seja nem `{tabela}_canonical.json` (gerado) nem `parquet/{tabela}_canonical.parquet` de tabela gerenciada (descartado).

**Rationale**: preservar os parquets legados deixaria dados velhos dentro do pacote devolvido (o parquet congelaria o estado pré-importação enquanto o JSON carrega a base curada) — exatamente o tipo de inconsistência que o pipeline downstream não pode receber. É diferença **declarada** em relação ao comportamento da SEP-019 (constituição, Princípio II): o formato de saída muda por decisão da spec (FR-009), não é correção silenciosa.

**Alternatives considered**:
- *Preservar os parquets antigos intactos quando o original é legado* — rejeitada: pacote com dados duplicados divergentes é pior que pacote limpo no formato novo.
- *Regenerar os parquets junto com os JSONs (manter os dois formatos na saída)* — rejeitada: a spec (FR-009) e o consumidor definiram JSON-only; gerar parquet que ninguém espera é lixo no pacote.

## R5. Conversão JSON ⇄ armazenamento TEXT

**Decision**: no import, cada valor JSON vira texto com spellings canônicos: `true/false` → `"True"/"False"` (mesma convenção do leitor parquet atual, que o export já reconhece), números → representação decimal textual do próprio JSON (serde preserva `23` vs `23.0`), `null` → SQL NULL, lista/objeto → texto JSON compacto. No export, a inversão usa o tipo inferido (R2): booleano reconhece `1/True/true/0/False/false` (função `as_bool` existente), número faz parse com fallback `i64`→`f64`, estrutura revalida como JSON.

**Rationale**: reutiliza as convenções já testadas do round-trip parquet (`parquet_io::cell` e `export::as_bool`), mantém um único caminho de armazenamento e garante que `null` nunca vire string vazia nem `"NaN"` (FR-003/FR-012).

**Alternatives considered**:
- *Spellings novos (`"true"` minúsculo) para o caminho JSON* — rejeitada: criaria dois dialetos dentro do mesmo banco e quebraria a comparação entre imports legado e novo.

## R6. Campos extras e ausentes nos registros JSON

**Decision**: campos do registro que não pertencem ao `columns` da entidade são ignorados (nunca reaparecem no export); campos gerenciados ausentes no registro entram como NULL. A falha só acontece se o arquivo inteiro não for JSON válido (FR-005) ou se faltar o arquivo da tabela (FR-014).

**Rationale**: absorve evolução do formato DataLake (spec, Assumptions) e reproduz o comportamento do leitor parquet atual, que só seleciona as colunas desejadas.

**Alternatives considered**: *falhar em campo desconhecido* — rejeitada: tornaria cada evolução do upstream uma quebra do app.

## R7. Orderm de operações do import

**Decision**: sequência nova do `import_archive`: (1) abrir ZIP; (2) resolver fonte (JSON/parquet) por tabela e validar completude das 15 (FR-014) — aborta aqui se faltar algo; (3) snapshot se há dados (comportamento atual); (4) `DELETE` das tabelas; (5) carga transacional tabela a tabela com progresso; (6) commit; (7) guardar `original.zip`.

**Rationale**: validar antes do snapshot evita snapshots inúteis de uma importação que falhou; validar antes do `DELETE` é o que materializa a decisão Q1 ("base permanece intacta"). A transação única vigente (FR-005) permanece como segunda linha de defesa.

**Alternatives considered**: *validar depois do snapshot* — rejeitada: geraria arquivo de snapshot órfão a cada pacote quebrado.

# Contract: Layout do Pacote Canônico (fronteira com o DataLake)

**Branch**: `032-canonical-export-json` | **Date**: 2026-09-25

Contrato externo do artefato que o Research Hub consome e devolve. É a interface entre o curador (via app) e os pipelines do DataLake.

## Formato consumido (import)

```text
exports_canonical.zip
├── {tabela}_canonical.json          # 15 gerenciadas (obrigatórias) + canônicos não gerenciados
├── parquet/{tabela}_canonical.parquet  # somente em pacotes legados; fallback (FR-006)
├── *_relationship_graph.json / *_collaboration_graph.json / *_tracking.json / *_mart.json
├── research_group_relationship_graphs/*.json
├── data_snapshot.zip
└── (demais entradas não gerenciadas)
```

- `{tabela}_canonical.json`: array de objetos; campos = colunas gerenciadas + possíveis extras (ignorados). Valores: `null`, booleano, número, string, array, objeto.
- **Aborta o import** se qualquer uma das 15 tabelas não tiver nem JSON nem parquet (FR-014). Tabelas: `researchers`, `students`, `articles`, `research_groups`, `initiatives`, `advisorships`, `awards`, `campuses`, `organizations`, `fellowships`, `proficiencies`, `professional_activities`, `knowledge_areas`, `languages`, `research_productions`.
- JSON malformado de qualquer tabela gerenciada aborta o import inteiro (FR-005).

## Formato produzido (export)

```text
portal_export_canonical.zip
├── {tabela}_canonical.json          # 15 gerenciadas, regeneradas da base curada
├── (todas as entradas não gerenciadas do original, byte a byte, sem recompressão)
└── (SEM parquet — nunca gerado; parquets legados do original são descartados)
```

- Serialização dos JSONs: `indent=4`, `ensure_ascii=false` (UTF-8 literal), `null` explícito (nunca `NaN`), vínculos como estrutura real.
- Tipos restaurados por coluna a partir do JSON do pacote original (primeiro não-null); coluna sem referência sai como string.

## Exemplos

`campuses_canonical.json` (v2, corrigido — 23 registros no pacote atual):

```json
[
    {
        "id": 1,
        "name": "Vila Velha",
        "description": null,
        "short_name": null,
        "organization_id": 1,
        "parent_id": null
    }
]
```

Formato v1 (legado, aceito no import; **nunca** produzido no export):

```json
[
    {
        "id": 6,
        "name": "Serra",
        "description": null,
        "short_name": null,
        "organization_id": 1,
        "parent_id": null,
        "campus": { "id": 6, "name": "Serra" }
    }
]
```

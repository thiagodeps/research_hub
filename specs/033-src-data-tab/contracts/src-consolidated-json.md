# Contrato: Arquivo JSON Consolidado do SRC (entrada/saída da área SRC)

**Fonte da verdade**: `src-etl-consolidate` (`/home/thiagodeps/src/src/src_etl/etl/consolidar.py`). Este contrato descreve o formato que `import_src_json` aceita e `export_src_json` produz. Ver research.md R2/R4 para as decisões.

## Estrutura

```json
{
  "campus": "Serra",
  "total_acoes": 2,
  "acoes_com_participacoes": 1,
  "total_atividades": 2,
  "total_publico_alvo": 3,
  "total_equipe": 1,
  "acoes": [
    {
      "acao_id": "12345",
      "url_detalhe": "https://.../detalhe.jsf?id=12345",
      "Campus": "Serra",
      "Processo nº": "1234.567/2026",
      "Natureza": "Extensão",
      "Coordenador(a)": "Nome do Coordenador",
      "Tipo ação": "Projeto",
      "Título ação": "Título da Ação",
      "Fomento": "...",
      "Ação vinculante": "Processo ou título do programa pai",
      "Grande área conhecimento": "...",
      "Área temática principal": "...",
      "Área temática secundária": "...",
      "Relatório aprovado": "Sim",
      "Data último relatorio": "...",
      "Data de cadastro": "...",
      "Resumo": "...",
      "total_participacoes": 4,
      "participacoes": [
        {
          "atividade_num": "1",
          "atividade_id": "98765",
          "atividade": "Nome da atividade",
          "tipo": "Público-alvo",
          "Nome": "Nome da Pessoa",
          "CPF": "***",
          "E-mail": "***",
          "Situação": "APROVADO"
        },
        {
          "atividade_num": "2",
          "atividade_id": "98766",
          "atividade": "Outra atividade",
          "tipo": "Equipe de execução",
          "Nome": "Nome do Membro",
          "Função": "Coordenador",
          "Vínculo": "..."
        }
      ]
    },
    {
      "acao_id": "12346",
      "total_participacoes": 0,
      "participacoes": []
    }
  ]
}
```

## Regras do contrato

| # | Regra | Direção |
|---|---|---|
| C1 | Raiz é objeto com chave `acoes` (lista) | Import valida; sem isso aborta com mensagem clara |
| C2 | Cada ação é objeto com `acao_id` (texto não vazio) | Import valida; `acao_id` duplicado aborta |
| C3 | `participacoes` presente ⇒ lista | Import valida |
| C4 | `acoes: []` é válido (base vazia informada) | Import aceita |
| C5 | Chaves dos campos vêm dos rótulos da página do SRC (`"Processo nº"`, `"Título ação"`, `"Nome"`, ...). Chaves desconhecidas são preservadas íntegras (`extra: "allow"` na fonte) | Import→export: byte-objetual para registros não editados |
| C6 | `tipo` da participação: `"Público-alvo"` \| `"Equipe de execução"` | Outro valor entra como veio e é sinalizado (o arquivo é o oráculo) |
| C7 | O arquivo contém PII; uso local | Nunca logar, telemetrar ou publicar conteúdo de participações |
| C8 | Contadores da raiz são derivados das linhas; recomputados no export (research R4). Diferença esperada declarada: `total_atividades` pode reduzir se o arquivo de origem contou atividade com zero pessoas | Export |
| C9 | Serialização: `indent=2`, não-ASCII sem escape, ordem de chaves dos objetos originais preservada | Export |
| C10 | `campus` da raiz: verbatim do import (editável como metadado) | Export |

## Fidelidade de round-trip (SC-003)

Importar sem editar e exportar deve produzir arquivo equivalente campo a campo: mesmos objetos de ação (chaves, ordem, valores), mesmas listas de participações na mesma ordem, mesmos contadores (exceto o caso declarado em C8). Esta é a asserção central do teste de round-trip (`cargo test`, fixture `tests/fixtures/src_consolidado_exemplo.json`).

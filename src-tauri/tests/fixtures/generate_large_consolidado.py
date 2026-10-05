#!/usr/bin/env python3
"""One-off generator for the SC-006 performance fixture (SEP-033 T041).

Writes src_consolidado_grande.json next to this script: ~2.000 ações and
~10.000 participações, same shape as consolidar.py's output (contract
src-consolidated-json.md). Disposable — the file it produces must never be
committed (it carries fake PII and would bloat the repo).
"""
import json
import pathlib
import random

random.seed(33)  # reproducible

ACOES = 2000
PARTS = 5  # per ação → ~10.000

def fake_cpf(i):
    s = f"{i:011d}"
    return f"{s[0:3]}.{s[3:6]}.{s[6:9]}-{s[9:11]}"

acoes = []
for a in range(ACOES):
    partes = []
    for p in range(PARTS):
        tipo = "Público-alvo" if (a + p) % 2 == 0 else "Equipe de execução"
        part = {
            "atividade_num": str((p % 2) + 1),
            "atividade_id": str(9000 + a * 2 + (p % 2)),
            "atividade": f"Turma {(p % 2) + 1}",
            "tipo": tipo,
            "Nome": f"Participante {a * PARTS + p}",
            "CPF": fake_cpf(a * PARTS + p),
            "E-mail": f"p{a * PARTS + p}@exemplo.local",
        }
        if tipo == "Público-alvo":
            part["Situação"] = "APROVADO"
        else:
            part["Função"] = "Bolsista"
            part["Vínculo"] = "Estudante"
        partes.append(part)
    acoes.append({
        "acao_id": str(10000 + a),
        "url_detalhe": f"https://src.ifes.edu.br/detalhe.jsf?id={10000 + a}",
        "Campus": "Serra",
        "Processo nº": f"0123.{a:04d}/2025",
        "Natureza": "Extensão",
        "Coordenador(a)": f"Coordenador {a}",
        "Tipo ação": "Projeto",
        "Título ação": f"Ação sintética {a}",
        "Ação vinculante": "" if a == 0 else "0123.0000/2025",
        "Relatório aprovado": "Sim",
        "Data de cadastro": "10/03/2025",
        "Resumo": f"Resumo sintético da ação {a}.",
        "total_participacoes": len(partes),
        "participacoes": partes,
    })

root = {
    "campus": "Serra",
    "total_acoes": len(acoes),
    "acoes_com_participacoes": len(acoes),
    "total_atividades": len(acoes) * 2,
    "total_publico_alvo": len(acoes) * PARTS // 2,
    "total_equipe": len(acoes) * PARTS // 2,
    "acoes": acoes,
}

dest = pathlib.Path(__file__).parent / "src_consolidado_grande.json"
dest.write_text(json.dumps(root, ensure_ascii=False, indent=2), encoding="utf-8")
print(f"ok: {dest} ({dest.stat().st_size / 1e6:.1f} MB, {ACOES} ações, {ACOES * PARTS} participações)")

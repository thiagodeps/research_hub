import React, { useState, useEffect } from 'react';
import { apiFetch } from '../services/api.js';

// SEP-033 / US3 (FR-012): editor da coleção aninhada da ação. As participações
// são exibidas agrupadas por `tipo` e contexto de atividade, com os campos da
// pessoa sob os rótulos do arquivo (C5) — Nome, CPF, E-mail, Situação, Função,
// Vínculo — mais qualquer chave extra que o scraper tenha gravado.
const TIPOS_CANONICOS = ['Público-alvo', 'Equipe de execução'];
const CTX_FIELDS = [
  ['atividade_num', 'Atividade nº'],
  ['atividade_id', 'ID da atividade'],
  ['atividade', 'Atividade'],
];
const PESSOA_FIELDS = ['Nome', 'CPF', 'E-mail', 'Situação', 'Função', 'Vínculo'];
// Chaves de sistema que não aparecem como campos da pessoa.
const SYSTEM_KEYS = ['id', 'ord', 'tipo', 'atividade_num', 'atividade_id', 'atividade', 'nome'];

const EMPTY_FORM = {
  tipo: 'Público-alvo',
  atividade_num: '',
  atividade_id: '',
  atividade: '',
  Nome: '',
  CPF: '',
  'E-mail': '',
  Situação: '',
  Função: '',
  Vínculo: '',
};

export default function ParticipacoesEditor({ acaoId, vinculo = { text: '', resolvido: true }, onChanged }) {
  const [items, setItems] = useState([]);
  const [form, setForm] = useState(EMPTY_FORM);
  const [editingId, setEditingId] = useState(null);
  const [error, setError] = useState(null);
  const [busy, setBusy] = useState(false);

  const load = async () => {
    try {
      setError(null);
      setItems(await apiFetch(`/src/acoes/${acaoId}/participacoes`));
    } catch (e) {
      setError(e?.message ?? String(e));
    }
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [acaoId]);

  const setField = (key, value) => setForm((prev) => ({ ...prev, [key]: value }));

  const payloadFromForm = () => {
    // Only non-empty fields travel — absent keys keep the file's own style.
    return Object.fromEntries(Object.entries(form).filter(([, v]) => String(v).trim() !== ''));
  };

  const handleAdd = async () => {
    setBusy(true);
    setError(null);
    try {
      await apiFetch(`/src/acoes/${acaoId}/participacoes`, {
        method: 'POST',
        body: JSON.stringify(payloadFromForm()),
      });
      setForm(EMPTY_FORM);
      await load();
      onChanged?.();
    } catch (e) {
      setError(e?.message ?? String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleSaveEdit = async () => {
    setBusy(true);
    setError(null);
    try {
      await apiFetch(`/src/participacoes/${editingId}`, {
        method: 'PUT',
        body: JSON.stringify(payloadFromForm()),
      });
      setEditingId(null);
      setForm(EMPTY_FORM);
      await load();
      onChanged?.();
    } catch (e) {
      setError(e?.message ?? String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleRemove = async (id) => {
    if (!window.confirm('Remover esta participação? A ordem das demais será renumerada.')) return;
    setBusy(true);
    setError(null);
    try {
      await apiFetch(`/src/participacoes/${id}`, { method: 'DELETE' });
      await load();
      onChanged?.();
    } catch (e) {
      setError(e?.message ?? String(e));
    } finally {
      setBusy(false);
    }
  };

  const startEdit = (row) => {
    setEditingId(row.id);
    setForm({
      ...EMPTY_FORM,
      tipo: row.tipo ?? 'Público-alvo',
      ...Object.fromEntries(
        Object.entries(row).filter(
          ([k, v]) => !SYSTEM_KEYS.includes(k) && v !== null && v !== undefined
        )
      ),
    });
  };

  // Agrupamento: tipo → contexto de atividade, preservando a ordem do arquivo.
  const groups = [];
  for (const item of items) {
    const tipo = item.tipo || '';
    let group = groups.find((g) => g.tipo === tipo);
    if (!group) {
      group = { tipo, contexts: [] };
      groups.push(group);
    }
    const ctxKey = `${item.atividade_num ?? ''}|${item.atividade_id ?? ''}|${item.atividade ?? ''}`;
    let ctx = group.contexts.find((c) => c.key === ctxKey);
    if (!ctx) {
      ctx = { key: ctxKey, num: item.atividade_num, nome: item.atividade, rows: [] };
      group.contexts.push(ctx);
    }
    ctx.rows.push(item);
  }
  groups.sort((a, b) => {
    const ia = TIPOS_CANONICOS.indexOf(a.tipo);
    const ib = TIPOS_CANONICOS.indexOf(b.tipo);
    return (ia === -1 ? 99 : ia) - (ib === -1 ? 99 : ib);
  });

  const extraKeysOf = (row) =>
    Object.keys(row).filter(
      (k) => !SYSTEM_KEYS.includes(k) && !PESSOA_FIELDS.includes(k) && row[k] !== null && row[k] !== undefined
    );

  return (
    <div className="mt-6 bg-white border border-slate-200 rounded-lg p-6">
      <h3 className="text-lg font-semibold text-slate-800 mb-1">Participações</h3>
      <p className="text-sm text-slate-500 mb-4">
        Uma linha por pessoa, como no arquivo consolidado. As chaves de rótulo
        originais são preservadas.
      </p>

      {vinculo?.text && !vinculo.resolvido && (
        <div className="mb-4 p-3 rounded bg-amber-50 text-amber-700 text-sm">
          Vínculo não encontrado: esta ação aponta para
          {' '}<strong>{vinculo.text}</strong> como &quot;Ação vinculante&quot;, mas nenhuma
          ação com esse processo/título existe na base. Ajuste o campo na ação
          ou crie a ação pai.
        </div>
      )}

      {error && <div className="mb-4 p-3 rounded bg-red-50 text-red-600 text-sm">{error}</div>}

      <div className="space-y-6">
        {groups.map((group) => (
          <div key={group.tipo}>
            <h4 className="text-md font-semibold text-slate-700 border-b border-slate-200 pb-1 mb-3">
              {group.tipo || 'Sem tipo'}
            </h4>
            {group.contexts.map((ctx) => (
              <div key={ctx.key} className="mb-4">
                {(ctx.num || ctx.nome) && (
                  <p className="text-xs font-medium text-slate-500 mb-2">
                    Atividade {ctx.num || '—'}
                    {ctx.nome ? ` — ${ctx.nome}` : ''}
                  </p>
                )}
                <ul className="space-y-2">
                  {ctx.rows.map((row) => (
                    <li
                      key={row.id}
                      className="flex flex-col sm:flex-row sm:items-start justify-between gap-2 border border-slate-200 rounded p-3"
                    >
                      <div className="text-sm space-y-1">
                        <p className="font-medium text-slate-800">{row.Nome || row.nome || '(sem nome)'}</p>
                        <div className="text-xs text-slate-500 space-y-0.5">
                          {PESSOA_FIELDS.filter((k) => k !== 'Nome' && row[k])
                            .map((k) => (
                              <p key={k}>
                                <span className="font-semibold">{k}:</span> {row[k]}
                              </p>
                            ))}
                          {extraKeysOf(row).map((k) => (
                            <p key={k}>
                              <span className="font-semibold">{k}:</span> {String(row[k])}
                            </p>
                          ))}
                        </div>
                      </div>
                      <div className="flex gap-2 shrink-0">
                        <button
                          type="button"
                          onClick={() => startEdit(row)}
                          className="text-indigo-600 hover:text-indigo-800 text-sm font-medium"
                        >
                          Editar
                        </button>
                        <button
                          type="button"
                          onClick={() => handleRemove(row.id)}
                          className="text-red-600 hover:text-red-800 text-sm font-medium"
                        >
                          Remover
                        </button>
                      </div>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        ))}
        {items.length === 0 && !error && (
          <p className="text-sm text-slate-500">Nenhuma participação nesta ação.</p>
        )}
      </div>

      <div className="mt-6 border-t border-slate-200 pt-4">
        <h4 className="text-md font-semibold text-slate-700 mb-3">
          {editingId ? 'Editar participação' : 'Adicionar participação'}
        </h4>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          <label className="text-sm">
            <span className="block mb-1 font-medium text-slate-700">Tipo</span>
            <select
              aria-label="tipo"
              value={form.tipo}
              onChange={(e) => setField('tipo', e.target.value)}
              className="w-full px-3 py-2 border border-slate-300 rounded-md"
            >
              {TIPOS_CANONICOS.map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
          </label>
          {CTX_FIELDS.map(([key, label]) => (
            <label key={key} className="text-sm">
              <span className="block mb-1 font-medium text-slate-700">{label}</span>
              <input
                aria-label={key}
                type="text"
                value={form[key] ?? ''}
                onChange={(e) => setField(key, e.target.value)}
                className="w-full px-3 py-2 border border-slate-300 rounded-md"
              />
            </label>
          ))}
          {PESSOA_FIELDS.map((key) => (
            <label key={key} className="text-sm">
              <span className="block mb-1 font-medium text-slate-700">{key}</span>
              <input
                aria-label={key}
                type="text"
                value={form[key] ?? ''}
                onChange={(e) => setField(key, e.target.value)}
                className="w-full px-3 py-2 border border-slate-300 rounded-md"
              />
            </label>
          ))}
        </div>
        <div className="flex justify-end gap-3 mt-4">
          {editingId && (
            <button
              type="button"
              onClick={() => {
                setEditingId(null);
                setForm(EMPTY_FORM);
              }}
              className="px-4 py-2 text-sm text-slate-700 bg-white border border-slate-300 rounded-md hover:bg-slate-50"
            >
              Cancelar edição
            </button>
          )}
          <button
            type="button"
            onClick={editingId ? handleSaveEdit : handleAdd}
            disabled={busy}
            className={`px-4 py-2 text-sm font-medium text-white rounded-md transition-colors ${
              editingId ? 'bg-emerald-600 hover:bg-emerald-700' : 'bg-indigo-600 hover:bg-indigo-700'
            } ${busy ? 'opacity-50 cursor-not-allowed' : ''}`}
          >
            {editingId ? 'Salvar edição' : 'Adicionar'}
          </button>
        </div>
      </div>
    </div>
  );
}

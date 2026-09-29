import React, { useState, useEffect } from 'react';
import { apiFetch } from '../services/api.js';
import EntityTable from './EntityTable.jsx';
import EntityForm from './EntityForm.jsx';
import ParticipacoesEditor from './ParticipacoesEditor.jsx';

// SEP-033 / US3 (FR-010/FR-011/FR-013): curadoria das ações no caminho
// dedicado /src — mesmo ritual do EntityPage do Horizon, porém sempre nos
// comandos SRC, com exclusão consciente de vínculos (Conflict → confirmação
// forçada) e o editor de participações embutido na ação aberta.
const COLUMNS = ['titulo', 'processo', 'natureza', 'tipo', 'coordenador'];

const FIELDS = [
  { name: 'id', label: 'ID', type: 'number' },
  { name: 'acao_id', label: 'Ação ID (pipeline)' },
  { name: 'processo', label: 'Processo nº' },
  { name: 'titulo', label: 'Título ação' },
  { name: 'natureza', label: 'Natureza' },
  { name: 'tipo', label: 'Tipo ação' },
  { name: 'coordenador', label: 'Coordenador(a)' },
  { name: 'acao_vinculante', label: 'Ação vinculante (processo ou título da ação pai)' },
  { name: 'campus', label: 'Campus' },
  { name: 'resumo', label: 'Resumo' },
];

const LIMIT = 50;

export default function SrcAcoesPage() {
  const [data, setData] = useState([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(0);
  const [search, setSearch] = useState('');
  const [sortCol, setSortCol] = useState(null);
  const [sortOrder, setSortOrder] = useState('asc');
  const [editingItem, setEditingItem] = useState(null);
  const [vinculo, setVinculo] = useState({ text: '', resolvido: true });
  const [message, setMessage] = useState(null);
  const [dirty, setDirty] = useState(false);

  const totalPages = Math.max(1, Math.ceil(total / LIMIT));

  const loadData = async () => {
    try {
      const offset = page * LIMIT;
      let url = `/src/acoes?limit=${LIMIT}&offset=${offset}`;
      if (search) url += `&search=${encodeURIComponent(search)}`;
      if (sortCol) url += `&sort=${encodeURIComponent(sortCol)}&order=${sortOrder}`;
      const res = await apiFetch(url);
      setData(res.items || []);
      setTotal(res.total || 0);
    } catch (e) {
      setMessage({ type: 'error', text: e?.message ?? String(e) });
    }
  };

  useEffect(() => {
    loadData();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [page, search, sortCol, sortOrder]);

  // US1-5 / FR-003: o estado do formulário ativo é anunciado para os layouts —
  // é isso que a seleção de projeto e o menu consultam antes de navegar.
  useEffect(() => {
    window.dispatchEvent(new CustomEvent('src:dirty', { detail: dirty }));
    const guard = (e) => {
      if (dirty) {
        e.preventDefault();
        e.returnValue = '';
      }
    };
    window.addEventListener('beforeunload', guard);
    return () => window.removeEventListener('beforeunload', guard);
  }, [dirty]);

  // FR-013: resolver o vínculo declarado — procurar uma ação cujo processo ou
  // título seja exatamente o texto de "Ação vinculante".
  useEffect(() => {
    const text = editingItem?.acao_vinculante?.trim();
    if (!editingItem?.id || !text) {
      setVinculo({ text: '', resolvido: true });
      return;
    }
    let alive = true;
    (async () => {
      try {
        const res = await apiFetch(
          `/src/acoes?limit=1000&search=${encodeURIComponent(text)}`
        );
        const resolvido = (res.items || []).some(
          (r) =>
            r.id !== editingItem.id &&
            (r.processo === text || r.titulo === text)
        );
        if (alive) setVinculo({ text, resolvido });
      } catch {
        if (alive) setVinculo({ text, resolvido: false });
      }
    })();
    return () => {
      alive = false;
    };
  }, [editingItem?.id, editingItem?.acao_vinculante]);

  const openEdit = async (row) => {
    try {
      // O resumo só existe dentro do raw_json — a listagem não o traz.
      const full = await apiFetch(`/src/acoes/${row.id}`);
      setEditingItem(full ?? row);
      setDirty(true);
      setMessage(null);
    } catch (e) {
      setMessage({ type: 'error', text: e?.message ?? String(e) });
    }
  };

  const handleSave = async (payload) => {
    try {
      const isEditing = editingItem?.id !== undefined && editingItem?.id !== null;
      if (isEditing) {
        await apiFetch(`/src/acoes/${editingItem.id}`, {
          method: 'PUT',
          body: JSON.stringify(payload),
        });
      } else {
        await apiFetch('/src/acoes', {
          method: 'POST',
          body: JSON.stringify(payload),
        });
      }
      setEditingItem(null);
      setDirty(false);
      loadData();
    } catch (e) {
      setMessage({ type: 'error', text: e?.message ?? String(e) });
    }
  };

  const handleDelete = async (id) => {
    try {
      await apiFetch(`/src/acoes/${id}`, { method: 'DELETE' });
      setMessage(null);
      loadData();
    } catch (e) {
      const text = e?.message ?? String(e);
      if (/exclusão forçada/.test(text)) {
        // FR-013: o conflito é decidido pelo curador — nunca silencioso.
        if (window.confirm(`${text}\n\nExcluir mesmo assim? As ações filhas serão desvinculadas.`)) {
          try {
            await apiFetch(`/src/acoes/${id}?force=true`, { method: 'DELETE' });
            setMessage(null);
            loadData();
            return;
          } catch (e2) {
            setMessage({ type: 'error', text: e2?.message ?? String(e2) });
            return;
          }
        }
      }
      setMessage({ type: 'error', text });
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center mb-6 gap-4">
        <h1 className="text-2xl font-bold text-slate-900">Ações do SRC</h1>
        <div className="flex w-full sm:w-auto items-center space-x-3">
          <input
            type="text"
            placeholder="Buscar por título..."
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setPage(0);
            }}
            className="w-full sm:w-64 px-4 py-2 text-sm border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-sky-500"
          />
          <button
            onClick={() => {
              setEditingItem({});
              setDirty(true);
              setMessage(null);
            }}
            className="whitespace-nowrap px-4 py-2 text-sm font-medium text-white bg-sky-700 rounded-md hover:bg-sky-800 transition-colors"
          >
            Novo Registro
          </button>
        </div>
      </div>

      {message && (
        <div className={`p-3 rounded text-sm ${message.type === 'error' ? 'bg-red-50 text-red-600' : 'bg-green-50 text-green-700'}`}>
          {message.text}
        </div>
      )}

      {editingItem && (
        <div onChangeCapture={() => setDirty(true)}>
          <EntityForm
            initialData={editingItem}
            fields={FIELDS}
            onSubmit={handleSave}
            onCancel={() => {
              setEditingItem(null);
              setDirty(false);
            }}
          />
          {editingItem?.id && (
            <ParticipacoesEditor
              acaoId={editingItem.id}
              vinculo={vinculo}
              onChanged={loadData}
            />
          )}
        </div>
      )}

      <EntityTable
        entityName="Ações do SRC"
        entities={data}
        columns={COLUMNS}
        fields={FIELDS}
        sortCol={sortCol}
        sortOrder={sortOrder}
        onSort={(col) => {
          if (sortCol === col) {
            setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
          } else {
            setSortCol(col);
            setSortOrder('asc');
          }
        }}
        onEdit={openEdit}
        onDelete={handleDelete}
        onMerge={() => {}}
        onLink={() => {}}
      />

      <div className="flex justify-between items-center mt-4 text-sm text-slate-600">
        <div>
          Mostrando {total === 0 ? 0 : page * LIMIT + 1} a{' '}
          {Math.min((page + 1) * LIMIT, total)} de {total} registros
        </div>
        <div className="flex items-center space-x-2">
          <button
            disabled={page === 0}
            onClick={() => setPage(page - 1)}
            className="px-3 py-1 bg-white border border-slate-300 rounded disabled:opacity-50 hover:bg-slate-50 transition-colors"
          >
            Anterior
          </button>
          <span>
            Página {page + 1} de {totalPages}
          </span>
          <button
            disabled={(page + 1) * LIMIT >= total}
            onClick={() => setPage(page + 1)}
            className="px-3 py-1 bg-white border border-slate-300 rounded disabled:opacity-50 hover:bg-slate-50 transition-colors"
          >
            Próxima
          </button>
        </div>
      </div>
    </div>
  );
}

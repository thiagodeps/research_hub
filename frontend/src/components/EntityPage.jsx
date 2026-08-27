import React, { useState, useEffect } from 'react';
import { apiFetch } from '../services/api';
import EntityTable from './EntityTable';
import EntityForm from './EntityForm';

export default function EntityPage({ entity, columns, fields }) {
  const [data, setData] = useState([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(0);
  const [search, setSearch] = useState("");
  const [sortCol, setSortCol] = useState(null);
  const [sortOrder, setSortOrder] = useState("asc");
  const [editingItem, setEditingItem] = useState(null);
  const limit = 50;

  const loadData = async () => {
    try {
      const offset = page * limit;
      let url = `/${entity}?limit=${limit}&offset=${offset}`;
      if (search) url += `&search=${encodeURIComponent(search)}`;
      if (sortCol) url += `&sort=${encodeURIComponent(sortCol)}&order=${sortOrder}`;
      
      const res = await apiFetch(url, { headers: { Authorization: `Bearer ${localStorage.getItem('token')}` }});
      setData(res.items || []);
      setTotal(res.total || 0);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => { loadData(); }, [entity, page, search, sortCol, sortOrder]);

  // Deep linking: Check URL for openId on mount
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const openId = params.get('openId');
    if (openId) {
      // Remove it from URL so refresh doesn't trigger it again
      window.history.replaceState({}, '', window.location.pathname);
      // Fetch and open item
      apiFetch(`/${entity}/${openId}`, { headers: { Authorization: `Bearer ${localStorage.getItem('token')}` }})
        .then(res => {
          if (res && res.id) setEditingItem(res);
        })
        .catch(err => console.error("Could not load deep link:", err));
    }
  }, [entity]);

  const handleSave = async (payload) => {
    try {
      if (editingItem?.id === payload.id) {
        await apiFetch(`/${entity}/${payload.id}`, {
          method: 'PUT',
          body: JSON.stringify(payload),
          headers: { Authorization: `Bearer ${localStorage.getItem('token')}` }
        });
      } else {
        await apiFetch(`/${entity}`, {
          method: 'POST',
          body: JSON.stringify(payload),
          headers: { Authorization: `Bearer ${localStorage.getItem('token')}` }
        });
      }
      setEditingItem(null);
      loadData();
    } catch (e) {
      alert(e.message);
    }
  };

  const handleDelete = async (id) => {
    try {
      await apiFetch(`/${entity}/${id}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${localStorage.getItem('token')}` }
      });
      loadData();
    } catch (e) {
      alert(e.message);
    }
  };

  const handleMerge = async (ids, resolvedData) => {
    try {
      await apiFetch(`/merge/${entity}`, {
        method: 'POST',
        body: JSON.stringify({ source_ids: ids, resolved_data: resolvedData }),
        headers: { Authorization: `Bearer ${localStorage.getItem('token')}` }
      });
      loadData();
    } catch (e) {
      alert(e.message);
    }
  };

  const handleLink = async (parentTable, parentId, childTable, childId) => {
    try {
      await apiFetch(`/link`, {
        method: 'POST',
        body: JSON.stringify({
          parent_type: parentTable,
          parent_id: parentId,
          child_type: childTable,
          child_id: childId
        }),
        headers: { Authorization: `Bearer ${localStorage.getItem('token')}` }
      });
      alert('Vínculo criado com sucesso!');
      loadData();
    } catch (e) {
      alert(e.message);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center mb-6 gap-4">
        <h1 className="text-2xl font-bold text-slate-900 capitalize">{entity}</h1>
        
        <div className="flex w-full sm:w-auto items-center space-x-3">
          <input 
            type="text" 
            placeholder="Buscar registro..." 
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setPage(0); // Reset page on search
            }}
            className="w-full sm:w-64 px-4 py-2 text-sm border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
          <button 
            onClick={() => setEditingItem({})}
            className="whitespace-nowrap px-4 py-2 text-sm font-medium text-white bg-indigo-600 rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 transition-colors"
          >
            Novo Registro
          </button>
        </div>
      </div>
      
      {editingItem && (
        <EntityForm 
          initialData={editingItem} 
          fields={fields} 
          onSubmit={handleSave} 
          onCancel={() => setEditingItem(null)} 
        />
      )}

      <EntityTable 
        entityName={entity}
        entities={data} 
        columns={columns} 
        fields={fields}
        sortCol={sortCol}
        sortOrder={sortOrder}
        onSort={(col) => {
          if (sortCol === col) {
            setSortOrder(sortOrder === "asc" ? "desc" : "asc");
          } else {
            setSortCol(col);
            setSortOrder("asc");
          }
        }}
        onEdit={setEditingItem} 
        onDelete={handleDelete} 
        onMerge={handleMerge}
        onLink={handleLink}
      />

      <div className="flex justify-between items-center mt-4 text-sm text-slate-600">
        <div>
          Mostrando {page * limit + 1} a {Math.min((page + 1) * limit, total)} de {total} registros
        </div>
        <div className="space-x-2">
          <button 
            disabled={page === 0} 
            onClick={() => setPage(page - 1)}
            className="px-3 py-1 bg-white border border-slate-300 rounded disabled:opacity-50 hover:bg-slate-50 transition-colors"
          >
            Anterior
          </button>
          <button 
            disabled={(page + 1) * limit >= total} 
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

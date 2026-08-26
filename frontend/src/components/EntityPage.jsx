import React, { useState, useEffect } from 'react';
import { apiFetch } from '../services/api';
import EntityTable from './EntityTable';
import EntityForm from './EntityForm';

export default function EntityPage({ entity, columns, fields }) {
  const [data, setData] = useState([]);
  const [editingItem, setEditingItem] = useState(null);

  const loadData = async () => {
    try {
      const res = await apiFetch(`/${entity}`, { headers: { Authorization: `Bearer ${localStorage.getItem('token')}` }});
      setData(res.items || []);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => { loadData(); }, [entity]);

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

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-slate-900 capitalize">{entity}</h1>
        <button 
          onClick={() => setEditingItem({})}
          className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 transition-colors"
        >
          Novo Registro
        </button>
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
        entities={data} 
        columns={columns} 
        onEdit={setEditingItem} 
        onDelete={handleDelete} 
      />
    </div>
  );
}

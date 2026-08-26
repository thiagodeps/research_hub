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
    <div>
      <button onClick={() => setEditingItem({})}>Novo Registro</button>
      
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

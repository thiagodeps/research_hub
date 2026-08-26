import React, { useState } from 'react';

export default function EntityForm({ initialData = {}, fields, onSubmit, onCancel }) {
  const [data, setData] = useState(initialData);

  const handleChange = (name, value) => {
    setData(prev => ({ ...prev, [name]: value }));
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    onSubmit(data);
  };

  return (
    <form onSubmit={handleSubmit} style={{ border: '1px solid #ccc', padding: '1rem', marginTop: '1rem' }}>
      <h3>{initialData.id ? 'Editar' : 'Criar'}</h3>
      {fields.map(f => (
        <div key={f.name} style={{ marginBottom: '0.5rem' }}>
          <label style={{ display: 'block' }}>{f.label}</label>
          <input 
            type={f.type || 'text'} 
            value={data[f.name] || ''} 
            onChange={e => handleChange(f.name, e.target.value)}
            required={f.required}
            style={{ width: '100%', padding: '0.4rem' }}
          />
        </div>
      ))}
      <div style={{ marginTop: '1rem' }}>
        <button type="submit" style={{ marginRight: '0.5rem' }}>Salvar</button>
        <button type="button" onClick={onCancel}>Cancelar</button>
      </div>
    </form>
  );
}

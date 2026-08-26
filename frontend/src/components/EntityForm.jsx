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
    <form onSubmit={handleSubmit} className="p-6 mb-6 bg-white border border-slate-200 rounded-lg shadow-sm">
      <h3 className="mb-4 text-lg font-semibold text-slate-900">{initialData.id ? 'Editar' : 'Criar'}</h3>
      <div className="space-y-4">
        {fields.map(f => (
          <div key={f.name}>
            <label className="block mb-1 text-sm font-medium text-slate-700">{f.label}</label>
            <input 
              type={f.type || 'text'} 
              value={data[f.name] || ''} 
              onChange={e => handleChange(f.name, e.target.value)}
              required={f.required}
              className="w-full px-3 py-2 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
            />
          </div>
        ))}
      </div>
      <div className="flex justify-end mt-6 space-x-3">
        <button 
          type="button" 
          onClick={onCancel}
          className="px-4 py-2 text-sm font-medium text-slate-700 bg-white border border-slate-300 rounded-md hover:bg-slate-50 focus:outline-none focus:ring-2 focus:ring-indigo-500"
        >
          Cancelar
        </button>
        <button 
          type="submit"
          className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500"
        >
          Salvar
        </button>
      </div>
    </form>
  );
}

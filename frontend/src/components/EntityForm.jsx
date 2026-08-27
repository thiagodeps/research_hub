import React, { useState } from 'react';

export default function EntityForm({ initialData = {}, fields, onSubmit, onCancel }) {
  const [data, setData] = useState(initialData);

  const handleChange = (name, value, type) => {
    let parsedValue = value;
    if (type === 'number' && value !== '') {
      parsedValue = Number(value);
    }
    setData(prev => ({ ...prev, [name]: parsedValue }));
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    onSubmit(data);
  };

  return (
    <form onSubmit={handleSubmit} className="p-6 mb-6 bg-white border border-slate-200 rounded-lg shadow-sm">
      <h3 className="mb-4 text-lg font-semibold text-slate-900">{initialData.id ? 'Editar' : 'Criar'}</h3>
      <div className="space-y-4">
        {fields.map(f => {
          if (f.type === 'json_readonly') {
            let items = [];
            try { 
              const parsed = data[f.name] ? JSON.parse(data[f.name]) : []; 
              items = Array.isArray(parsed) ? parsed : (parsed ? [parsed] : []);
            } catch(e) {}
            
            // Map column names to dashboard routes
            const getRoute = (colName) => {
              if (colName === 'research_groups') return 'groups';
              if (colName === 'campus') return 'campuses';
              if (colName === 'organizations') return 'organizations';
              return colName;
            };
            const route = getRoute(f.name);

            return (
              <div key={f.name}>
                <label className="block mb-1 text-sm font-medium text-slate-700">{f.label}</label>
                <div className="p-3 bg-slate-50 border border-slate-200 rounded max-h-48 overflow-y-auto text-sm">
                  {items.length > 0 ? (
                    <ul className="list-disc pl-5 space-y-1">
                      {items.map((item, idx) => (
                        <li key={idx}>
                          <a 
                            href={`/dashboard/${route}?openId=${item.id}`}
                            className="text-indigo-600 hover:text-indigo-800 hover:underline cursor-pointer"
                          >
                            ID: {item.id} - {item.name || item.title || 'Sem nome'}
                          </a>
                        </li>
                      ))}
                    </ul>
                  ) : <span className="text-slate-400">Nenhum vínculo.</span>}
                </div>
              </div>
            );
          }

          return (
            <div key={f.name}>
              <label className="block mb-1 text-sm font-medium text-slate-700">{f.label}</label>
              <input 
                type={f.type || 'text'} 
                value={data[f.name] || ''} 
                onChange={e => handleChange(f.name, e.target.value, f.type)}
                required={f.required}
                className="w-full px-3 py-2 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
              />
            </div>
          );
        })}
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

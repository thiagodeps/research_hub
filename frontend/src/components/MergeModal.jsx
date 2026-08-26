import React, { useState } from 'react';

export default function MergeModal({ entityName, selectedIds, onMerge, onCancel }) {
  const [resolvedName, setResolvedName] = useState('');

  const handleMerge = () => {
    onMerge(selectedIds, { name: resolvedName });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/50 backdrop-blur-sm">
      <div className="w-full max-w-md p-6 bg-white rounded-lg shadow-xl ring-1 ring-slate-900/5">
        <h3 className="mb-2 text-lg font-semibold text-slate-900">Fusão de {entityName}</h3>
        <p className="mb-4 text-sm text-slate-500">IDs selecionados: {selectedIds.join(', ')}</p>
        
        <div className="mb-6">
          <label className="block mb-2 text-sm font-medium text-slate-700">Nome final da entidade resultante:</label>
          <input 
            type="text" 
            value={resolvedName} 
            onChange={e => setResolvedName(e.target.value)} 
            className="w-full px-3 py-2 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
            placeholder="Digite o novo nome..."
          />
        </div>
        
        <div className="flex justify-end space-x-3">
          <button 
            onClick={onCancel}
            className="px-4 py-2 text-sm font-medium text-slate-700 bg-white border border-slate-300 rounded-md hover:bg-slate-50 focus:outline-none focus:ring-2 focus:ring-indigo-500"
          >
            Cancelar
          </button>
          <button 
            onClick={handleMerge}
            disabled={!resolvedName.trim()}
            className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Confirmar Fusão
          </button>
        </div>
      </div>
    </div>
  );
}

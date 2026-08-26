import React, { useState } from 'react';
import MergeModal from './MergeModal';
import LinkModal from './LinkModal';

export default function EntityTable({ entities, entityName, columns, onEdit, onDelete, onMerge, onLink }) {
  const [selectedIds, setSelectedIds] = useState([]);
  const [showMerge, setShowMerge] = useState(false);
  const [linkParent, setLinkParent] = useState(null);

  if (!entities || entities.length === 0) {
    return <p>Nenhum registro encontrado.</p>;
  }

  const toggleSelect = (id) => {
    setSelectedIds(prev => prev.includes(id) ? prev.filter(x => x !== id) : [...prev, id]);
  };

  return (
    <div className="mt-4">
      <div className="mb-4">
        <button 
          className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 rounded-md disabled:bg-slate-300 disabled:cursor-not-allowed hover:bg-indigo-700 transition-colors"
          disabled={selectedIds.length < 2} 
          onClick={() => setShowMerge(true)}
        >
          Fundir Selecionados
        </button>
      </div>

      {showMerge && (
        <MergeModal 
          entityName={entityName || 'entidades'}
          selectedIds={selectedIds}
          onMerge={(ids, data) => { onMerge(ids, data); setShowMerge(false); setSelectedIds([]); }}
          onCancel={() => setShowMerge(false)}
        />
      )}

      {linkParent && (
        <LinkModal 
          parentEntity={entityName || 'entidades'}
          parentId={linkParent}
          onLink={(pt, pid, ct, cid) => { onLink(pt, pid, ct, cid); setLinkParent(null); }}
          onCancel={() => setLinkParent(null)}
        />
      )}

      <div className="overflow-x-auto bg-white rounded-lg shadow ring-1 ring-slate-200">
        <table className="w-full text-sm text-left text-slate-500">
          <thead className="text-xs text-slate-700 uppercase bg-slate-50 border-b border-slate-200">
            <tr>
              <th scope="col" className="px-6 py-3 w-10">
                <span className="sr-only">Selecionar</span>
              </th>
              {columns.map(col => <th key={col} scope="col" className="px-6 py-3 font-semibold">{col}</th>)}
              <th scope="col" className="px-6 py-3 text-right font-semibold">Ações</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-200">
            {entities.map(entity => (
              <tr key={entity.id} className="hover:bg-slate-50 transition-colors">
                <td className="px-6 py-4">
                  <input 
                    type="checkbox" 
                    className="w-4 h-4 text-indigo-600 bg-gray-100 border-gray-300 rounded focus:ring-indigo-500"
                    checked={selectedIds.includes(entity.id)}
                    onChange={() => toggleSelect(entity.id)}
                  />
                </td>
                {columns.map(col => <td key={col} className="px-6 py-4 whitespace-nowrap text-slate-900">{entity[col]}</td>)}
                <td className="px-6 py-4 text-right space-x-3 whitespace-nowrap">
                  <button className="text-indigo-600 hover:text-indigo-900 font-medium transition-colors" onClick={() => onEdit(entity)}>Editar</button>
                  <button className="text-red-600 hover:text-red-900 font-medium transition-colors" onClick={() => onDelete(entity.id)}>Deletar</button>
                  <button className="text-slate-600 hover:text-slate-900 font-medium transition-colors" onClick={() => setLinkParent(entity.id)}>Vincular</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

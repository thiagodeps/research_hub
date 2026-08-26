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
    <div style={{ marginTop: '1rem' }}>
      <div style={{ marginBottom: '1rem' }}>
        <button 
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

      <table style={{ width: '100%', borderCollapse: 'collapse', marginTop: '1rem' }}>
        <thead>
          <tr style={{ borderBottom: '2px solid #ccc' }}>
            <th>#</th>
            {columns.map(col => <th key={col} style={{ textAlign: 'left', padding: '0.5rem' }}>{col}</th>)}
            <th>Ações</th>
          </tr>
        </thead>
        <tbody>
          {entities.map(entity => (
            <tr key={entity.id} style={{ borderBottom: '1px solid #eee' }}>
              <td>
                <input 
                  type="checkbox" 
                  checked={selectedIds.includes(entity.id)}
                  onChange={() => toggleSelect(entity.id)}
                />
              </td>
              {columns.map(col => <td key={col} style={{ padding: '0.5rem' }}>{entity[col]}</td>)}
              <td style={{ padding: '0.5rem' }}>
                <button onClick={() => onEdit(entity)}>Editar</button>
                <button onClick={() => onDelete(entity.id)}>Deletar</button>
                <button onClick={() => setLinkParent(entity.id)}>Vincular</button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

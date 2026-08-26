import React, { useState } from 'react';

export default function MergeModal({ entityName, selectedIds, onMerge, onCancel }) {
  const [resolvedName, setResolvedName] = useState('');

  const handleMerge = () => {
    onMerge(selectedIds, { name: resolvedName });
  };

  return (
    <div style={{ border: '1px solid #f00', padding: '1rem', marginTop: '1rem' }}>
      <h3>Fusão de {entityName}</h3>
      <p>IDs selecionados: {selectedIds.join(', ')}</p>
      
      <label>Nome final da entidade resultante:</label>
      <input 
        type="text" 
        value={resolvedName} 
        onChange={e => setResolvedName(e.target.value)} 
        style={{ display: 'block', margin: '0.5rem 0' }}
      />
      
      <button onClick={handleMerge}>Confirmar Fusão</button>
      <button onClick={onCancel} style={{ marginLeft: '1rem' }}>Cancelar</button>
    </div>
  );
}

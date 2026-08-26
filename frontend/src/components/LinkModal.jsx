import React, { useState } from 'react';

export default function LinkModal({ parentEntity, parentId, onLink, onCancel }) {
  const [childType, setChildType] = useState('article');
  const [childId, setChildId] = useState('');

  const handleLink = () => {
    onLink(parentEntity, parentId, childType, parseInt(childId, 10));
  };

  return (
    <div style={{ border: '1px solid #00f', padding: '1rem', marginTop: '1rem' }}>
      <h3>Vincular a {parentEntity} (ID: {parentId})</h3>
      
      <label>Tipo do item filho:</label>
      <select value={childType} onChange={e => setChildType(e.target.value)} style={{ display: 'block', margin: '0.5rem 0' }}>
        <option value="article">Artigo</option>
        <option value="research_group">Grupo de Pesquisa</option>
      </select>
      
      <label>ID do item filho:</label>
      <input 
        type="number" 
        value={childId} 
        onChange={e => setChildId(e.target.value)} 
        style={{ display: 'block', margin: '0.5rem 0' }}
      />
      
      <button onClick={handleLink}>Confirmar Vínculo</button>
      <button onClick={onCancel} style={{ marginLeft: '1rem' }}>Cancelar</button>
    </div>
  );
}

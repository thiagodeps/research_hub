import React, { useState } from 'react';

export default function LinkModal({ parentEntity, parentId, onLink, onCancel }) {
  const [childType, setChildType] = useState('articles');
  const [childId, setChildId] = useState('');

  const handleLink = () => {
    onLink(parentEntity, parentId, childType, parseInt(childId, 10));
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/50 backdrop-blur-sm">
      <div className="w-full max-w-md p-6 bg-white rounded-lg shadow-xl ring-1 ring-slate-900/5">
        <h3 className="mb-4 text-lg font-semibold text-slate-900">Vincular a {parentEntity} (ID: {parentId})</h3>
        
        <div className="mb-4">
          <label className="block mb-2 text-sm font-medium text-slate-700">Tipo do item filho:</label>
          <select 
            value={childType} 
            onChange={e => setChildType(e.target.value)} 
            className="w-full px-3 py-2 border border-slate-300 rounded-md bg-white focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
          >
            <option value="articles">Artigo</option>
            <option value="groups">Grupo de Pesquisa</option>
            <option value="initiatives">Iniciativas</option>
            <option value="advisorships">Orientações</option>
            <option value="awards">Premiações</option>
            <option value="students">Alunos</option>
            <option value="campuses">Campus</option>
            <option value="organizations">Organizações</option>
            <option value="fellowships">Bolsas</option>
            <option value="proficiencies">Proficiências</option>
            <option value="professional_activities">Ativ. Profissionais</option>
            <option value="knowledge_areas">Áreas de Conhecimento</option>
            <option value="languages">Idiomas</option>
            <option value="research_productions">Prod. Científicas</option>
          </select>
        </div>
        
        <div className="mb-6">
          <label className="block mb-2 text-sm font-medium text-slate-700">ID do item filho:</label>
          <input 
            type="number" 
            value={childId} 
            onChange={e => setChildId(e.target.value)} 
            className="w-full px-3 py-2 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
            placeholder="ID da entidade"
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
            onClick={handleLink}
            disabled={!childId}
            className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Confirmar Vínculo
          </button>
        </div>
      </div>
    </div>
  );
}

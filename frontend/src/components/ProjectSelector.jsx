import React, { useState } from 'react';

// SEP-033 / US1 (FR-001..FR-003): nova página principal pós-login.
// Sem `onSelect`, o componente navega para `project.href` (comportamento da
// página /projects); com `onSelect`, o chamador decide — assim o componente é
// testável isoladamente. `pendingWarning` ativa o aviso de mudança não salva
// antes de trocar de projeto — nunca perde trabalho silenciosamente.
/**
 * @param {{
 *   projects?: Array<{ id: string, title: string, description: string, href: string }>,
 *   onSelect?: (project: { id: string, title: string, description: string, href: string }) => void,
 *   pendingWarning?: boolean,
 * }} props
 */
export default function ProjectSelector({ projects = [], onSelect, pendingWarning = false }) {
  const [warnedId, setWarnedId] = useState(null);

  const go = (project) => {
    if (onSelect) {
      onSelect(project);
    } else {
      window.location.href = project.href;
    }
  };

  const choose = (project) => {
    if (pendingWarning) {
      setWarnedId(project.id);
      const proceed = window.confirm(
        'Há alterações não salvas neste projeto. Trocar de projeto agora pode perder o trabalho em andamento. Continuar?'
      );
      if (!proceed) return;
    }
    setWarnedId(null);
    go(project);
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-3xl w-full">
      {projects.map((project) => (
        <button
          key={project.id}
          type="button"
          onClick={() => choose(project)}
          className="text-left bg-white border border-slate-200 rounded-xl shadow-sm hover:shadow-md hover:border-sky-400 transition-all p-6 focus:outline-none focus:ring-2 focus:ring-sky-500"
        >
          <h2 className="text-xl font-bold text-slate-900">{project.title}</h2>
          <p className="mt-2 text-sm text-slate-600">{project.description}</p>
          <span className="mt-4 inline-block text-sm font-medium text-sky-700">
            Abrir projeto →
          </span>
        </button>
      ))}
    </div>
  );
}

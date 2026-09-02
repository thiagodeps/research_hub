import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';

// SEP-018. The file dialog is opened in Rust, so this component never touches
// the filesystem and the app declares no filesystem permission.
export default function DataControlCenter() {
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState(null);
  const [progress, setProgress] = useState([]);

  useEffect(() => {
    const unlisten = listen('import://progress', (event) => {
      setProgress((prev) => [...prev, event.payload]);
    });
    return () => { unlisten.then((off) => off()); };
  }, []);

  // Dropping the archive on the window follows the same path as the dialog.
  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'drop' && !busy) {
        const file = event.payload.paths?.[0];
        if (file) runImport(file);
      }
    });
    return () => { unlisten.then((off) => off()); };
  }, [busy]);

  async function runImport(path) {
    setBusy(true);
    setMessage(null);
    setProgress([]);
    try {
      const summary = await invoke('import_canonical_zip', { path: path ?? null });
      if (summary === null) {
        setBusy(false);
        return; // dialog dismissed
      }
      setMessage({
        type: 'success',
        text: `${summary.total_rows.toLocaleString('pt-BR')} registros em ${summary.tables.length} tabelas.`,
        snapshot: summary.snapshot,
      });
    } catch (err) {
      setMessage({ type: 'error', text: err?.message ?? String(err) });
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-8">
      <div className="bg-white p-6 rounded-lg shadow-sm border border-slate-200">
        <h2 className="text-xl font-semibold text-slate-800 mb-4">Importar Dados</h2>
        <p className="text-slate-600 mb-6 text-sm">
          Selecione o <code className="bg-slate-100 px-1 rounded">exports_canonical.zip</code> ou
          arraste-o para a janela. <strong>Atenção:</strong> isso substitui a base atual — uma
          cópia de segurança é gravada automaticamente antes.
        </p>

        <button
          onClick={() => runImport(null)}
          disabled={busy}
          className={`bg-blue-600 text-white px-4 py-2 rounded font-medium hover:bg-blue-700 transition-colors ${busy ? 'opacity-50 cursor-not-allowed' : ''}`}
        >
          {busy ? 'Importando...' : 'Escolher arquivo .ZIP'}
        </button>

        {progress.length > 0 && (
          <ul className="mt-4 max-h-40 overflow-y-auto text-xs text-slate-600 space-y-1">
            {progress.map((p, i) => (
              <li key={i} className="flex justify-between border-b border-slate-100 pb-1">
                <span>{p.table}</span>
                <span className="tabular-nums">{p.rows.toLocaleString('pt-BR')}</span>
              </li>
            ))}
          </ul>
        )}

        {message && (
          <div className={`mt-4 p-3 rounded text-sm ${message.type === 'error' ? 'bg-red-50 text-red-600' : 'bg-green-50 text-green-700'}`}>
            {message.text}
            {message.snapshot && (
              <div className="mt-1 text-xs text-slate-500 break-all">
                Cópia da base anterior: {message.snapshot}
              </div>
            )}
          </div>
        )}
      </div>

      <div className="bg-white p-6 rounded-lg shadow-sm border border-slate-200">
        <h2 className="text-xl font-semibold text-slate-800 mb-4">Exportar Base Curada</h2>
        <p className="text-slate-600 mb-6 text-sm">
          Gera o pacote canônico com as tabelas editadas, preservando intactos os demais
          arquivos do arquivo original.
        </p>
        <button
          disabled
          title="Disponível na próxima etapa da migração"
          className="bg-slate-200 text-slate-500 px-4 py-2 rounded font-medium cursor-not-allowed"
        >
          Exportar (em breve)
        </button>
      </div>
    </div>
  );
}

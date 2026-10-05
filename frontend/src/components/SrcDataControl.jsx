import React, { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { apiFetch } from '../services/api.js';

// SEP-033 / US2 (FR-006..FR-009, FR-014): import/export do JSON consolidado do
// SRC, espelhando o ritual do DataControlCenter do Horizon — progresso por
// evento, snapshot informado, diálogo dispensado é no-op, erro não destrói a
// base. A contagem é SEMPRE informada, inclusive base vazia (FR-009).
export default function SrcDataControl() {
  const [busy, setBusy] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [message, setMessage] = useState(null);
  const [progress, setProgress] = useState([]);

  useEffect(() => {
    const unlisten = listen('src-import://progress', (event) => {
      setProgress((prev) => [...prev, event.payload]);
    });
    return () => { unlisten.then((off) => off()); };
  }, []);

  // Dropping the file on the window follows the same path as the dialog.
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
      const summary = await apiFetch('/src/import', {
        method: 'POST',
        body: JSON.stringify({ path: path ?? null }),
      });
      if (summary === null) {
        setBusy(false);
        return; // dialog dismissed
      }
      setMessage({
        type: 'success',
        text:
          summary.total_acoes === 0
            ? 'Importação concluída: nenhuma ação no arquivo — a base SRC ficou vazia.'
            : `Importação concluída: ${summary.total_acoes.toLocaleString('pt-BR')} ações, ` +
              `${summary.total_participacoes.toLocaleString('pt-BR')} participações ` +
              `(${summary.acoes_com_participacoes.toLocaleString('pt-BR')} ações com participações).`,
        snapshot: summary.snapshot,
        unexpected: summary.unexpected_tipos?.length ? summary.unexpected_tipos : null,
      });
      // A página de resumo (e qualquer outro ouvinte) reage ao novo estado.
      window.dispatchEvent(new CustomEvent('src:imported', { detail: summary }));
    } catch (err) {
      setMessage({ type: 'error', text: err?.message ?? String(err) });
    } finally {
      setBusy(false);
    }
  }

  async function runExport() {
    setExporting(true);
    setMessage(null);
    try {
      const summary = await apiFetch('/src/export', { method: 'POST' });
      if (summary === null) return; // dialog dismissed
      setMessage({
        type: 'success',
        text:
          `Exportação concluída: ${summary.total_acoes.toLocaleString('pt-BR')} ações, ` +
          `${summary.total_participacoes.toLocaleString('pt-BR')} participações.`,
        snapshot: summary.path,
      });
    } catch (err) {
      setMessage({ type: 'error', text: err?.message ?? String(err) });
    } finally {
      setExporting(false);
    }
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-8">
      <div className="bg-white p-6 rounded-lg shadow-sm border border-slate-200">
        <h2 className="text-xl font-semibold text-slate-800 mb-4">Importar consolidado</h2>
        <p className="text-slate-600 mb-6 text-sm">
          Selecione o arquivo JSON consolidado do SRC ou arraste-o para a janela.
          <strong> Atenção:</strong> isso substitui a base SRC atual — um snapshot
          de segurança é gravado automaticamente antes.
        </p>

        <button
          onClick={() => runImport(null)}
          disabled={busy}
          className={`bg-blue-600 text-white px-4 py-2 rounded font-medium hover:bg-blue-700 transition-colors ${busy ? 'opacity-50 cursor-not-allowed' : ''}`}
        >
          {busy ? 'Importando...' : 'Escolher arquivo .JSON'}
        </button>

        {progress.length > 0 && (
          <ul className="mt-4 max-h-24 overflow-y-auto text-xs text-slate-500 space-y-1">
            {progress.map((p, i) => (
              <li key={i}>{typeof p === 'string' ? p : p?.detail}</li>
            ))}
          </ul>
        )}

        {message && (
          <div className={`mt-4 p-3 rounded text-sm ${message.type === 'error' ? 'bg-red-50 text-red-600' : 'bg-green-50 text-green-700'}`}>
            {message.text}
            {message.snapshot && (
              <div className="mt-1 text-xs text-slate-500 break-all">
                Arquivo: {message.snapshot}
              </div>
            )}
            {message.unexpected && (
              <div className="mt-1 text-xs text-amber-600">
                Valores inesperados em "tipo" (mantidos como vieram): {message.unexpected.join(', ')}
              </div>
            )}
          </div>
        )}
      </div>

      <div className="bg-white p-6 rounded-lg shadow-sm border border-slate-200">
        <h2 className="text-xl font-semibold text-slate-800 mb-4">Exportar base SRC</h2>
        <p className="text-slate-600 mb-6 text-sm">
          Gera o arquivo JSON consolidado com a base curada, na mesma estrutura
          do arquivo importado — pronto para o pipeline do SRC.
        </p>
        <button
          onClick={runExport}
          disabled={exporting || busy}
          className={`bg-emerald-600 text-white px-4 py-2 rounded font-medium hover:bg-emerald-700 transition-colors ${exporting || busy ? 'opacity-50 cursor-not-allowed' : ''}`}
        >
          {exporting ? 'Exportando...' : 'Exportar .JSON'}
        </button>
      </div>
    </div>
  );
}

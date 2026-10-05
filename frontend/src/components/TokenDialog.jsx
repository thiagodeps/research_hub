import React, { useState } from 'react';
import { apiFetch } from '../services/api.js';

export default function TokenDialog({
  isOpen = false,
  onClose,
  hasToken = false,
  tokenHint = null,
  onTokenChanged,
}) {
  const [tokenInput, setTokenInput] = useState('');
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState(null);
  const [testResult, setTestResult] = useState(null);

  if (!isOpen) return null;

  async function handleSaveToken(e) {
    if (e) e.preventDefault();
    const token = tokenInput.trim();
    if (!token) {
      setMessage({
        type: 'error',
        text: 'Informe o token de acesso.',
      });
      return;
    }

    setBusy(true);
    setMessage(null);
    setTestResult(null);

    try {
      await apiFetch('/github/token', {
        method: 'POST',
        body: JSON.stringify({ token }),
      });
      setTokenInput('');
      setMessage({
        type: 'success',
        text: 'Token salvo com sucesso.',
      });
      if (typeof onTokenChanged === 'function') {
        onTokenChanged();
      }
    } catch (err) {
      setMessage({
        type: 'error',
        text: err?.message ?? 'Falha ao salvar token.',
        kind: err?.kind,
      });
    } finally {
      setBusy(false);
    }
  }

  async function handleTestToken() {
    setBusy(true);
    setMessage(null);
    setTestResult(null);

    try {
      const res = await apiFetch('/github/token/test', {
        method: 'POST',
      });
      setTestResult(res);
      setMessage({
        type: 'success',
        text: 'Conexão testada com sucesso.',
      });
    } catch (err) {
      setMessage({
        type: 'error',
        text: err?.message ?? 'Falha ao testar token.',
        kind: err?.kind,
      });
    } finally {
      setBusy(false);
    }
  }

  async function handleRemoveToken() {
    const confirmed = window.confirm('Deseja realmente remover o token salvo?');
    if (!confirmed) return;

    setBusy(true);
    setMessage(null);
    setTestResult(null);

    try {
      await apiFetch('/github/token', {
        method: 'DELETE',
      });
      setMessage({
        type: 'success',
        text: 'Token removido com sucesso.',
      });
      if (typeof onTokenChanged === 'function') {
        onTokenChanged();
      }
    } catch (err) {
      setMessage({
        type: 'error',
        text: err?.message ?? 'Falha ao remover token.',
        kind: err?.kind,
      });
    } finally {
      setBusy(false);
    }
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Configuração de token GitHub"
      className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 backdrop-blur-sm p-4"
    >
      <div className="bg-white rounded-xl shadow-xl border border-slate-200 w-full max-w-lg overflow-hidden">
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-200">
          <h2 className="text-lg font-semibold text-slate-800">
            Token de acesso pessoal (GitHub)
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="text-slate-400 hover:text-slate-600 rounded-lg p-1 transition-colors"
            aria-label="Fechar"
          >
            ✕
          </button>
        </div>

        <div className="p-6 space-y-4">
          <p className="text-xs text-slate-500">
            Necessário para baixar dados de repositórios privados ou enviar dados ao GitHub.
            O token é armazenado com permissão restrita apenas neste computador.
          </p>

          {hasToken && (
            <div className="p-3 bg-slate-50 border border-slate-200 rounded-lg flex items-center justify-between">
              <div>
                <span className="text-xs font-medium text-slate-700 block">
                  Token configurado
                </span>
                <span className="text-xs font-mono text-slate-500">
                  Terminação: …{tokenHint ?? '****'}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={handleTestToken}
                  disabled={busy}
                  className="px-3 py-1.5 bg-blue-50 hover:bg-blue-100 text-blue-700 text-xs font-medium rounded-lg transition-colors border border-blue-200 disabled:opacity-50"
                >
                  {busy ? 'Testando…' : 'Testar conexão'}
                </button>
                <button
                  type="button"
                  onClick={handleRemoveToken}
                  disabled={busy}
                  className="px-3 py-1.5 bg-rose-50 hover:bg-rose-100 text-rose-700 text-xs font-medium rounded-lg transition-colors border border-rose-200 disabled:opacity-50"
                >
                  Remover token
                </button>
              </div>
            </div>
          )}

          {testResult && (
            <div className="p-3 bg-emerald-50 border border-emerald-200 rounded-lg text-xs text-emerald-800 space-y-1">
              <p className="font-semibold">Conexão válida!</p>
              <p>
                Usuário: <span className="font-mono">{testResult.login}</span>
              </p>
              <p>
                Escopos autorizados: <span className="font-mono">{testResult.scopes || '(nenhum)'}</span>
              </p>
            </div>
          )}

          {message && (
            <div
              className={`p-3 rounded-lg text-xs flex items-start justify-between ${
                message.type === 'success'
                  ? 'bg-emerald-50 text-emerald-800 border border-emerald-200'
                  : 'bg-rose-50 text-rose-800 border border-rose-200'
              }`}
            >
              <span>{message.text}</span>
              {message.kind && (
                <span className="ml-2 px-1.5 py-0.5 bg-rose-200 text-rose-900 rounded font-mono text-[10px]">
                  {message.kind}
                </span>
              )}
            </div>
          )}

          <form onSubmit={handleSaveToken} className="space-y-3 pt-2">
            <div>
              <label
                htmlFor="github-token-input"
                className="block text-xs font-medium text-slate-700 mb-1"
              >
                {hasToken ? 'Substituir token de acesso' : 'Novo token de acesso'}
              </label>
              <input
                id="github-token-input"
                name="token"
                type="password"
                aria-label="Token de acesso"
                placeholder="ghp_..."
                value={tokenInput}
                onChange={(e) => setTokenInput(e.target.value)}
                disabled={busy}
                autoComplete="off"
                className="w-full px-3 py-2 border border-slate-300 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-slate-50"
              />
            </div>

            <div className="flex justify-end gap-2 pt-2">
              <button
                type="button"
                onClick={onClose}
                disabled={busy}
                className="px-4 py-2 text-sm text-slate-600 hover:text-slate-800 font-medium rounded-lg"
              >
                Fechar
              </button>
              <button
                type="submit"
                disabled={busy || !tokenInput.trim()}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-300 text-white font-medium text-sm rounded-lg transition-colors shadow-sm"
              >
                {busy ? 'Salvando…' : 'Salvar token'}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}

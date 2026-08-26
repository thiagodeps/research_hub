import React, { useState } from 'react';

export default function DataControlCenter() {
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState(null);

  const handleFileUpload = async (event) => {
    const file = event.target.files[0];
    if (!file) return;

    if (!file.name.endsWith('.zip')) {
      setMessage({ type: 'error', text: 'Por favor, envie um arquivo .zip contendo os Parquets.' });
      return;
    }

    setLoading(true);
    setMessage(null);

    const formData = new FormData();
    formData.append('file', file);

    try {
      const token = localStorage.getItem('token');
      const response = await fetch('http://localhost:8000/api/v1/data/import', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`
        },
        body: formData
      });

      if (!response.ok) {
        throw new Error('Falha ao importar arquivo.');
      }
      
      const result = await response.json();
      setMessage({ type: 'success', text: result.message || 'Importação realizada com sucesso!' });
    } catch (err) {
      setMessage({ type: 'error', text: err.message });
    } finally {
      setLoading(false);
      // Reset input
      event.target.value = '';
    }
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-8">
      {/* Import Card */}
      <div className="bg-white p-6 rounded-lg shadow-sm border border-slate-200">
        <h2 className="text-xl font-semibold text-slate-800 mb-4">Importar Dados (Upload)</h2>
        <p className="text-slate-600 mb-6 text-sm">
          Envie o arquivo <code className="bg-slate-100 px-1 rounded">exports_canonical.zip</code> contendo as tabelas em formato Parquet para popular o banco de dados da aplicação. 
          <strong>Atenção:</strong> Isso substituirá a base atual.
        </p>

        <div className="flex items-center gap-4">
          <label className={`cursor-pointer bg-blue-600 text-white px-4 py-2 rounded font-medium hover:bg-blue-700 transition-colors ${loading ? 'opacity-50 pointer-events-none' : ''}`}>
            {loading ? 'Processando...' : 'Escolher Arquivo .ZIP'}
            <input 
              type="file" 
              accept=".zip" 
              className="hidden" 
              onChange={handleFileUpload} 
              disabled={loading}
            />
          </label>
        </div>

        {message && (
          <div className={`mt-4 p-3 rounded text-sm ${message.type === 'error' ? 'bg-red-50 text-red-600' : 'bg-green-50 text-green-600'}`}>
            {message.text}
          </div>
        )}
      </div>

      {/* Export Card */}
      <div className="bg-white p-6 rounded-lg shadow-sm border border-slate-200">
        <h2 className="text-xl font-semibold text-slate-800 mb-4">Exportar Base Curada</h2>
        <p className="text-slate-600 mb-6 text-sm">
          Baixe o estado atual de todas as tabelas editadas neste painel em formato Parquet.
          O arquivo zipado gerado estará pronto para ser ingerido pelo projeto analítico.
        </p>
        
        <a 
          href="http://localhost:8000/api/v1/data/export" 
          download="portal_export_canonical.zip"
          className="inline-block bg-emerald-600 text-white px-4 py-2 rounded font-medium hover:bg-emerald-700 transition-colors"
        >
          Exportar Arquivos .Parquet
        </a>
      </div>
    </div>
  );
}

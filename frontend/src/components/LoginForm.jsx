import React, { useState } from 'react';
import { apiFetch } from '../services/api';

export default function LoginForm() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState(null);
  const [registeredSuccess] = useState(() => {
    if (typeof window !== 'undefined' && window.location) {
      const params = new URLSearchParams(window.location.search);
      return params.get('registered') === 'true';
    }
    return false;
  });

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError(null);
    try {
      const response = await apiFetch('/auth/login', {
        method: 'POST',
        body: JSON.stringify({ email: email.trim(), password })
      });
      if (response.access_token) {
        localStorage.setItem('token', response.access_token);
        window.location.href = '/dashboard';
      }
    } catch (err) {
      setError(err.message || 'Erro ao realizar login.');
    }
  };

  return (
    <form onSubmit={handleSubmit} className="w-full max-w-sm p-8 bg-white rounded-lg shadow-lg">
      <h2 className="mb-6 text-2xl font-bold text-center text-slate-800">Bem-vindo Admin</h2>
      {registeredSuccess && (
        <div className="p-3 mb-4 text-sm text-green-800 bg-green-100 rounded-md">
          Conta criada com sucesso! Faça login com suas credenciais.
        </div>
      )}
      {error && <div className="p-3 mb-4 text-sm text-red-700 bg-red-100 rounded-md">{error}</div>}
      <div className="mb-4">
        <label htmlFor="email" className="block mb-1 text-sm font-medium text-slate-700">Email</label>
        <input 
          id="email"
          type="email" 
          value={email}
          onChange={e => setEmail(e.target.value)}
          required
          className="w-full px-4 py-2 border border-slate-300 rounded-md focus:ring-2 focus:ring-brand-500 focus:border-brand-500 outline-none transition-shadow"
        />
      </div>
      <div className="mb-6">
        <label htmlFor="password" className="block mb-1 text-sm font-medium text-slate-700">Senha</label>
        <input 
          id="password"
          type="password" 
          value={password}
          onChange={e => setPassword(e.target.value)}
          required
          className="w-full px-4 py-2 border border-slate-300 rounded-md focus:ring-2 focus:ring-brand-500 focus:border-brand-500 outline-none transition-shadow"
        />
      </div>
      <button type="submit" className="w-full px-4 py-2 text-white bg-slate-900 hover:bg-slate-800 rounded-md transition-colors focus:ring-2 focus:ring-slate-900 focus:ring-offset-2 outline-none font-medium">
        Entrar
      </button>

      <div className="mt-4 text-center">
        <a href="/register" className="text-sm font-medium text-slate-600 hover:text-slate-900 underline">
          Não tem uma conta? Cadastre-se
        </a>
      </div>
    </form>
  );
}


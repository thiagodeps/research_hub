import React, { useState } from 'react';
import { apiFetch } from '../services/api';

export default function RegisterForm() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [passwordConfirm, setPasswordConfirm] = useState('');
  const [error, setError] = useState(null);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError(null);

    if (password.length < 8) {
      setError('A senha deve conter no mínimo 8 caracteres.');
      return;
    }

    if (password !== passwordConfirm) {
      setError('A confirmação de senha não coincide com a senha digitada.');
      return;
    }

    setLoading(true);
    try {
      await apiFetch('/auth/register', {
        method: 'POST',
        body: JSON.stringify({
          email: email.trim(),
          password,
          password_confirm: passwordConfirm,
        }),
      });

      window.location.href = '/login?registered=true';
    } catch (err) {
      setError(err.message || 'Erro ao criar conta.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="w-full max-w-sm p-8 bg-white rounded-lg shadow-lg">
      <h2 className="mb-6 text-2xl font-bold text-center text-slate-800">Criar Conta</h2>
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

      <div className="mb-4">
        <label htmlFor="password" className="block mb-1 text-sm font-medium text-slate-700">Senha</label>
        <input 
          id="password"
          type="password" 
          value={password}
          onChange={e => setPassword(e.target.value)}
          required
          minLength={8}
          placeholder="Mínimo 8 caracteres"
          className="w-full px-4 py-2 border border-slate-300 rounded-md focus:ring-2 focus:ring-brand-500 focus:border-brand-500 outline-none transition-shadow"
        />
      </div>

      <div className="mb-6">
        <label htmlFor="passwordConfirm" className="block mb-1 text-sm font-medium text-slate-700">Confirmar Senha</label>
        <input 
          id="passwordConfirm"
          type="password" 
          value={passwordConfirm}
          onChange={e => setPasswordConfirm(e.target.value)}
          required
          className="w-full px-4 py-2 border border-slate-300 rounded-md focus:ring-2 focus:ring-brand-500 focus:border-brand-500 outline-none transition-shadow"
        />
      </div>

      <button 
        type="submit" 
        disabled={loading}
        className="w-full px-4 py-2 text-white bg-slate-900 hover:bg-slate-800 rounded-md transition-colors focus:ring-2 focus:ring-slate-900 focus:ring-offset-2 outline-none font-medium disabled:opacity-50"
      >
        {loading ? 'Cadastrando...' : 'Criar Conta'}
      </button>

      <div className="mt-4 text-center">
        <a href="/login" className="text-sm font-medium text-slate-600 hover:text-slate-900 underline">
          Já tem uma conta? Entrar
        </a>
      </div>
    </form>
  );
}

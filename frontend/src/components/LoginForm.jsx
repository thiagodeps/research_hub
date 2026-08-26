import React, { useState } from 'react';
import { apiFetch } from '../services/api';

export default function LoginForm() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState(null);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError(null);
    try {
      const response = await apiFetch('/auth/login', {
        method: 'POST',
        body: JSON.stringify({ email, password })
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
    <form onSubmit={handleSubmit} style={{ width: '100%', maxWidth: '300px' }}>
      <h2>Bem-vindo Admin</h2>
      {error && <div style={{ color: 'red', marginBottom: '1rem' }}>{error}</div>}
      <div style={{ marginBottom: '1rem' }}>
        <label style={{ display: 'block' }}>Email</label>
        <input 
          type="email" 
          value={email}
          onChange={e => setEmail(e.target.value)}
          required
          style={{ width: '100%', padding: '0.5rem' }}
        />
      </div>
      <div style={{ marginBottom: '1rem' }}>
        <label style={{ display: 'block' }}>Senha</label>
        <input 
          type="password" 
          value={password}
          onChange={e => setPassword(e.target.value)}
          required
          style={{ width: '100%', padding: '0.5rem' }}
        />
      </div>
      <button type="submit" style={{ width: '100%', padding: '0.5rem', backgroundColor: '#0f172a', color: 'white', border: 'none' }}>
        Entrar
      </button>
    </form>
  );
}

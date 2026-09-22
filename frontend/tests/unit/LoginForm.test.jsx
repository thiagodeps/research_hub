/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';

// Mock apiFetch
const mockApiFetch = vi.fn();
vi.mock('../../src/services/api', () => ({
  apiFetch: (...args) => mockApiFetch(...args),
}));

import LoginForm from '../../src/components/LoginForm.jsx';

describe('LoginForm Component', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
    try {
      delete window.location;
      window.location = { href: '', search: '' };
    } catch {
      Object.defineProperty(window, 'location', {
        value: { href: '', search: '' },
        writable: true,
        configurable: true,
      });
    }
    localStorage.clear();
  });

  it('renders login fields, submit button, and link to register', () => {
    render(<LoginForm />);
    expect(screen.getByRole('heading', { name: /bem-vindo admin/i })).toBeDefined();
    expect(screen.getByLabelText(/^email/i)).toBeDefined();
    expect(screen.getByLabelText(/^senha/i)).toBeDefined();
    expect(screen.getByRole('button', { name: /entrar/i })).toBeDefined();
    expect(screen.getByRole('link', { name: /cadastre-se/i })).toBeDefined();
  });

  it('displays success alert when ?registered=true is present in url search', () => {
    window.location.search = '?registered=true';
    render(<LoginForm />);
    expect(screen.getByText(/conta criada com sucesso/i)).toBeDefined();
  });

  it('calls apiFetch with trimmed email and sets token on successful login', async () => {
    mockApiFetch.mockResolvedValue({ access_token: 'fake-jwt-token' });
    render(<LoginForm />);

    fireEvent.change(screen.getByLabelText(/^email/i), { target: { value: '  prof@ifes.edu.br  ' } });
    fireEvent.change(screen.getByLabelText(/^senha/i), { target: { value: 'MinhaSenha123' } });
    fireEvent.click(screen.getByRole('button', { name: /entrar/i }));

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/auth/login', {
        method: 'POST',
        body: JSON.stringify({
          email: 'prof@ifes.edu.br',
          password: 'MinhaSenha123',
        }),
      });
      expect(localStorage.getItem('token')).toBe('fake-jwt-token');
      expect(window.location.href).toBe('/dashboard');
    });
  });

  it('displays error alert on login failure', async () => {
    mockApiFetch.mockRejectedValue(new Error('Credenciais inválidas'));
    render(<LoginForm />);

    fireEvent.change(screen.getByLabelText(/^email/i), { target: { value: 'admin@admin.com' } });
    fireEvent.change(screen.getByLabelText(/^senha/i), { target: { value: 'errada' } });
    fireEvent.click(screen.getByRole('button', { name: /entrar/i }));

    expect(await screen.findByText(/credenciais inválidas/i)).toBeDefined();
  });
});

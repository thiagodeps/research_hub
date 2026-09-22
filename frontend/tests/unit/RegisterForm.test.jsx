/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react';

// Mock apiFetch
const mockApiFetch = vi.fn();
vi.mock('../../src/services/api', () => ({
  apiFetch: (...args) => mockApiFetch(...args),
}));

import RegisterForm from '../../src/components/RegisterForm.jsx';

describe('RegisterForm Component', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
    window.history.pushState({}, '', '/register');
  });

  afterEach(() => {
    cleanup();
  });

  it('renders all registration fields and submit button', () => {
    render(<RegisterForm />);
    expect(screen.getByRole('heading', { name: /criar conta/i })).toBeDefined();
    expect(screen.getByLabelText(/^email/i)).toBeDefined();
    expect(screen.getByLabelText(/^senha/i)).toBeDefined();
    expect(screen.getByLabelText(/confirmar senha/i)).toBeDefined();
    expect(screen.getByRole('button', { name: /cadastrar|criar conta/i })).toBeDefined();
    expect(screen.getByRole('link', { name: /entrar/i })).toBeDefined();
  });

  it('validates password length on client before submission', async () => {
    render(<RegisterForm />);
    fireEvent.change(screen.getByLabelText(/^email/i), { target: { value: 'user@ifes.edu.br' } });
    fireEvent.change(screen.getByLabelText(/^senha/i), { target: { value: '12345' } });
    fireEvent.change(screen.getByLabelText(/confirmar senha/i), { target: { value: '12345' } });
    fireEvent.click(screen.getByRole('button', { name: /cadastrar|criar conta/i }));

    expect(await screen.findByText(/no mínimo 8 caracteres/i)).toBeDefined();
    expect(mockApiFetch).not.toHaveBeenCalled();
  });

  it('validates matching passwords before submission', async () => {
    render(<RegisterForm />);
    fireEvent.change(screen.getByLabelText(/^email/i), { target: { value: 'user@ifes.edu.br' } });
    fireEvent.change(screen.getByLabelText(/^senha/i), { target: { value: 'SenhaForte123' } });
    fireEvent.change(screen.getByLabelText(/confirmar senha/i), { target: { value: 'OutraSenha123' } });
    fireEvent.click(screen.getByRole('button', { name: /cadastrar|criar conta/i }));

    expect(await screen.findByText(/não coincide/i)).toBeDefined();
    expect(mockApiFetch).not.toHaveBeenCalled();
  });

  it('calls apiFetch with trimmed email and redirects to login on success', async () => {
    mockApiFetch.mockResolvedValue(null);
    render(<RegisterForm />);
    fireEvent.change(screen.getByLabelText(/^email/i), { target: { value: '  user@ifes.edu.br  ' } });
    fireEvent.change(screen.getByLabelText(/^senha/i), { target: { value: 'SenhaForte123' } });
    fireEvent.change(screen.getByLabelText(/confirmar senha/i), { target: { value: 'SenhaForte123' } });
    fireEvent.click(screen.getByRole('button', { name: /cadastrar|criar conta/i }));

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/auth/register', {
        method: 'POST',
        body: JSON.stringify({
          email: 'user@ifes.edu.br',
          password: 'SenhaForte123',
          password_confirm: 'SenhaForte123',
        }),
      });
    });
  });

  it('displays API error message on failure', async () => {
    mockApiFetch.mockRejectedValue(new Error('Este email já está cadastrado.'));
    render(<RegisterForm />);
    fireEvent.change(screen.getByLabelText(/^email/i), { target: { value: 'admin@admin.com' } });
    fireEvent.change(screen.getByLabelText(/^senha/i), { target: { value: 'SenhaForte123' } });
    fireEvent.change(screen.getByLabelText(/confirmar senha/i), { target: { value: 'SenhaForte123' } });
    fireEvent.click(screen.getByRole('button', { name: /cadastrar|criar conta/i }));

    expect(await screen.findByText(/este email já está cadastrado/i)).toBeDefined();
  });
});

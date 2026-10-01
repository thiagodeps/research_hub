/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react';

const mockApiFetch = vi.fn();

vi.mock('../../src/services/api', () => ({
  apiFetch: (...args) => mockApiFetch(...args),
}));

import TokenDialog from '../../src/components/TokenDialog.jsx';

describe('TokenDialog (US3)', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
  });

  afterEach(() => {
    cleanup();
  });

  it('does not render when isOpen is false', () => {
    render(<TokenDialog isOpen={false} onClose={vi.fn()} />);
    expect(screen.queryByRole('heading', { name: /token de acesso/i })).toBeNull();
  });

  it('renders password input and save button when open', () => {
    render(<TokenDialog isOpen={true} onClose={vi.fn()} hasToken={false} />);
    expect(screen.getByRole('heading', { name: /token de acesso/i })).toBeDefined();

    const input = screen.getByLabelText(/token de acesso/i);
    expect(input.getAttribute('type')).toBe('password');
    expect(screen.getByRole('button', { name: /salvar token/i })).toBeDefined();
  });

  it('saves token without displaying raw token anywhere in the UI', async () => {
    mockApiFetch.mockResolvedValueOnce(null);
    const onTokenChanged = vi.fn();

    render(
      <TokenDialog
        isOpen={true}
        onClose={vi.fn()}
        hasToken={false}
        onTokenChanged={onTokenChanged}
      />
    );

    const input = screen.getByLabelText(/token de acesso/i);
    fireEvent.change(input, { target: { value: 'ghp_supersecret12345' } });

    fireEvent.click(screen.getByRole('button', { name: /salvar token/i }));

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/token', {
        method: 'POST',
        body: JSON.stringify({ token: 'ghp_supersecret12345' }),
      });
    });

    await waitFor(() => {
      expect(screen.getByText(/token salvo com sucesso/i)).toBeDefined();
    });

    expect(screen.queryByText('ghp_supersecret12345')).toBeNull();
    expect(input.value).toBe('');
    expect(onTokenChanged).toHaveBeenCalled();
  });

  it('tests active token and displays login and scopes', async () => {
    mockApiFetch.mockResolvedValueOnce({
      login: 'curador_ifes',
      scopes: 'repo, read:user',
      valid: true,
    });

    render(
      <TokenDialog
        isOpen={true}
        onClose={vi.fn()}
        hasToken={true}
        tokenHint="cdef"
      />
    );

    expect(screen.getByText(/cdef/)).toBeDefined();

    fireEvent.click(screen.getByRole('button', { name: /testar conexão/i }));

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/token/test', {
        method: 'POST',
      });
    });

    await waitFor(() => {
      expect(screen.getByText(/curador_ifes/i)).toBeDefined();
      expect(screen.getByText(/repo, read:user/i)).toBeDefined();
    });
  });

  it('removes token with confirmation', async () => {
    window.confirm = vi.fn().mockReturnValue(true);
    mockApiFetch.mockResolvedValueOnce(null);
    const onTokenChanged = vi.fn();

    render(
      <TokenDialog
        isOpen={true}
        onClose={vi.fn()}
        hasToken={true}
        tokenHint="abcd"
        onTokenChanged={onTokenChanged}
      />
    );

    fireEvent.click(screen.getByRole('button', { name: /remover token/i }));

    expect(window.confirm).toHaveBeenCalled();

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/token', {
        method: 'DELETE',
      });
    });

    await waitFor(() => {
      expect(screen.getByText(/token removido com sucesso/i)).toBeDefined();
    });

    expect(onTokenChanged).toHaveBeenCalled();
  });

  it('does not remove token if user cancels confirmation', async () => {
    window.confirm = vi.fn().mockReturnValue(false);

    render(
      <TokenDialog
        isOpen={true}
        onClose={vi.fn()}
        hasToken={true}
        tokenHint="abcd"
      />
    );

    fireEvent.click(screen.getByRole('button', { name: /remover token/i }));

    expect(window.confirm).toHaveBeenCalled();
    expect(mockApiFetch).not.toHaveBeenCalledWith('/github/token', expect.anything());
  });

  it('displays typed error on test failure', async () => {
    const error = new Error('Token inválido ou revogado; salve um token atualizado');
    error.kind = 'github_api';

    mockApiFetch.mockRejectedValueOnce(error);

    render(
      <TokenDialog
        isOpen={true}
        onClose={vi.fn()}
        hasToken={true}
        tokenHint="abcd"
      />
    );

    fireEvent.click(screen.getByRole('button', { name: /testar conexão/i }));

    await waitFor(() => {
      expect(screen.getByText(/token inválido ou revogado/i)).toBeDefined();
    });
    expect(screen.getByText(/github_api/i)).toBeDefined();
  });
});

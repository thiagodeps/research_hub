/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react';

const mockApiFetch = vi.fn();
vi.mock('../../src/services/api', () => ({ apiFetch: (...args) => mockApiFetch(...args) }));

// listen / onDragDropEvent are Tauri APIs; unit tests stub them out.
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));
vi.mock('@tauri-apps/api/webview', () => ({
  getCurrentWebview: () => ({ onDragDropEvent: vi.fn().mockResolvedValue(() => {}) }),
}));

import SrcAcoesPage from '../../src/components/SrcAcoesPage.jsx';

// SEP-033 / US3 (FR-010/FR-011): curadoria das ações no caminho dedicado /src,
// com busca, ordenação, exclusão com conflito → confirmação forçada e editor
// de participações embutido.

const acoes = {
  items: [
    {
      id: 1, acao_id: '1001', processo: '0123.456/2025',
      titulo: 'Alfabetização Digital no Campo', natureza: 'Extensão',
      tipo: 'Projeto', coordenador: 'Maria Aparecida Souza',
      acao_vinculante: '', total_participacoes: 3,
    },
    {
      id: 3, acao_id: '1003', processo: '0123.999/2025',
      titulo: 'Semana de Extensão Serra', natureza: 'Extensão',
      tipo: 'Evento', coordenador: null,
      acao_vinculante: '0123.456/2025', total_participacoes: 0,
    },
  ],
  total: 2,
};

describe('SrcAcoesPage', () => {
  beforeEach(() => mockApiFetch.mockReset());
  afterEach(() => cleanup());

  it('lists ações from the dedicated SRC route', async () => {
    mockApiFetch.mockResolvedValue(acoes);
    render(<SrcAcoesPage />);
    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith(
        expect.stringMatching(/^\/src\/acoes\?limit=/)
      )
    );
    expect(await screen.findByText('Alfabetização Digital no Campo')).toBeDefined();
  });

  it('opens the editor with participações when an existing ação is edited', async () => {
    mockApiFetch
      .mockResolvedValueOnce(acoes) // lista
      .mockResolvedValueOnce({ ...acoes.items[1], resumo: null }) // get_acao 1003
      .mockResolvedValueOnce([]) // participações
      .mockResolvedValueOnce({ items: [{ processo: '0123.456/2025', titulo: 'Alfabetização Digital no Campo' }], total: 1 }); // busca do pai
    render(<SrcAcoesPage />);
    await waitFor(() => expect(screen.getAllByRole('button', { name: /visualizar/i }).length).toBe(2));

    fireEvent.click(screen.getAllByRole('button', { name: /visualizar/i })[1]);
    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith('/src/acoes/3')
    );
    // Editor de participações montado para a ação aberta:
    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith('/src/acoes/3/participacoes')
    );
    // Vínculo da 1003 resolve na 1001 → sem aviso de vínculo quebrado.
    await waitFor(() =>
      expect(screen.queryByText(/vínculo não encontrado/i)).toBeNull()
    );
  });

  it('retries the delete with force after the user confirms the conflict', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
    const conflict = new Error(
      'existem 1 ação(ões) vinculada(s) a esta (acao_id: 1003). Desvincule-as primeiro ou confirme a exclusão forçada.'
    );
    mockApiFetch
      .mockResolvedValueOnce(acoes) // lista
      .mockRejectedValueOnce(conflict) // DELETE simples → Conflict
      .mockResolvedValueOnce(null) // DELETE force=true
      .mockResolvedValueOnce(acoes); // recarga
    render(<SrcAcoesPage />);
    await waitFor(() => expect(screen.getAllByRole('button', { name: /deletar/i }).length).toBe(2));

    fireEvent.click(screen.getAllByRole('button', { name: /deletar/i })[0]);
    await waitFor(() => expect(confirmSpy).toHaveBeenCalled());
    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith('/src/acoes/1?force=true', { method: 'DELETE' })
    );
  });

  it('shows the conflict message and does not retry when the user declines', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);
    const conflict = new Error('existem 1 ação(ões) vinculada(s) (acao_id: 1003).');
    mockApiFetch
      .mockResolvedValueOnce(acoes)
      .mockRejectedValueOnce(conflict);
    render(<SrcAcoesPage />);
    await waitFor(() => expect(screen.getAllByRole('button', { name: /deletar/i }).length).toBe(2));

    fireEvent.click(screen.getAllByRole('button', { name: /deletar/i })[0]);
    await waitFor(() => expect(screen.getByText(/vinculada/i)).toBeDefined());
    expect(mockApiFetch).toHaveBeenCalledTimes(2);
  });

  it('announces unsaved-changes state while the form is open (T039)', async () => {
    mockApiFetch.mockResolvedValue(acoes);
    const events = [];
    const listener = (e) => events.push(e.detail);
    window.addEventListener('src:dirty', listener);

    render(<SrcAcoesPage />);
    fireEvent.click(screen.getByRole('button', { name: /novo registro/i }));
    await waitFor(() => expect(events[events.length - 1]).toBe(true));

    fireEvent.click(screen.getByRole('button', { name: /cancelar/i }));
    await waitFor(() => expect(events[events.length - 1]).toBe(false));
    window.removeEventListener('src:dirty', listener);
  });
});

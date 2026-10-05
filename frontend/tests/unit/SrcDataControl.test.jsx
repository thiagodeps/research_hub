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

import SrcDataControl from '../../src/components/SrcDataControl.jsx';

// SEP-033 / US2: import com contagens + snapshot informado (FR-006/009),
// export com caminho, erros sem perder estado, diálogo dispensado é no-op.
describe('SrcDataControl', () => {
  beforeEach(() => mockApiFetch.mockReset());
  afterEach(() => cleanup());

  it('renders the import and export controls', () => {
    render(<SrcDataControl />);
    expect(screen.getByRole('button', { name: /escolher arquivo .json/i })).toBeDefined();
    expect(screen.getByRole('button', { name: /exportar .json/i })).toBeDefined();
  });

  it('reports counts and snapshot after a successful import', async () => {
    mockApiFetch.mockResolvedValueOnce({
      total_acoes: 3,
      total_participacoes: 3,
      acoes_com_participacoes: 1,
      replaced: true,
      snapshot: '/dados/snapshots/src-1700.db',
      unexpected_tipos: [],
    });
    render(<SrcDataControl />);
    fireEvent.click(screen.getByRole('button', { name: /escolher arquivo .json/i }));

    await waitFor(() => expect(screen.getByText(/3 ações/i)).toBeDefined());
    expect(screen.getByText(/3 participações/i)).toBeDefined();
    expect(screen.getByText(/src-1700\.db/)).toBeDefined();
    expect(mockApiFetch).toHaveBeenCalledWith('/src/import', {
      method: 'POST',
      body: JSON.stringify({ path: null }),
    });
  });

  it('announces the new base state so the summary page can react (T028)', async () => {
    const listener = vi.fn();
    window.addEventListener('src:imported', listener);
    mockApiFetch.mockResolvedValueOnce({
      total_acoes: 3, total_participacoes: 3, acoes_com_participacoes: 1,
      replaced: true, snapshot: null, unexpected_tipos: [],
    });
    render(<SrcDataControl />);
    fireEvent.click(screen.getByRole('button', { name: /escolher arquivo .json/i }));
    await waitFor(() => expect(listener).toHaveBeenCalledTimes(1));
    expect(listener.mock.calls[0][0].detail.total_acoes).toBe(3);
    window.removeEventListener('src:imported', listener);
  });

  it('reports the empty base instead of pretending it loaded (FR-009)', async () => {
    mockApiFetch.mockResolvedValueOnce({
      total_acoes: 0,
      total_participacoes: 0,
      acoes_com_participacoes: 0,
      replaced: true,
      snapshot: null,
      unexpected_tipos: [],
    });
    render(<SrcDataControl />);
    fireEvent.click(screen.getByRole('button', { name: /escolher arquivo .json/i }));
    await waitFor(() => expect(screen.getByText(/nenhuma ação/i)).toBeDefined());
  });

  it('shows unexpected tipo values flagged by the import (C6)', async () => {
    mockApiFetch.mockResolvedValueOnce({
      total_acoes: 1,
      total_participacoes: 1,
      acoes_com_participacoes: 1,
      replaced: false,
      snapshot: null,
      unexpected_tipos: ['Ouvinte'],
    });
    render(<SrcDataControl />);
    fireEvent.click(screen.getByRole('button', { name: /escolher arquivo .json/i }));
    await waitFor(() => expect(screen.getByText(/Ouvinte/)).toBeDefined());
  });

  it('keeps the message area and shows the error when the import fails (FR-007)', async () => {
    mockApiFetch.mockRejectedValueOnce(new Error('acao_id duplicado no arquivo: 1001'));
    render(<SrcDataControl />);
    fireEvent.click(screen.getByRole('button', { name: /escolher arquivo .json/i }));
    await waitFor(() => expect(screen.getByText(/duplicado/)).toBeDefined());
    // A base anterior continua lá — nada foi apagado; o botão volta ao normal.
    expect(screen.getByRole('button', { name: /escolher arquivo .json/i })).toBeDefined();
  });

  it('reports the exported path (FR-014)', async () => {
    mockApiFetch.mockResolvedValueOnce({
      total_acoes: 3,
      total_participacoes: 3,
      path: '/tmp/saida.json',
    });
    render(<SrcDataControl />);
    fireEvent.click(screen.getByRole('button', { name: /exportar .json/i }));
    await waitFor(() => expect(screen.getByText(/\/tmp\/saida\.json/)).toBeDefined());
  });

  it('treats a dismissed dialog as a no-op', async () => {
    mockApiFetch.mockResolvedValueOnce(null);
    render(<SrcDataControl />);
    fireEvent.click(screen.getByRole('button', { name: /escolher arquivo .json/i }));
    await waitFor(() => expect(mockApiFetch).toHaveBeenCalled());
    expect(screen.queryByText(/ações/i)).toBeNull();
  });
});

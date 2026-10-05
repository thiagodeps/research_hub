/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react';

const mockApiFetch = vi.fn();
vi.mock('../../src/services/api', () => ({ apiFetch: (...args) => mockApiFetch(...args) }));

import ParticipacoesEditor from '../../src/components/ParticipacoesEditor.jsx';

// SEP-033 / US3 (FR-012): coleção aninhada da ação — agrupada por tipo e
// contexto de atividade, campos da pessoa com os rótulos do arquivo (C5),
// mutações com confirmação.

const participacoes = [
  {
    id: 1, ord: 1, tipo: 'Público-alvo', atividade_num: '1',
    atividade_id: '9001', atividade: 'Turma da manhã',
    nome: 'João Pedro Alves', Nome: 'João Pedro Alves',
    CPF: '123.456.789-00', 'E-mail': 'joao@example.com', Situação: 'APROVADO',
  },
  {
    id: 2, ord: 2, tipo: 'Público-alvo', atividade_num: '2',
    atividade_id: '9002', atividade: 'Turma da tarde',
    nome: 'Ana Beatriz Lima', Nome: 'Ana Beatriz Lima',
  },
  {
    id: 3, ord: 3, tipo: 'Equipe de execução', atividade_num: '2',
    atividade_id: '9002', atividade: 'Turma da tarde',
    nome: 'Carlos Eduardo Rocha', Nome: 'Carlos Eduardo Rocha',
    Função: 'Bolsista', Vínculo: 'Estudante',
  },
];

describe('ParticipacoesEditor', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
    mockApiFetch.mockResolvedValue(participacoes);
  });
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it('lists participações grouped by tipo and activity context with file labels', async () => {
    render(<ParticipacoesEditor acaoId={7} vinculo={{ text: '', resolvido: true }} />);

    await waitFor(() => expect(mockApiFetch).toHaveBeenCalledWith('/src/acoes/7/participacoes'));
    // Agrupamento por tipo:
    expect(screen.getByRole('heading', { name: /público-alvo/i })).toBeDefined();
    expect(screen.getByRole('heading', { name: /equipe de execução/i })).toBeDefined();
    // Contexto de atividade como sub-cabeçalho:
    expect(screen.getByText(/turma da manhã/i)).toBeDefined();
    // Campos da pessoa pelos rótulos do arquivo:
    expect(screen.getByText('João Pedro Alves')).toBeDefined();
    expect(screen.getByText('123.456.789-00')).toBeDefined();
    expect(screen.getByText('joao@example.com')).toBeDefined();
    expect(screen.getByText('Bolsista')).toBeDefined();
  });

  it('adds a participação posting the form payload and refreshes', async () => {
    mockApiFetch
      .mockResolvedValueOnce(participacoes) // carga inicial
      .mockResolvedValueOnce(9) // POST
      .mockResolvedValueOnce(participacoes); // recarga
    render(<ParticipacoesEditor acaoId={7} vinculo={{ text: '', resolvido: true }} />);
    await waitFor(() => expect(screen.getByLabelText(/^tipo/i)).toBeDefined());

    fireEvent.change(screen.getByLabelText(/^tipo/i), { target: { value: 'Público-alvo' } });
    fireEvent.change(screen.getByLabelText(/^nome/i), { target: { value: 'Nova Pessoa' } });
    fireEvent.change(screen.getByLabelText(/^cpf/i), { target: { value: '111.222.333-44' } });
    fireEvent.click(screen.getByRole('button', { name: /adicionar/i }));

    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith('/src/acoes/7/participacoes', {
        method: 'POST',
        body: JSON.stringify({
          tipo: 'Público-alvo',
          Nome: 'Nova Pessoa',
          CPF: '111.222.333-44',
        }),
      })
    );
    expect(mockApiFetch).toHaveBeenCalledTimes(3); // carga + POST + recarga
  });

  it('edits an existing participação in place', async () => {
    mockApiFetch
      .mockResolvedValueOnce(participacoes)
      .mockResolvedValueOnce(null) // PUT
      .mockResolvedValueOnce(participacoes); // recarga
    render(<ParticipacoesEditor acaoId={7} vinculo={{ text: '', resolvido: true }} />);
    await waitFor(() => expect(screen.getAllByRole('button', { name: /editar/i }).length).toBe(3));

    fireEvent.click(screen.getAllByRole('button', { name: /editar/i })[0]);
    expect(screen.getByLabelText(/^nome/i).value).toBe('João Pedro Alves');

    fireEvent.change(screen.getByLabelText(/^nome/i), { target: { value: 'Nome Corrigido' } });
    fireEvent.click(screen.getByRole('button', { name: /salvar edição/i }));

    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith('/src/participacoes/1', {
        method: 'PUT',
        body: expect.stringContaining('Nome Corrigido'),
      })
    );
  });

  it('removes with confirmation', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
    mockApiFetch
      .mockResolvedValueOnce(participacoes)
      .mockResolvedValueOnce(null) // DELETE
      .mockResolvedValueOnce(participacoes); // recarga
    render(<ParticipacoesEditor acaoId={7} vinculo={{ text: '', resolvido: true }} />);
    await waitFor(() => expect(screen.getAllByRole('button', { name: /remover/i }).length).toBe(3));

    fireEvent.click(screen.getAllByRole('button', { name: /remover/i })[1]);
    expect(confirmSpy).toHaveBeenCalledTimes(1);
    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith('/src/participacoes/2', { method: 'DELETE' })
    );
  });

  it('flags a broken vinculante and stays silent when it resolves', async () => {
    const { unmount } = render(
      <ParticipacoesEditor acaoId={7} vinculo={{ text: '0123.456/2025', resolvido: false }} />
    );
    await waitFor(() =>
      expect(screen.getByText(/vínculo não encontrado/i)).toBeDefined()
    );
    unmount();

    render(<ParticipacoesEditor acaoId={7} vinculo={{ text: '0123.456/2025', resolvido: true }} />);
    await waitFor(() => expect(screen.getByText('João Pedro Alves')).toBeDefined());
    expect(screen.queryByText(/vínculo não encontrado/i)).toBeNull();
  });
});

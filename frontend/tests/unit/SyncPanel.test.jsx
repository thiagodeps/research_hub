/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, waitFor, cleanup, act } from '@testing-library/react';

const mockApiFetch = vi.fn();
let progressCallback = null;
const mockOnSyncProgress = vi.fn((cb) => {
  progressCallback = cb;
  return Promise.resolve(() => {});
});

vi.mock('../../src/services/api', () => ({
  apiFetch: (...args) => mockApiFetch(...args),
  onSyncProgress: (...args) => mockOnSyncProgress(...args),
}));

import SyncPanel from '../../src/components/SyncPanel.jsx';

describe('SyncPanel (Download - US1)', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
    mockOnSyncProgress.mockClear();
    progressCallback = null;
  });

  afterEach(() => cleanup());

  it('loads config and pre-fills URL input with last_source_url', async () => {
    mockApiFetch.mockResolvedValueOnce({
      repo: 'owner/repo',
      branch: 'main',
      path: 'exports_canonical.zip',
      last_source_url: 'https://raw.githubusercontent.com/owner/repo/main/exports_canonical.zip',
      has_token: true,
      token_hint: 'abcd',
    });

    render(<SyncPanel project="horizon" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    const input = screen.getByRole('textbox', { name: /url do arquivo/i });
    expect(input.value).toBe('https://raw.githubusercontent.com/owner/repo/main/exports_canonical.zip');
    expect(mockOnSyncProgress).toHaveBeenCalled();
  });

  it('triggers download and displays summary on success', async () => {
    mockApiFetch
      .mockResolvedValueOnce({
        repo: '',
        branch: '',
        path: '',
        last_source_url: null,
        has_token: false,
        token_hint: null,
      })
      .mockResolvedValueOnce({
        total_rows: 150,
        tables: [{ table: 'campuses', rows: 2 }],
        snapshot: '/dados/snapshots/hub-123.db',
      });

    render(<SyncPanel project="horizon" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    const input = screen.getByRole('textbox', { name: /url do arquivo/i });
    fireEvent.change(input, {
      target: { value: 'https://raw.githubusercontent.com/a/b/main/exports.zip' },
    });

    const button = screen.getByRole('button', { name: /baixar e importar/i });
    fireEvent.click(button);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/download', {
        method: 'POST',
        body: JSON.stringify({
          project: 'horizon',
          url: 'https://raw.githubusercontent.com/a/b/main/exports.zip',
        }),
      });
    });

    await waitFor(() => {
      expect(screen.getByText(/150 registros/i)).toBeDefined();
    });
    expect(screen.getByText(/hub-123\.db/i)).toBeDefined();
  });

  it('displays progress updates from onSyncProgress', async () => {
    mockApiFetch
      .mockResolvedValueOnce({
        repo: '',
        branch: '',
        path: '',
        last_source_url: null,
        has_token: false,
        token_hint: null,
      })
      .mockImplementationOnce(() => new Promise(() => {})); // stays pending

    render(<SyncPanel project="horizon" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    const input = screen.getByRole('textbox', { name: /url do arquivo/i });
    fireEvent.change(input, {
      target: { value: 'https://raw.githubusercontent.com/a/b/main/exports.zip' },
    });

    fireEvent.click(screen.getByRole('button', { name: /baixar e importar/i }));

    act(() => {
      if (progressCallback) {
        progressCallback({
          operation: 'download',
          project: 'horizon',
          phase: 'transferring',
          bytes_done: 1024,
          bytes_total: 2048,
        });
      }
    });

    expect(screen.getByText(/transferindo/i)).toBeDefined();
  });

  it('displays typed errors with kind and message', async () => {
    const error = new Error('URL exige acesso autenticado; configure o token');
    error.kind = 'github_api';

    mockApiFetch
      .mockResolvedValueOnce({
        repo: '',
        branch: '',
        path: '',
        last_source_url: null,
        has_token: false,
        token_hint: null,
      })
      .mockRejectedValueOnce(error);

    render(<SyncPanel project="horizon" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    const input = screen.getByRole('textbox', { name: /url do arquivo/i });
    fireEvent.change(input, {
      target: { value: 'https://github.com/privado/repo/blob/main/exports.zip' },
    });

    fireEvent.click(screen.getByRole('button', { name: /baixar e importar/i }));

    await waitFor(() => {
      expect(screen.getByText(/URL exige acesso autenticado/i)).toBeDefined();
    });
    expect(screen.getByText(/github_api/i)).toBeDefined();
  });

  it('does not render upload controls when allowUpload is false', async () => {
    mockApiFetch.mockResolvedValueOnce({
      repo: 'a/b',
      branch: 'main',
      path: 'x.zip',
      last_source_url: null,
      has_token: false,
      token_hint: null,
    });

    render(<SyncPanel project="src" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalled();
    });

    expect(screen.queryByRole('button', { name: /enviar para o github/i })).toBeNull();
  });
});

describe('SyncPanel (Upload - US2)', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
    mockOnSyncProgress.mockClear();
    progressCallback = null;
  });

  afterEach(() => cleanup());

  it('renders destination fields and privacy notice when allowUpload is true', async () => {
    mockApiFetch.mockResolvedValueOnce({
      repo: 'owner/repo',
      branch: 'main',
      path: 'exports/exports_canonical.zip',
      last_source_url: null,
      has_token: true,
      token_hint: 'abcd',
    });

    render(<SyncPanel project="horizon" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    expect(screen.getByRole('textbox', { name: /repositório/i })).toBeDefined();
    expect(screen.getByRole('textbox', { name: /branch/i })).toBeDefined();
    expect(screen.getByRole('textbox', { name: /caminho do arquivo/i })).toBeDefined();
    expect(screen.getByRole('button', { name: /enviar para o github/i })).toBeDefined();
    expect(screen.getByText(/dados pessoais|visibilidade|confidenciais/i)).toBeDefined();
  });

  it('validates owner/name repo format before saving or uploading', async () => {
    mockApiFetch.mockResolvedValueOnce({
      repo: '',
      branch: 'main',
      path: 'exports.zip',
      last_source_url: null,
      has_token: true,
      token_hint: 'abcd',
    });

    render(<SyncPanel project="horizon" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalled();
    });

    const repoInput = screen.getByRole('textbox', { name: /repositório/i });
    fireEvent.change(repoInput, { target: { value: 'invalid_repo_without_slash' } });

    fireEvent.click(screen.getByRole('button', { name: /salvar destino/i }));

    expect(screen.getByText(/owner\/name/i)).toBeDefined();
    expect(mockApiFetch).not.toHaveBeenCalledWith('/github/config', expect.anything());
  });

  it('uploads successfully when file does not exist in destination', async () => {
    mockApiFetch
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'main',
        path: 'exports/exports_canonical.zip',
        last_source_url: null,
        has_token: true,
        token_hint: 'abcd',
      })
      // GET /github/check?project=horizon
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'main',
        path: 'exports/exports_canonical.zip',
        branch_exists: true,
        file_exists: false,
        file_sha: null,
      })
      // POST /github/upload
      .mockResolvedValueOnce({
        commit_sha: '1234567890abcdef',
        html_url: 'https://github.com/owner/repo/commit/1234567890abcdef',
        replaced: false,
      });

    render(<SyncPanel project="horizon" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    fireEvent.click(screen.getByRole('button', { name: /enviar para o github/i }));

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/check?project=horizon');
      expect(mockApiFetch).toHaveBeenCalledWith('/github/upload', {
        method: 'POST',
        body: JSON.stringify({ project: 'horizon', confirmOverwrite: false }),
      });
    });

    await waitFor(() => {
      expect(screen.getByText(/enviado com sucesso/i)).toBeDefined();
    });
    const link = screen.getByRole('link', { name: /1234567/i });
    expect(link.getAttribute('href')).toBe('https://github.com/owner/repo/commit/1234567890abcdef');
  });

  it('prompts for overwrite confirmation when file exists in destination', async () => {
    window.confirm = vi.fn().mockReturnValue(true);

    mockApiFetch
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'main',
        path: 'exports/exports_canonical.zip',
        last_source_url: null,
        has_token: true,
        token_hint: 'abcd',
      })
      // GET /github/check
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'main',
        path: 'exports/exports_canonical.zip',
        branch_exists: true,
        file_exists: true,
        file_sha: 'old_sha_123',
      })
      // POST /github/upload with confirmOverwrite: true
      .mockResolvedValueOnce({
        commit_sha: 'new_commit_sha',
        html_url: 'https://github.com/owner/repo/commit/new_commit_sha',
        replaced: true,
      });

    render(<SyncPanel project="horizon" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    fireEvent.click(screen.getByRole('button', { name: /enviar para o github/i }));

    await waitFor(() => {
      expect(window.confirm).toHaveBeenCalledWith(
        expect.stringMatching(/já existe|sobrescrever/i),
      );
    });

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/upload', {
        method: 'POST',
        body: JSON.stringify({ project: 'horizon', confirmOverwrite: true }),
      });
    });
  });

  it('does not upload if user declines overwrite confirmation', async () => {
    window.confirm = vi.fn().mockReturnValue(false);

    mockApiFetch
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'main',
        path: 'exports/exports_canonical.zip',
        last_source_url: null,
        has_token: true,
        token_hint: 'abcd',
      })
      // GET /github/check
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'main',
        path: 'exports/exports_canonical.zip',
        branch_exists: true,
        file_exists: true,
        file_sha: 'old_sha_123',
      });

    render(<SyncPanel project="horizon" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    fireEvent.click(screen.getByRole('button', { name: /enviar para o github/i }));

    await waitFor(() => {
      expect(window.confirm).toHaveBeenCalled();
    });

    expect(mockApiFetch).not.toHaveBeenCalledWith('/github/upload', expect.anything());
  });

  it('displays conflict or sync_policy error on upload failure', async () => {
    const error = new Error('O consolidado do SRC contém dados pessoais');
    error.kind = 'sync_policy';

    mockApiFetch
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'main',
        path: 'dados/src.json',
        last_source_url: null,
        has_token: true,
        token_hint: 'abcd',
      })
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'main',
        path: 'dados/src.json',
        branch_exists: true,
        file_exists: false,
        file_sha: null,
      })
      .mockRejectedValueOnce(error);

    render(<SyncPanel project="src" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalled();
    });

    fireEvent.click(screen.getByRole('button', { name: /enviar para o github/i }));

    await waitFor(() => {
      expect(screen.getByText(/dados pessoais/i)).toBeDefined();
    });
    expect(screen.getByText(/sync_policy/i)).toBeDefined();
  });
});

describe('SyncPanel (Token integration - US3)', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
    mockOnSyncProgress.mockClear();
    progressCallback = null;
  });

  afterEach(() => cleanup());

  it('renders "Token de acesso…" button and opens TokenDialog on click', async () => {
    mockApiFetch.mockResolvedValueOnce({
      repo: '',
      branch: 'main',
      path: '',
      last_source_url: null,
      has_token: true,
      token_hint: '9876',
    });

    render(<SyncPanel project="horizon" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalled();
    });

    const tokenButton = screen.getByRole('button', { name: /^token de acesso/i });
    fireEvent.click(tokenButton);

    expect(screen.getByRole('dialog')).toBeDefined();
    expect(screen.getAllByText(/9876/).length).toBeGreaterThan(0);
  });

  it('shows token configure action on authentication error', async () => {
    const error = new Error('URL exige acesso autenticado; configure o token');
    error.kind = 'github_api';

    mockApiFetch
      .mockResolvedValueOnce({
        repo: '',
        branch: '',
        path: '',
        last_source_url: null,
        has_token: false,
        token_hint: null,
      })
      .mockRejectedValueOnce(error);

    render(<SyncPanel project="horizon" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalled();
    });

    const input = screen.getByRole('textbox', { name: /url do arquivo/i });
    fireEvent.change(input, {
      target: { value: 'https://github.com/privado/repo/blob/main/exports.zip' },
    });

    fireEvent.click(screen.getByRole('button', { name: /baixar e importar/i }));

    await waitFor(() => {
      expect(screen.getByText(/exige acesso autenticado/i)).toBeDefined();
    });

    const configButton = screen.getByRole('button', { name: /configurar token/i });
    fireEvent.click(configButton);
    expect(screen.getByRole('dialog')).toBeDefined();
  });
});



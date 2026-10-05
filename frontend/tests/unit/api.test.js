import { describe, it, expect, vi, beforeEach } from 'vitest';

// The bridge is a single mockable function, which makes this suite simpler
// than it was against fetch: no response objects, no status codes, no bodies.
const invoke = vi.fn();
const listen = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args) => invoke(...args) }));
vi.mock('@tauri-apps/api/event', () => ({ listen: (...args) => listen(...args) }));

const { apiFetch, onSyncProgress } = await import('../../src/services/api.js');

beforeEach(() => {
  invoke.mockReset();
  listen.mockReset();
});

describe('apiFetch routing', () => {
  it('lists with pagination, search and sort', async () => {
    invoke.mockResolvedValue({ items: [], total: 0 });
    await apiFetch('/campuses?limit=50&offset=100&search=serra&sort=name&order=desc');

    expect(invoke).toHaveBeenCalledWith('list_entities', {
      entity: 'campuses', limit: 50, offset: 100,
      search: 'serra', sort: 'name', order: 'desc',
    });
  });

  it('omits absent query parameters instead of sending zero', async () => {
    invoke.mockResolvedValue({ items: [], total: 0 });
    await apiFetch('/campuses');
    const args = invoke.mock.calls[0][1];
    expect(args.limit).toBeNull();
    expect(args.offset).toBeNull();
  });

  it('reads one record by id', async () => {
    invoke.mockResolvedValue({ id: 7 });
    await apiFetch('/campuses/7');
    expect(invoke).toHaveBeenCalledWith('get_entity', { entity: 'campuses', id: 7 });
  });

  it('creates from a JSON body', async () => {
    invoke.mockResolvedValue({ id: 1 });
    await apiFetch('/campuses', { method: 'POST', body: JSON.stringify({ name: 'Serra' }) });
    expect(invoke).toHaveBeenCalledWith('create_entity', {
      entity: 'campuses', payload: { name: 'Serra' },
    });
  });

  it('updates by id', async () => {
    invoke.mockResolvedValue({ id: 3 });
    await apiFetch('/campuses/3', { method: 'PUT', body: JSON.stringify({ name: 'X' }) });
    expect(invoke).toHaveBeenCalledWith('update_entity', {
      entity: 'campuses', id: 3, payload: { name: 'X' },
    });
  });

  // Regression for the bug this migration fixed: the old apiFetch parsed the
  // 204 empty body before checking status, so every successful delete threw.
  it('resolves on delete instead of throwing', async () => {
    invoke.mockResolvedValue(null);
    await expect(apiFetch('/campuses/3', { method: 'DELETE' })).resolves.toBeNull();
    expect(invoke).toHaveBeenCalledWith('delete_entity', { entity: 'campuses', id: 3 });
  });

  it('routes login', async () => {
    invoke.mockResolvedValue({ access_token: 't' });
    await apiFetch('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email: 'a@b.c', password: 'x' }),
    });
    expect(invoke).toHaveBeenCalledWith('login', { email: 'a@b.c', password: 'x' });
  });

  it('routes register', async () => {
    invoke.mockResolvedValue(null);
    await apiFetch('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ email: 'a@b.c', password: 'secretpassword', password_confirm: 'secretpassword' }),
    });
    expect(invoke).toHaveBeenCalledWith('register', {
      email: 'a@b.c',
      password: 'secretpassword',
      passwordConfirm: 'secretpassword',
    });
  });

  // `/link` is one segment, so it would match the generic create pattern if
  // the ordering were wrong.
  it('routes link before the generic create', async () => {
    invoke.mockResolvedValue({});
    await apiFetch('/link', {
      method: 'POST',
      body: JSON.stringify({ parent_type: 'researchers', parent_id: 1, child_type: 'groups', child_id: 2 }),
    });
    expect(invoke).toHaveBeenCalledWith('link_entities', {
      parentType: 'researchers', parentId: 1, childType: 'groups', childId: 2,
    });
  });

  it('routes merge with its entity', async () => {
    invoke.mockResolvedValue({});
    await apiFetch('/merge/groups', {
      method: 'POST',
      body: JSON.stringify({ source_ids: [1, 2], resolved_data: { name: 'U' } }),
    });
    expect(invoke).toHaveBeenCalledWith('merge_entities', {
      entity: 'groups', sourceIds: [1, 2], resolvedData: { name: 'U' },
    });
  });
});

// SEP-033: dedicated SRC commands — the generic Horizon routes never see them.
describe('apiFetch SRC routing', () => {
  it('routes the consolidated JSON import with the drag-drop path', async () => {
    invoke.mockResolvedValue({ total_acoes: 3 });
    await apiFetch('/src/import', { method: 'POST', body: JSON.stringify({ path: '/tmp/x.json' }) });
    expect(invoke).toHaveBeenCalledWith('import_src_json', { path: '/tmp/x.json' });
  });

  it('routes the import without a path when the dialog is opened in Rust', async () => {
    invoke.mockResolvedValue({ total_acoes: 0 });
    await apiFetch('/src/import', { method: 'POST' });
    expect(invoke).toHaveBeenCalledWith('import_src_json', { path: null });
  });

  it('routes the consolidated JSON export', async () => {
    invoke.mockResolvedValue({ total_acoes: 3, path: '/tmp/out.json' });
    await apiFetch('/src/export', { method: 'POST' });
    expect(invoke).toHaveBeenCalledWith('export_src_json');
  });

  it('reads and updates the SRC metadata', async () => {
    invoke.mockResolvedValue({ campus: 'Serra' });
    await apiFetch('/src/meta');
    expect(invoke).toHaveBeenCalledWith('src_get_meta');

    await apiFetch('/src/meta', { method: 'PUT', body: JSON.stringify({ campus: 'Serra' }) });
    expect(invoke).toHaveBeenCalledWith('src_update_meta', { campus: 'Serra' });
  });

  it('lists ações with pagination, search and sort on the SRC command', async () => {
    invoke.mockResolvedValue({ items: [], total: 0 });
    await apiFetch('/src/acoes?limit=50&offset=10&search=robotica&sort=titulo&order=desc');
    expect(invoke).toHaveBeenCalledWith('src_list_acoes', {
      limit: 50, offset: 10, search: 'robotica', sort: 'titulo', order: 'desc',
    });
  });

  it('creates and reads ações through the dedicated commands', async () => {
    invoke.mockResolvedValue({ id: 1 });
    await apiFetch('/src/acoes', {
      method: 'POST',
      body: JSON.stringify({ acao_id: '3001', titulo: 'Nova' }),
    });
    expect(invoke).toHaveBeenCalledWith('src_create_acao', {
      payload: { acao_id: '3001', titulo: 'Nova' },
    });

    await apiFetch('/src/acoes/7');
    expect(invoke).toHaveBeenCalledWith('src_get_acao', { id: 7 });

    await apiFetch('/src/acoes/7', {
      method: 'PUT',
      body: JSON.stringify({ titulo: 'Editada' }),
    });
    expect(invoke).toHaveBeenCalledWith('src_update_acao', { id: 7, payload: { titulo: 'Editada' } });
  });

  it('routes the forced delete as an explicit flag', async () => {
    invoke.mockResolvedValue(null);
    await apiFetch('/src/acoes/7', { method: 'DELETE' });
    expect(invoke).toHaveBeenCalledWith('src_delete_acao', { id: 7, force: false });

    await apiFetch('/src/acoes/7?force=true', { method: 'DELETE' });
    expect(invoke).toHaveBeenCalledWith('src_delete_acao', { id: 7, force: true });
  });

  it('routes participações under the ação, before the generic id pattern', async () => {
    invoke.mockResolvedValue([]);
    await apiFetch('/src/acoes/7/participacoes');
    expect(invoke).toHaveBeenCalledWith('src_list_participacoes', { acaoId: 7 });

    await apiFetch('/src/acoes/7/participacoes', {
      method: 'POST',
      body: JSON.stringify({ tipo: 'Público-alvo', Nome: 'X' }),
    });
    expect(invoke).toHaveBeenCalledWith('src_create_participacao', {
      acaoId: 7,
      payload: { tipo: 'Público-alvo', Nome: 'X' },
    });
  });

  it('updates and deletes participações by their own id', async () => {
    invoke.mockResolvedValue(null);
    await apiFetch('/src/participacoes/9', {
      method: 'PUT',
      body: JSON.stringify({ Nome: 'Y' }),
    });
    expect(invoke).toHaveBeenCalledWith('src_update_participacao', { id: 9, payload: { Nome: 'Y' } });

    await apiFetch('/src/participacoes/9', { method: 'DELETE' });
    expect(invoke).toHaveBeenCalledWith('src_delete_participacao', { id: 9 });
  });
});

// SEP-034: GitHub sync routing
describe('apiFetch GitHub routing', () => {
  it('reads config for a project', async () => {
    invoke.mockResolvedValue({ repo: 'owner/repo', branch: 'main', path: 'exp.zip', last_source_url: null, has_token: false, token_hint: null });
    await apiFetch('/github/config?project=horizon');
    expect(invoke).toHaveBeenCalledWith('github_get_config', { project: 'horizon' });
  });

  it('updates config for a project', async () => {
    invoke.mockResolvedValue(null);
    await apiFetch('/github/config', {
      method: 'POST',
      body: JSON.stringify({ project: 'horizon', repo: 'o/r', branch: 'main', path: 'exp.zip' }),
    });
    expect(invoke).toHaveBeenCalledWith('github_set_config', {
      project: 'horizon', repo: 'o/r', branch: 'main', path: 'exp.zip',
    });
  });

  it('triggers download with project and url', async () => {
    invoke.mockResolvedValue({ total_rows: 10 });
    await apiFetch('/github/download', {
      method: 'POST',
      body: JSON.stringify({ project: 'horizon', url: 'https://raw.githubusercontent.com/o/r/main/exp.zip' }),
    });
    expect(invoke).toHaveBeenCalledWith('github_download', {
      project: 'horizon', url: 'https://raw.githubusercontent.com/o/r/main/exp.zip',
    });
  });

  it('registers sync progress listener via onSyncProgress', async () => {
    const handler = vi.fn();
    listen.mockImplementation((event, cb) => {
      cb({ payload: { phase: 'transferring', bytes_done: 50 } });
      return Promise.resolve(() => {});
    });

    await onSyncProgress(handler);
    expect(listen).toHaveBeenCalledWith('sync://progress', expect.any(Function));
    expect(handler).toHaveBeenCalledWith({ phase: 'transferring', bytes_done: 50 });
  });

  it('checks destination for a project', async () => {
    invoke.mockResolvedValue({
      repo: 'owner/repo',
      branch: 'main',
      path: 'exp.zip',
      branch_exists: true,
      file_exists: false,
      file_sha: null,
    });
    await apiFetch('/github/check?project=horizon');
    expect(invoke).toHaveBeenCalledWith('github_check_destination', { project: 'horizon' });
  });

  it('triggers upload with project and confirmOverwrite', async () => {
    invoke.mockResolvedValue({
      commit_sha: 'c1',
      html_url: 'https://github.com/o/r/commit/c1',
      replaced: false,
    });
    await apiFetch('/github/upload', {
      method: 'POST',
      body: JSON.stringify({ project: 'horizon', confirmOverwrite: true }),
    });
    expect(invoke).toHaveBeenCalledWith('github_upload', {
      project: 'horizon',
      confirmOverwrite: true,
    });
  });

  it('saves personal access token', async () => {
    invoke.mockResolvedValue(null);
    await apiFetch('/github/token', {
      method: 'POST',
      body: JSON.stringify({ token: 'ghp_secret123' }),
    });
    expect(invoke).toHaveBeenCalledWith('github_save_token', {
      token: 'ghp_secret123',
    });
  });

  it('removes personal access token', async () => {
    invoke.mockResolvedValue(null);
    await apiFetch('/github/token', {
      method: 'DELETE',
    });
    expect(invoke).toHaveBeenCalledWith('github_remove_token');
  });

  it('tests personal access token', async () => {
    invoke.mockResolvedValue({ login: 'octocat', scopes: 'repo', valid: true });
    const result = await apiFetch('/github/token/test', {
      method: 'POST',
    });
    expect(invoke).toHaveBeenCalledWith('github_test_token');
    expect(result).toEqual({ login: 'octocat', scopes: 'repo', valid: true });
  });
});


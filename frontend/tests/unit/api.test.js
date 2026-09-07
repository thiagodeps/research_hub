import { describe, it, expect, vi, beforeEach } from 'vitest';

// The bridge is a single mockable function, which makes this suite simpler
// than it was against fetch: no response objects, no status codes, no bodies.
const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args) => invoke(...args) }));

const { apiFetch } = await import('../../src/services/api.js');

beforeEach(() => invoke.mockReset());

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

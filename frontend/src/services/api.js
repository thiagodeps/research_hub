import { invoke } from '@tauri-apps/api/core';

// IPC bridge (SEP-017). Every React component calls apiFetch, so translating
// here — and only here — keeps all of them unchanged. The observable contract is
// preserved on purpose: same arguments, parsed result, Error on failure.
//
// Matched in order; the first entry whose pattern and method both match wins.
// `/link` must precede the generic `/{entity}` POST, or it would be read as a
// request to create an entity named "link".
const ROUTES = [
  // SEP-033: dedicated SRC routes must precede the generic /{entity} patterns,
  // or "/src/acoes" would be read as a request to list an entity named "src".
  [/^\/src\/import$/, 'POST', (_m, b) =>
    invoke('import_src_json', { path: b?.path ?? null })],

  [/^\/src\/export$/, 'POST', () =>
    invoke('export_src_json')],

  [/^\/src\/meta$/, 'PUT', (_m, b) =>
    invoke('src_update_meta', { campus: b.campus ?? null })],

  [/^\/src\/meta$/, 'GET', () =>
    invoke('src_get_meta')],

  // SEP-033 / US3: curadoria das ações — mesmos parâmetros do genérico,
  // porém nos comandos dedicados (R6: o caminho do Horizon fica invariante).
  [/^\/src\/acoes$/, 'GET', (m, _b, q) =>
    invoke('src_list_acoes', {
      limit: q.has('limit') ? Number(q.get('limit')) : null,
      offset: q.has('offset') ? Number(q.get('offset')) : null,
      search: q.get('search'),
      sort: q.get('sort'),
      order: q.get('order'),
    })],

  [/^\/src\/acoes$/, 'POST', (_m, b) =>
    invoke('src_create_acao', { payload: b ?? {} })],

  // participações moram sob a ação; precisa vir antes de /src/acoes/:id.
  [/^\/src\/acoes\/(\d+)\/participacoes$/, 'GET', (m) =>
    invoke('src_list_participacoes', { acaoId: Number(m[1]) })],

  [/^\/src\/acoes\/(\d+)\/participacoes$/, 'POST', (m, b) =>
    invoke('src_create_participacao', { acaoId: Number(m[1]), payload: b ?? {} })],

  [/^\/src\/acoes\/(\d+)$/, 'GET', (m) =>
    invoke('src_get_acao', { id: Number(m[1]) })],

  [/^\/src\/acoes\/(\d+)$/, 'PUT', (m, b) =>
    invoke('src_update_acao', { id: Number(m[1]), payload: b ?? {} })],

  [/^\/src\/acoes\/(\d+)$/, 'DELETE', (m, _b, q) =>
    invoke('src_delete_acao', { id: Number(m[1]), force: q.get('force') === 'true' })],

  [/^\/src\/participacoes\/(\d+)$/, 'PUT', (m, b) =>
    invoke('src_update_participacao', { id: Number(m[1]), payload: b ?? {} })],

  [/^\/src\/participacoes\/(\d+)$/, 'DELETE', (m) =>
    invoke('src_delete_participacao', { id: Number(m[1]) })],

  [/^\/auth\/login$/, 'POST', (_m, b) =>
    invoke('login', { email: b.email, password: b.password })],

  [/^\/auth\/register$/, 'POST', (_m, b) =>
    invoke('register', {
      email: b.email,
      password: b.password,
      passwordConfirm: b.password_confirm,
    })],

  [/^\/merge\/([^/?]+)$/, 'POST', (m, b) =>
    invoke('merge_entities', {
      entity: m[1], sourceIds: b.source_ids, resolvedData: b.resolved_data,
    })],

  [/^\/link$/, 'POST', (_m, b) =>
    invoke('link_entities', {
      parentType: b.parent_type, parentId: b.parent_id,
      childType: b.child_type, childId: b.child_id,
    })],

  [/^\/([^/?]+)\/(\d+)$/, 'GET', (m) =>
    invoke('get_entity', { entity: m[1], id: Number(m[2]) })],

  [/^\/([^/?]+)\/(\d+)$/, 'PUT', (m, b) =>
    invoke('update_entity', { entity: m[1], id: Number(m[2]), payload: b })],

  [/^\/([^/?]+)\/(\d+)$/, 'DELETE', (m) =>
    invoke('delete_entity', { entity: m[1], id: Number(m[2]) })],

  [/^\/([^/?]+)$/, 'POST', (m, b) =>
    invoke('create_entity', { entity: m[1], payload: b ?? {} })],

  [/^\/([^/?]+)$/, 'GET', (m, _b, q) =>
    invoke('list_entities', {
      entity: m[1],
      limit: q.has('limit') ? Number(q.get('limit')) : null,
      offset: q.has('offset') ? Number(q.get('offset')) : null,
      search: q.get('search'),
      sort: q.get('sort'),
      order: q.get('order'),
    })],
];

// Rust sends { kind, message }; components expect an Error they can show.
function toError(err) {
  if (err && typeof err === 'object' && 'message' in err) return new Error(err.message);
  return new Error(typeof err === 'string' ? err : 'Erro inesperado');
}

export async function apiFetch(endpoint, options = {}) {
  const [path, qs = ''] = endpoint.split('?');
  const query = new URLSearchParams(qs);
  const method = (options.method || 'GET').toUpperCase();

  let body;
  if (options.body) {
    try {
      body = JSON.parse(options.body);
    } catch {
      throw new Error('Corpo de requisição inválido');
    }
  }

  for (const [pattern, verb, handler] of ROUTES) {
    const match = path.match(pattern);
    if (match && verb === method) {
      try {
        // A command returning unit arrives as null. DELETE relies on this:
        // the old code parsed the 204 empty body and threw on every success.
        return await handler(match, body, query);
      } catch (err) {
        throw toError(err);
      }
    }
  }

  throw new Error(`Rota não mapeada: ${method} ${path}`);
}

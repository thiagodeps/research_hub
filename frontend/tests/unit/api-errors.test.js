import { describe, it, expect, vi } from 'vitest';

// Kept in its own file: these cases reject the mock, and sharing a mockReset
// beforeEach with the routing suite makes vitest attribute the stored rejected
// results to whichever test is running when the mock is cleared.
const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args) => invoke(...args) }));

const { apiFetch } = await import('../../src/services/api.js');

async function failWith(thrown) {
  invoke.mockImplementation(async () => { throw thrown; });
  try {
    await apiFetch('/campuses/9');
  } catch (e) {
    return e;
  }
  throw new Error('apiFetch deveria ter falhado');
}

describe('apiFetch error contract', () => {
  // Components already do `catch (e) { alert(e.message) }`, so the bridge must
  // keep handing them an Error with a readable message.
  it('turns a Rust error into an Error carrying its message', async () => {
    const err = await failWith({ kind: 'not_found', message: 'Não encontrado' });
    expect(err).toBeInstanceOf(Error);
    expect(err.message).toBe('Não encontrado');
  });

  it('handles a plain string rejection', async () => {
    const err = await failWith('falhou');
    expect(err).toBeInstanceOf(Error);
    expect(err.message).toBe('falhou');
  });

  it('reports an unmapped route instead of failing silently', async () => {
    let caught;
    try {
      await apiFetch('/a/b/c/d');
    } catch (e) {
      caught = e;
    }
    expect(caught?.message).toMatch(/Rota não mapeada/);
  });
});

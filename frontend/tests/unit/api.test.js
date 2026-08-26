import { describe, it, expect, vi, beforeEach } from 'vitest';
import { apiFetch } from '../../src/services/api';

describe('API Client', () => {
  beforeEach(() => {
    global.fetch = vi.fn();
  });

  it('should make a GET request and return JSON', async () => {
    global.fetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ success: true })
    });

    const response = await apiFetch('/test');
    expect(response).toEqual({ success: true });
    expect(global.fetch).toHaveBeenCalledWith('/test', expect.objectContaining({
      method: 'GET',
      headers: expect.any(Object)
    }));
  });

  it('should throw an error when response is not ok', async () => {
    global.fetch.mockResolvedValueOnce({
      ok: false,
      status: 401,
      json: async () => ({ detail: 'Unauthorized' })
    });

    await expect(apiFetch('/test')).rejects.toThrow('Unauthorized');
  });
});

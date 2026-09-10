/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react';
import React from 'react';
import EntityPage from '../../src/components/EntityPage';
import { apiFetch } from '../../src/services/api';

vi.mock('../../src/services/api', () => ({
  apiFetch: vi.fn(),
}));

describe('EntityPage Pagination Navigation', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.setItem('token', 'fake-token');
  });

  afterEach(() => {
    cleanup();
  });

  it('renders page input and navigates on enter', async () => {
    apiFetch.mockResolvedValueOnce({
      items: Array.from({ length: 50 }, (_, i) => ({ id: i, name: `Item ${i}` })),
      total: 200,
    });

    render(<EntityPage entity="users" columns={['name']} fields={[{name: 'name', type: 'text'}]} />);
    
    // Wait for the data to load
    await waitFor(() => {
      expect(screen.getByText(/Mostrando 1 a 50 de 200 registros/i)).toBeDefined();
    });

    // The page input should show 1
    const pageInput = screen.getByLabelText('Ir para página');
    expect(pageInput.value).toBe('1');

    // Change value and press Enter
    fireEvent.change(pageInput, { target: { value: '3' } });
    
    apiFetch.mockResolvedValueOnce({
      items: Array.from({ length: 50 }, (_, i) => ({ id: i + 100, name: `Item ${i + 100}` })),
      total: 200,
    });

    fireEvent.keyDown(pageInput, { key: 'Enter', code: 'Enter' });

    await waitFor(() => {
      expect(screen.getByText(/Mostrando 101 a 150 de 200 registros/i)).toBeDefined();
    });
    
    // Also the API should have been called with offset 100
    expect(apiFetch).toHaveBeenNthCalledWith(2, expect.stringContaining('offset=100'), expect.anything());
  });

  it('clamps invalid inputs to bounds', async () => {
    apiFetch.mockResolvedValue({
      items: Array.from({ length: 50 }, (_, i) => ({ id: i, name: `Item ${i}` })),
      total: 100, // Max page is 2
    });

    render(<EntityPage entity="users" columns={['name']} fields={[]} />);
    
    await waitFor(() => {
      expect(screen.getByText(/Mostrando 1 a 50 de 100 registros/i)).toBeDefined();
    });

    const pageInput = screen.getByLabelText('Ir para página');
    
    // Attempt out of bounds > max
    fireEvent.change(pageInput, { target: { value: '5' } });
    apiFetch.mockResolvedValue({
      items: Array.from({ length: 50 }, (_, i) => ({ id: i + 50, name: `Item ${i + 50}` })),
      total: 100,
    });
    fireEvent.keyDown(pageInput, { key: 'Enter', code: 'Enter' });

    // Should clamp to page 2
    await waitFor(() => {
      expect(pageInput.value).toBe('2');
    });

    // Attempt out of bounds < 1
    fireEvent.change(pageInput, { target: { value: '-2' } });
    fireEvent.keyDown(pageInput, { key: 'Enter', code: 'Enter' });

    // Should clamp to page 1
    await waitFor(() => {
      expect(pageInput.value).toBe('1');
    });
  });
});

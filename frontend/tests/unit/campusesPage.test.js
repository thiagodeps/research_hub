import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

// SEP-032 / FR-007: the campus entity is only its real fields — the ghost
// "Campus (Vínculos)" column (a nested copy of the row itself, an upstream
// export bug) must be gone from the listing and the form.
const pagePath = join(
  dirname(fileURLToPath(import.meta.url)),
  '../../src/pages/dashboard/campuses.astro',
);
const source = readFileSync(pagePath, 'utf-8');

describe('Campuses dashboard page', () => {
  it('lists exactly id and name, without the ghost campus column', () => {
    expect(source).toContain("columns={['id', 'name']}");
    expect(source).not.toContain("'campus'");
  });

  it('offers no nested campus form field', () => {
    expect(source).not.toContain('Campus (Vínculos)');
    expect(source).not.toMatch(/name:\s*'campus'/);
  });
});

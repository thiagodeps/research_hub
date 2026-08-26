import { test, expect } from '@playwright/test';

test('login form flow', async ({ page }) => {
  await page.goto('/login');
  
  // Verify split screen structure roughly
  await expect(page.locator('form')).toBeVisible();
  
  // Fill the form
  await page.fill('input[type="email"]', 'admin@admin.com');
  await page.fill('input[type="password"]', 'senha_segura');
  await page.click('button[type="submit"]');

  // Should navigate or show success (since we don't have backend, it might just be the UI interaction)
  // For now just testing the existence and submissibility.
});

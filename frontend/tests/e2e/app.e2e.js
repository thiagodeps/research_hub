// End-to-end against the real bundled application, driven by tauri-driver.
// Playwright cannot drive a Tauri webview; WebdriverIO plus tauri-driver can.
// Linux and Windows only — which is every release target (decision Q2).
describe('ResearchHub', () => {
  it('opens on the login screen', async () => {
    await expect($('form')).toBeExisting();
    await expect($('input[type="password"]')).toBeExisting();
  });

  it('refuses invalid credentials', async () => {
    await $('input[type="email"]').setValue('admin@admin.com');
    await $('input[type="password"]').setValue('senha-errada');
    await $('button[type="submit"]').click();
    await expect($('body')).toHaveTextContaining('inválidas');
  });

  it('signs in and reaches the dashboard', async () => {
    await $('input[type="email"]').setValue('admin@admin.com');
    await $('input[type="password"]').setValue('admin123');
    await $('button[type="submit"]').click();
    await expect($('nav')).toBeExisting();
  });

  it('navigates to an entity page', async () => {
    await $('a[href="/dashboard/campuses"]').click();
    await expect($('table')).toBeExisting();
  });
});

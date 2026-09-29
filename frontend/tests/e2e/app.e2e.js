// End-to-end against the real bundled application, driven by tauri-driver.
// Playwright cannot drive a Tauri webview; WebdriverIO plus tauri-driver can.
// Linux and Windows only — which is every release target (decision Q2).
//
// SEP-033: the authenticated landing is now the project selection page
// (/projects); the suite follows the new flow and checks the separation
// between the Horizon and SRC workspaces (US1, FR-001..FR-005).
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

  it('signs in and reaches the project selection page', async () => {
    await $('input[type="email"]').setValue('admin@admin.com');
    await $('input[type="password"]').setValue('admin123');
    await $('button[type="submit"]').click();
    await expect($('body')).toHaveTextContaining('Em qual projeto');
    await expect($('h2=Horizon')).toBeExisting();
    await expect($('h2=SRC')).toBeExisting();
  });

  it('enters Horizon and reaches the dashboard with its own menu', async () => {
    await $('h2=Horizon').click();
    await expect($('nav')).toBeExisting();
    await expect($('a[href="/dashboard/researchers"]')).toBeExisting();
    // FR-004: no SRC content inside the Horizon workspace.
    await expect($('a[href="/src/acoes"]')).not.toBeExisting();
  });

  it('switches to the SRC project and sees its own empty workspace', async () => {
    await $('a[href="/projects"]').click();
    await expect($('h2=SRC')).toBeExisting();
    await $('h2=SRC').click();
    await expect($('body')).toHaveTextContaining('Projeto SRC');
    await expect($('body')).toHaveTextContaining('Nenhum dado carregado');
    // FR-004: no Horizon content inside the SRC workspace.
    await expect($('a[href="/dashboard/researchers"]')).not.toBeExisting();
  });
});

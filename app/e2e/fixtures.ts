import { test as base, expect, type Page } from '@playwright/test';

/**
 * Shared `test`: when a test fails, print what the page showed (visible text of <main>, alerts,
 * console errors, engine diagnostics) to stdout. CI artifacts aren't always reachable, so the
 * job log alone must say why a test failed.
 */
export const test = base.extend<{ failureDump: void }>({
  failureDump: [async ({ page }, use, info) => {
    const logs: string[] = [];
    page.on('console', (m) => { if (m.type() === 'error' || m.type() === 'warning') logs.push(`[${m.type()}] ${m.text()}`); });
    page.on('pageerror', (e) => logs.push(`[pageerror] ${e.message}`));
    await use();
    if (info.status === info.expectedStatus) return;
    const out: string[] = [`---- failure dump: ${info.title}`, `url: ${page.url()}`];
    try {
      const text = await page.evaluate(() => (document.querySelector('main') ?? document.body).innerText);
      out.push('main text:', text.slice(0, 1500));
      const alerts = await page.locator('[role=alert], .error').allInnerTexts();
      if (alerts.length) out.push('alerts:', ...alerts);
      const diag = await page.evaluate(() => (globalThis as unknown as { __kdDiagnostics?: () => Promise<string> }).__kdDiagnostics?.().catch((e) => String(e)));
      if (diag) out.push('engine diagnostics:', diag.slice(-1500));
    } catch (e) { out.push(`(page not readable: ${String(e)})`); }
    if (logs.length) out.push('console:', ...logs.slice(-30));
    console.log(out.join('\n'));
  }, { auto: true }],
});

export { expect, type Page };

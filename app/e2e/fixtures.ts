import { test as base, expect, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

/**
 * Shared `test`: when a test fails, print what the page showed (visible text of <main>, alerts,
 * console errors, engine diagnostics) to stdout. CI artifacts aren't always reachable, so the
 * job log alone must say why a test failed.
 */
export const test = base.extend<{ failureDump: void; settingsOpen: void }>({
  /** Each test gets its own on-disk browser profile, closed and deleted afterwards. Playwright's default
   *  contexts keep site storage in memory inside one long-lived browser; after ~20 full downloads in CI
   *  the 500 MB opendict import failed with Chromium "no space" (write returned -8) although disk and
   *  quota were free (runs 117–119, 123, 125). A fresh disk-backed profile per test can't accumulate. */
  context: async ({ playwright, launchOptions, viewport, deviceScaleFactor, isMobile, hasTouch, baseURL, colorScheme }, use) => {
    const dir = await mkdtemp(join(tmpdir(), 'kd-e2e-'));
    const ctx = await playwright.chromium.launchPersistentContext(dir, {
      ...launchOptions, viewport, deviceScaleFactor, isMobile, hasTouch, baseURL, colorScheme,
    });
    try {
      await use(ctx);
    } finally {
      await ctx.close().catch(() => undefined);
      await rm(dir, { recursive: true, force: true }).catch(() => undefined);
    }
  },
  /** Settings sections start collapsed for users; tests that use Settings expect them open
   *  (settings-groups.spec.ts tests the collapsing itself and clears this). */
  settingsOpen: [async ({ page }, use) => {
    await page.addInitScript(() => { try { if (localStorage.getItem('kd.settings.open') == null) localStorage.setItem('kd.settings.open', JSON.stringify(['appearance', 'dictionaries', 'data', 'backup', 'about'])); } catch { /* */ } });
    await use();
  }, { auto: true }],
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
      const extra = await page.evaluate(async () => {
        const g = globalThis as unknown as { __kdErrors?: () => { src: string; msg: string }[] };
        const est = await navigator.storage?.estimate?.().catch(() => undefined);
        let settings = ''; try { settings = localStorage.getItem('kd.settings') ?? ''; } catch { /* */ }
        return { errors: (g.__kdErrors?.() ?? []).map((e) => `[${e.src}] ${e.msg}`), usage: est?.usage, quota: est?.quota, settings: settings.slice(0, 400) };
      });
      out.push(`storage: ${extra.usage} of ${extra.quota}`, `settings: ${extra.settings}`);
      if (extra.errors.length) out.push('app error log:', ...extra.errors.slice(-15));
    } catch (e) { out.push(`(page not readable: ${String(e)})`); }
    if (logs.length) out.push('console:', ...logs.slice(-30));
    console.log(out.join('\n'));
  }, { auto: true }],
});

export { expect, type Page };

/** First-run Download, then wait until every selected pack has finished importing (the search
 *  box appears as soon as the core pack is usable; the optional packs keep importing after it). */
export async function downloadAll(page: Page) {
  await page.getByRole('button', { name: 'Download' }).click();
  await expect(page.getByRole('searchbox', { name: 'Search' })).toBeVisible({ timeout: 240_000 });
  await page.waitForFunction(() => (globalThis as unknown as { __kdInstalling?: () => number }).__kdInstalling?.() === 0, null, { timeout: 600_000 });
}
